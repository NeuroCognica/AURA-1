use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VerdictId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchetypeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Capability {
    Code,
    Planning,
    General,
    Voice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SentinelDecisionKind {
    Allow,
    AllowWithWarning,
    RequireConsent,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Low,
    Elevated,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
pub struct SentinelConstraint {
    pub domain: SentinelDomain,
    pub severity: Severity,
    pub rule_id: String,
    pub statement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OverrideScope {
    RequestOnly { request_hash: String },
    ActionClass { class: String },
    Capability { capability: Capability },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsentRequirement {
    pub exact_phrase: String,
    pub scope: OverrideScope,
    pub ttl_seconds: u64,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FinalState {
    Allowed,
    AllowedWithWarning,
    Blocked,
    RequireConsent,
    RequireUserChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NextRequired {
    None,
    ProvideConsent { verdict_id: VerdictId, exact_phrase: String },
    ChooseOption { verdict_id: VerdictId, option_ids: Vec<String> },
    ReframeRequest { hint: String },
    HardStop { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlchemistOption {
    pub option_id: String,
    pub title: String,
    pub description: String,
    pub minimal_changes: Vec<String>,
    pub satisfies_constraints: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
pub struct OverrideToken {
    pub token_id: String,
    pub scope: OverrideScope,
    pub issued_at_ms: u128,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppealState {
    Idle,
    AwaitingUser { verdict_id: VerdictId, required: NextRequired },
    Authorized { verdict_id: VerdictId, token: OverrideToken, expires_at_ms: u128 },
    InAlchemist { verdict_id: VerdictId },
    Closed { verdict_id: VerdictId, final_state: FinalState },
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
#[serde(rename_all = "snake_case")]
pub enum CouncilMsgType {
    Verdict,
    AppealState,
    SentinelNotice,
    Interrupt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptKind {
    HaltLanguage,
    LockTools,
    RequireConsent,
    HardDeny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptScope {
    Session,
    Action,
    Tool,
    Generation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptRequirements {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_phrase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CouncilClientMsg {
    Ack { ack: u64, sid: String },
    Hello { sid: String, last_ack: Option<u64> },
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
