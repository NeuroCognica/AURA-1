use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(feature = "schema-export")]
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct SessionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct VerdictId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct ArchetypeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub enum Capability {
    Code,
    Planning,
    General,
    Voice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub enum SentinelDecisionKind {
    Allow,
    AllowWithWarning,
    RequireConsent,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub enum Severity {
    Low,
    Elevated,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub enum SentinelDomain {
    Safety,
    Security,
    Ethics,
    Legal,
    Sovereignty,
    IrreversibleAction,
    Privacy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct SentinelConstraint {
    pub domain: SentinelDomain,
    pub severity: Severity,
    pub rule_id: String,
    pub statement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub enum OverrideScope {
    RequestOnly { request_hash: String },
    ActionClass { class: String },
    Capability { capability: Capability },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct ConsentRequirement {
    pub exact_phrase: String,
    pub scope: OverrideScope,
    pub ttl_seconds: u64,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub enum FinalState {
    Allowed,
    AllowedWithWarning,
    Blocked,
    RequireConsent,
    RequireUserChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub enum NextRequired {
    None,
    ProvideConsent {
        verdict_id: VerdictId,
        exact_phrase: String,
    },
    ChooseOption {
        verdict_id: VerdictId,
        option_ids: Vec<String>,
    },
    ReframeRequest {
        hint: String,
    },
    HardStop {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct AlchemistOption {
    pub option_id: String,
    pub title: String,
    pub description: String,
    pub minimal_changes: Vec<String>,
    pub satisfies_constraints: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct CouncilVerdict {
    pub verdict_id: VerdictId,
    pub session_id: SessionId,
    pub timestamp_ms: u128,

    pub active_archetype: ArchetypeId,
    pub capability: Capability,

    pub sentinel_kind: SentinelDecisionKind,
    pub sentinel_constraints: Vec<SentinelConstraint>,
    pub sentinel_message: String,

    pub consent: Option<ConsentRequirement>,

    pub dissent: BTreeMap<ArchetypeId, String>,
    pub alchemist_options: Vec<AlchemistOption>,

    pub final_state: FinalState,
    pub next_required: NextRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct OverrideToken {
    pub token_id: String,
    pub scope: OverrideScope,
    pub issued_at_ms: u128,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub enum AppealState {
    Idle,
    AwaitingUser {
        verdict_id: VerdictId,
        required: NextRequired,
    },
    Authorized {
        verdict_id: VerdictId,
        token: OverrideToken,
        expires_at_ms: u128,
    },
    InAlchemist {
        verdict_id: VerdictId,
    },
    Closed {
        verdict_id: VerdictId,
        final_state: FinalState,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentSubmitRequest {
    pub session_id: String,
    pub verdict_id: String,
    pub phrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentSubmitResponse {
    pub ok: bool,
    pub state: AppealState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemistInvokeRequest {
    pub session_id: String,
    pub verdict_id: String,
    pub preference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemistInvokeResponse {
    pub ok: bool,
    pub verdict: CouncilVerdict,
    pub state: AppealState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilWsMsg {
    pub kind: String,
    pub session_id: String,
    pub payload: serde_json::Value,
}

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CouncilMsgType {
    Verdict,
    AppealState,
    SentinelNotice,
    Interrupt,
    ProposedAction,
    SubTask,
    Deliberation,
    Decision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum InterruptKind {
    HaltLanguage,
    LockTools,
    RequireConsent,
    HardDeny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum InterruptScope {
    Session,
    Action,
    Tool,
    Generation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct InterruptRequirements {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_phrase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct InterruptPayload {
    pub kind: InterruptKind,
    pub scope: InterruptScope,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<InterruptRequirements>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct CouncilEnvelope {
    pub seq: u64,
    #[serde(rename = "type")]
    pub msg_type: CouncilMsgType,
    pub ts_ms: u64,
    pub sid: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid: Option<String>,

    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct ProposedAction {
    pub action_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct SubTask {
    pub parent_action_id: Option<String>,
    pub task_id: String,
    pub target_archetype: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct Deliberation {
    pub deliberation_id: String,
    pub summary: String,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
pub struct Decision {
    pub deliberation_id: String,
    pub decision: String,
    pub actor: String,
}

/// Typed message payloads for council envelopes. This enum is the
/// canonical typed representation; `make_council_envelope` converts
/// it into the existing `CouncilEnvelope` struct (preserving the
/// current serialized shape) so Pass 1 remains additive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(JsonSchema))]
#[serde(tag = "type", content = "payload")]
pub enum CouncilMsg {
    Verdict(CouncilVerdict),
    AppealState(AppealState),
    Interrupt(InterruptPayload),
    Notice(serde_json::Value),
    ProposedAction(ProposedAction),
    SubTask(SubTask),
    Deliberation(Deliberation),
    Decision(Decision),
}

/// Construct a `CouncilEnvelope` from typed pieces. `seq` is left to
/// the caller (broadcast layer will assign sequencing in Pass 2). The
/// function assigns `ts_ms` via `now_ms()` and serializes the typed
/// payload into `payload: serde_json::Value` while mapping to the
/// existing `CouncilMsgType` so the on-wire JSON remains unchanged.
pub fn make_council_envelope(
    sid: &str,
    vid: Option<String>,
    seq: u64,
    msg: CouncilMsg,
) -> CouncilEnvelope {
    let ts = now_ms();
    match msg {
        CouncilMsg::Verdict(v) => {
            let payload = serde_json::to_value(v).unwrap_or_else(|_| serde_json::json!({}));
            CouncilEnvelope {
                seq,
                msg_type: CouncilMsgType::Verdict,
                ts_ms: ts,
                sid: sid.to_string(),
                vid,
                payload,
            }
        }
        CouncilMsg::AppealState(s) => {
            let payload = serde_json::to_value(s).unwrap_or_else(|_| serde_json::json!({}));
            CouncilEnvelope {
                seq,
                msg_type: CouncilMsgType::AppealState,
                ts_ms: ts,
                sid: sid.to_string(),
                vid,
                payload,
            }
        }
        CouncilMsg::Interrupt(i) => {
            let payload = serde_json::to_value(i).unwrap_or_else(|_| serde_json::json!({}));
            CouncilEnvelope {
                seq,
                msg_type: CouncilMsgType::Interrupt,
                ts_ms: ts,
                sid: sid.to_string(),
                vid,
                payload,
            }
        }
        CouncilMsg::Notice(n) => CouncilEnvelope {
            seq,
            msg_type: CouncilMsgType::SentinelNotice,
            ts_ms: ts,
            sid: sid.to_string(),
            vid,
            payload: n,
        },
        CouncilMsg::ProposedAction(p) => {
            let payload = serde_json::to_value(p).unwrap_or_else(|_| serde_json::json!({}));
            CouncilEnvelope {
                seq,
                msg_type: CouncilMsgType::ProposedAction,
                ts_ms: ts,
                sid: sid.to_string(),
                vid,
                payload,
            }
        }
        CouncilMsg::SubTask(s2) => {
            let payload = serde_json::to_value(s2).unwrap_or_else(|_| serde_json::json!({}));
            CouncilEnvelope {
                seq,
                msg_type: CouncilMsgType::SubTask,
                ts_ms: ts,
                sid: sid.to_string(),
                vid,
                payload,
            }
        }
        CouncilMsg::Deliberation(d) => {
            let payload = serde_json::to_value(d).unwrap_or_else(|_| serde_json::json!({}));
            CouncilEnvelope {
                seq,
                msg_type: CouncilMsgType::Deliberation,
                ts_ms: ts,
                sid: sid.to_string(),
                vid,
                payload,
            }
        }
        CouncilMsg::Decision(dc) => {
            let payload = serde_json::to_value(dc).unwrap_or_else(|_| serde_json::json!({}));
            CouncilEnvelope {
                seq,
                msg_type: CouncilMsgType::Decision,
                ts_ms: ts,
                sid: sid.to_string(),
                vid,
                payload,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CouncilClientMsg {
    Ack { ack: u64, sid: String },
    Hello { sid: String, last_ack: Option<u64> },
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
