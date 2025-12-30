use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug)]
pub enum SentinelDecision {
    Allow,
    AllowWithWarning(String),
    RequireConsent(String),
    Deny(String),
}

/// Lightweight heuristic-based evaluator. Pure, fast, no network calls.
pub fn sentinel_evaluate(text: &str) -> SentinelDecision {
    let s = text.to_lowercase();

    // explicit summons (avoid overly broad tokens like plain "evaluate")
    let summons = ["sentinel", "enforce", "evaluate risk", "is this allowed", "is this safe"];
    for p in &summons {
        if s.contains(p) {
            return SentinelDecision::RequireConsent("explicit sentinel summon".to_string());
        }
    }

    // high-risk deny tokens
    let deny_tokens = ["suicide", "self-harm", "kill ", "weapon", "bomb", "poison", "explode", "harm others"];
    for t in &deny_tokens {
        if s.contains(t) {
            return SentinelDecision::Deny(format!("prohibited high-risk content detected: {}", t));
        }
    }

    // domain triggers that require consent/warning
    let consent_tokens = [
        "law", "legal", "liability", "compliance", "ethic", "ethics", "safety", "secure",
        "security", "exploit", "vulnerability", "delete ", "erase ", "publish", "release", "sovereign",
        "act for", "act on behalf",
    ];
    for t in &consent_tokens {
        if s.contains(t) {
            return SentinelDecision::RequireConsent(format!("trigger: {}", t));
        }
    }

    // low-severity warnings
    let warn_tokens = ["risk", "risky", "unsafe", "irreversible"];
    for t in &warn_tokens {
        if s.contains(t) {
            return SentinelDecision::AllowWithWarning(format!("flag: {}", t));
        }
    }

    SentinelDecision::Allow
}

#[derive(Serialize, Deserialize)]
struct OReqOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OReq<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OReqOptions>,
}

/// Call Ollama /api/generate once (non-streaming) for a short Sentinel message.
pub async fn sentinel_speak(ollama_url: &str, model: &str, system: &str) -> anyhow::Result<String> {
    let client = Client::new();

    let req = OReq {
        model,
        prompt: system,
        stream: false,
        options: Some(OReqOptions { num_ctx: Some(512), temperature: Some(0.0) }),
    };

    let res = client
        .post(format!("{}/api/generate", ollama_url.trim_end_matches('/')))
        .json(&req)
        .send()
        .await
        .context("failed to call ollama")?
        .error_for_status()
        .context("ollama returned error status")?;

    // Expect JSON with a `response` field or similar. Try to parse generically.
    let j: serde_json::Value = res.json().await.context("parsing ollama response json")?;
    if let Some(r) = j.get("response").and_then(|v| v.as_str()) {
        return Ok(r.to_string());
    }
    // fallback: stringify whole response
    Ok(serde_json::to_string(&j).unwrap_or_else(|_| "<no response>".to_string()))
}

/// Build a Sentinel-styled block text given reason and decision description.
pub fn format_sentinel_block(reason: &str, decision: &str) -> String {
    let ts = SystemTime::now();
    format!("[SENTINEL]\n\n{}\n\nReason: {}\nDecision: {}\n\nTo proceed, explicitly confirm: \"I understand the risks and accept responsibility.\"\n", "This action has legal and safety implications.", reason, decision)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case<'a> {
        input: &'a str,
        expect_variant: &'a str,
        expect_reason_sub: &'a str,
    }

    #[test]
    fn table_driven_sentinel_evaluate() {
        let cases = vec![
            Case { input: "Can you evaluate the legal liability here?", expect_variant: "RequireConsent", expect_reason_sub: "legal" },
            Case { input: "Sentinel, evaluate risk", expect_variant: "RequireConsent", expect_reason_sub: "explicit sentinel summon" },
            Case { input: "How to make a bomb", expect_variant: "Deny", expect_reason_sub: "prohibited" },
            Case { input: "This is risky and possibly irreversible", expect_variant: "AllowWithWarning", expect_reason_sub: "flag" },
            Case { input: "Hello, small talk only", expect_variant: "Allow", expect_reason_sub: "" },
        ];

        for c in cases {
            let res = sentinel_evaluate(c.input);
            match (res, c.expect_variant) {
                (SentinelDecision::Allow, "Allow") => { /* ok */ }
                (SentinelDecision::AllowWithWarning(s), "AllowWithWarning") => {
                    assert!(s.contains(c.expect_reason_sub), "expected reason substring '{}' in '{}'", c.expect_reason_sub, s);
                }
                (SentinelDecision::RequireConsent(s), "RequireConsent") => {
                    assert!(s.contains(c.expect_reason_sub), "expected reason substring '{}' in '{}'", c.expect_reason_sub, s);
                }
                (SentinelDecision::Deny(s), "Deny") => {
                    assert!(s.contains(c.expect_reason_sub), "expected reason substring '{}' in '{}'", c.expect_reason_sub, s);
                }
                (other, expect) => {
                    panic!("case failed: input='{}' expected='{}' got='{:?}'", c.input, expect, other);
                }
            }
        }
    }
}
