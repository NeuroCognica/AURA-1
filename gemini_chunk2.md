# Gemini Chunk 2: Mirrorborn Quiz & Profile System

## Architecture Overview

The Mirrorborn system generates cognitive profiles through structured psychological assessment, then synthesizes actionable intelligence for archetype tuning.

**Flow:**
```
Quiz Ritual → Feature Detection → Weight Accumulation → Profile Synthesis → Artifact Storage → Ledger Event
```

## Quiz Structure

**80 Questions × 3 Probes = 240 Total Assessments**

Probe Types:
- **Probe A**: Concrete Recall
- **Probe B**: Reflection & Interpretation  
- **Probe C**: Future Projection & Speculation

## Core Components

### 1. QuizState Engine

**`senkern/src/quiz/engine.rs`**
```rust
use std::collections::HashMap;

pub struct QuizState {
    pub session_id: String,
    pub current_question: usize,
    pub current_probe: usize,
    pub weights: HashMap<String, f64>,
    // ... other fields
}

impl QuizState {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            current_question: 0,
            current_probe: 0,
            weights: HashMap::new(),
        }
    }
    
    pub fn validate(&self, answer: &str) -> ValidationResult {
        ValidationResult {
            is_valid: answer.len() >= 20, // Minimum depth requirement
            score: calculate_complexity(answer),
        }
    }
    
    pub fn add_features(&mut self, features: Vec<String>) {
        for feature in features {
            *self.weights.entry(feature).or_insert(0.0) += 1.0;
        }
    }
    
    pub fn get_accumulated_weights(&self) -> HashMap<String, f64> {
        // Normalize weights to 0.0-1.0 range
        let max = self.weights.values().cloned().fold(0.0, f64::max);
        self.weights.iter()
            .map(|(k, v)| (k.clone(), v / max))
            .collect()
    }
    
    pub fn advance(&mut self) -> ProbeInfo {
        self.current_probe += 1;
        if self.current_probe >= 3 {
            self.current_probe = 0;
            self.current_question += 1;
        }
        self.current_probe_struct()
    }
    
    pub fn is_complete(&self) -> bool {
        self.current_question >= 80
    }
}
```

### 2. Profile Structure

**`senkern/src/quiz/profile.rs`**
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorbornProfile {
    pub session_id: String,
    pub timestamp: String,
    pub archetypes: HashMap<String, f64>,
    pub primary_archetype: String,
    pub secondary_archetype: String,
    pub traits: Vec<String>,
}

impl MirrorbornProfile {
    pub fn new(session_id: &str, weights: HashMap<String, f64>) -> Self {
        let mut sorted: Vec<_> = weights.iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        let primary = sorted.get(0)
            .map(|(k, _)| k.to_string())
            .unwrap_or("Unknown".into());
        let secondary = sorted.get(1)
            .map(|(k, _)| k.to_string())
            .unwrap_or("None".into());
        
        Self {
            session_id: session_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            archetypes: weights,
            primary_archetype: primary,
            secondary_archetype: secondary,
            traits: vec![],
        }
    }
}
```

### 3. Obsidian Commit Pattern

**The Forever Law: No State Without Persistence**

**`senkern/src/web.rs` - Quiz Answer Handler**
```rust
use crate::conversation_ledger::{self, EventType};
use serde_json::json;

pub async fn quiz_answer(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QuizAnswerRequest>,
) -> impl IntoResponse {
    let mut sessions = state.quiz_sessions.lock().unwrap();
    
    if let Some(quiz_state) = sessions.get_mut(&payload.session_id) {
        let validation = quiz_state.validate(&payload.answer_text);
        
        if validation.is_valid {
            // === OBSIDIAN COMMIT ===
            let ledger_payload = json!({
                "action": "quiz_response",
                "session_id": payload.session_id,
                "question_index": quiz_state.current_question,
                "probe_index": quiz_state.current_probe,
                "user_answer": payload.answer_text,
                "validation_score": validation.score
            });
            
            match conversation_ledger::append_event(
                &payload.session_id,
                "user",
                EventType::QuizEntry,
                ledger_payload
            ) {
                Ok(_) => {
                    eprintln!("Obsidian Commit: Q{}-{} saved", 
                        quiz_state.current_question, 
                        quiz_state.current_probe);
                }
                Err(e) => {
                    eprintln!("CRITICAL FAILURE: Could not persist answer: {}", e);
                    // Return error to enforce "No State Without Persistence"
                    return Json(QuizAnswerResponse {
                        ok: false,
                        error: Some("Ledger write failed".to_string()),
                        ..Default::default()
                    });
                }
            }
            
            // Accumulate features
            let detected_features = extract_cognitive_features(&payload.answer_text);
            quiz_state.add_features(detected_features);
            
            // Advance state
            let next_probe = quiz_state.advance();
            let is_complete = quiz_state.is_complete();
            
            // === SYNTHESIS ON COMPLETION ===
            if is_complete {
                let profile = MirrorbornProfile::new(
                    &payload.session_id,
                    quiz_state.get_accumulated_weights()
                );
                
                // Write artifact
                let filename = format!("codex/artifacts/profile_{}.json", payload.session_id);
                std::fs::create_dir_all("codex/artifacts")?;
                std::fs::write(&filename, serde_json::to_string_pretty(&profile)?)?;
                
                // Emit ProfileGenerated event
                conversation_ledger::append_event(
                    &payload.session_id,
                    "system",
                    EventType::ProfileGenerated,
                    json!({
                        "path": filename,
                        "primary": profile.primary_archetype
                    })
                )?;
            }
            
            return Json(QuizAnswerResponse {
                ok: true,
                next_probe,
                completed: is_complete,
                ..Default::default()
            });
        }
    }
    
    Json(QuizAnswerResponse { ok: false, ..Default::default() })
}
```

### 4. EventType Enum

**`senkern/src/conversation_ledger.rs`**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    ConversationStart,
    MessageUser,
    MessageSystem,
    QuizEntry,           // Added for quiz answers
    ProfileGenerated,    // Added for synthesis completion
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::QuizEntry => "quiz_entry",
            Self::ProfileGenerated => "profile_generated",
            // ... other variants
        }
    }
}
```

### 5. Integration Test Script

**`senkern/tools/verify_mirrorborn.py`**
```python
#!/usr/bin/env python3
import urllib.request
import json
import time
import os

API_URL = os.getenv("API_URL", "http://127.0.0.1:8080")
SESSION_ID = f"verify_{int(time.time())}"

def api_post(endpoint, payload):
    url = f"{API_URL}{endpoint}"
    data = json.dumps(payload).encode('utf-8')
    req = urllib.request.Request(url, data=data, 
                                  headers={'Content-Type': 'application/json'})
    with urllib.request.urlopen(req) as response:
        return json.loads(response.read())

def run_quiz_speedrun():
    # Start session
    api_post("/api/quiz/start", {"session_id": SESSION_ID})
    
    # Answer all probes
    steps = 0
    while steps < 240:
        answer = f"Verification answer {steps} with sufficient depth for validation."
        res = api_post("/api/quiz/answer", {
            "session_id": SESSION_ID,
            "answer_text": answer
        })
        
        if not res["ok"]:
            raise Exception(f"Failed at step {steps}: {res.get('error')}")
        
        if res.get("completed"):
            print(f"✓ Quiz completed in {steps + 1} steps")
            break
        steps += 1

def audit_artifacts():
    profile_path = f"./codex/artifacts/profile_{SESSION_ID}.json"
    
    if not os.path.exists(profile_path):
        raise Exception(f"Missing profile artifact: {profile_path}")
    
    with open(profile_path) as f:
        profile = json.load(f)
    
    assert "archetypes" in profile
    assert "primary_archetype" in profile
    print(f"✓ Profile artifact valid")
    print(f"  Primary: {profile['primary_archetype']}")
    print(f"  Weights: {len(profile['archetypes'])}")

if __name__ == "__main__":
    run_quiz_speedrun()
    time.sleep(1)
    audit_artifacts()
    print("[SUCCESS] Mirrorborn Integration Verified")
```

## Key Takeaways

1. **Immediate Ledger Persistence**: Every answer commits before state advances
2. **Profile Synthesis**: Weights normalized on completion, artifact generated
3. **Event-Sourced Recovery**: Quiz state can be rebuilt from ledger replay
4. **Constitutional Validation**: Answers must meet depth requirements
5. **Integration Signal**: ProfileGenerated event triggers mecha archetype tuning
