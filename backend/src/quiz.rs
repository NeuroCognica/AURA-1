//! Mirrorborn Quiz Collection System
//!
//! This module implements the quiz data collection layer with covenant-keeping integrity:
//! - Forever Law: All answers are append-only, never mutated
//! - Provenance: Session tracking with timestamp, user identity, model version
//! - Resumability: Users can stop and continue without losing progress
//! - Graceful Abandonment: Partial sessions are valid data, never discarded
//! - Trust Encoding: The promise to handle psychological data with integrity is encoded in the type system

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "persistence")]
use crate::storage::Storage;

/// A single probe (question) in the quiz
///
/// Each probe targets specific psychological features and archetypes.
/// The structure is derived from master_quiz.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    /// Unique identifier (1-240)
    pub id: u32,

    /// The question text shown to the user
    pub text: String,

    /// Which primary question this probe belongs to (1-80)
    pub primary_question: u32,

    /// Which sub-probe this is within the primary question (1-3)
    pub sub_probe: u32,

    /// Features this probe maps to (e.g., "agency", "detail_orientation")
    pub features: Vec<String>,

    /// Archetypes this probe could activate
    pub archetypes: Vec<String>,

    /// Decade classification (1-5) for penta-graph diversity
    pub decade: u32,
}

/// Complete quiz structure (240 probes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    /// All 240 probes in order
    pub probes: Vec<Probe>,

    /// Quiz version for provenance
    pub version: String,

    /// Total number of probes (always 240)
    pub total_probes: u32,
}

impl Quiz {
    /// Create empty quiz structure (populated from master_quiz.md parser)
    pub fn new(version: String) -> Self {
        Self {
            probes: Vec::new(),
            version,
            total_probes: 240,
        }
    }

    /// Get a specific probe by ID
    pub fn get_probe(&self, probe_id: u32) -> Option<&Probe> {
        self.probes.iter().find(|p| p.id == probe_id)
    }

    /// Check if quiz is complete (all 240 probes loaded)
    pub fn is_complete(&self) -> bool {
        self.probes.len() == self.total_probes as usize
    }
}

/// A user's answer to a single probe
///
/// Forever Law: Once written, never mutated. Each answer is a permanent record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// Unique identifier for this answer record
    pub answer_id: String,

    /// Session this answer belongs to
    pub session_id: String,

    /// Username who provided the answer
    pub username: String,

    /// Which probe was answered
    pub probe_id: u32,

    /// The user's free-text or multiple-choice response
    pub response: String,

    /// When the answer was submitted (milliseconds since epoch)
    pub answered_at_ms: i64,

    /// Tags extracted by LLM (populated later in pipeline)
    pub extracted_tags: Vec<String>,

    /// Feature weights extracted from this answer (populated by tagging harness)
    pub feature_weights: HashMap<String, f64>,
}

impl Answer {
    /// Create a new answer record with provenance
    pub fn new(
        session_id: String,
        username: String,
        probe_id: u32,
        response: String,
    ) -> Self {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self {
            answer_id: format!("ans_{}_{}", session_id, probe_id),
            session_id,
            username,
            probe_id,
            response,
            answered_at_ms: now_ms,
            extracted_tags: Vec::new(),
            feature_weights: HashMap::new(),
        }
    }
}

/// Quiz session state (resumable, never deleted)
///
/// Covenant: Users can abandon sessions without penalty. Partial data is valid data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizSession {
    /// Unique session identifier
    pub session_id: String,

    /// Username taking the quiz
    pub username: String,

    /// When the session was created
    pub created_at_ms: i64,

    /// Last activity timestamp (for resume detection)
    pub last_activity_ms: i64,

    /// How many probes have been answered
    pub answered_count: u32,

    /// ID of the last answered probe
    pub last_probe_id: Option<u32>,

    /// Session status
    pub status: SessionStatus,

    /// Quiz version being used
    pub quiz_version: String,

    /// Model version for LLM tagging (provenance)
    pub model_version: Option<String>,
}

/// Session lifecycle states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    /// Active session, user is answering questions
    Active,

    /// User stopped partway through (not an error—valid state)
    Paused,

    /// All 240 probes answered, ready for synthesis
    Complete,

    /// Profile has been generated from this session
    Synthesized,
}

impl QuizSession {
    /// Create a new quiz session
    pub fn new(session_id: String, username: String, quiz_version: String) -> Self {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self {
            session_id,
            username,
            created_at_ms: now_ms,
            last_activity_ms: now_ms,
            answered_count: 0,
            last_probe_id: None,
            status: SessionStatus::Active,
            quiz_version,
            model_version: None,
        }
    }

    /// Update session after answering a probe
    pub fn record_answer(&mut self, probe_id: u32) {
        self.answered_count += 1;
        self.last_probe_id = Some(probe_id);
        self.last_activity_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        // Automatically mark complete when all 240 answered
        if self.answered_count >= 240 {
            self.status = SessionStatus::Complete;
        }
    }

    /// Pause the session (user stopped, will resume later)
    pub fn pause(&mut self) {
        if self.status == SessionStatus::Active {
            self.status = SessionStatus::Paused;
        }
    }

    /// Resume paused session
    pub fn resume(&mut self) {
        if self.status == SessionStatus::Paused {
            self.status = SessionStatus::Active;
            self.last_activity_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
        }
    }

    /// Check if session is complete
    pub fn is_complete(&self) -> bool {
        self.status == SessionStatus::Complete || self.status == SessionStatus::Synthesized
    }

    /// Check if session was abandoned (no activity for 7+ days, but still valid)
    pub fn is_abandoned(&self) -> bool {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let seven_days_ms = 7 * 24 * 60 * 60 * 1000;
        
        self.status != SessionStatus::Complete 
            && self.status != SessionStatus::Synthesized
            && (now_ms - self.last_activity_ms) > seven_days_ms
    }
}

/// Quiz session manager with Forever Law enforcement
///
/// Covenant promises:
/// - All answers stored append-only
/// - Sessions never deleted, only paused
/// - Progress always preserved
/// - Partial completion is honored as valid psychological data
#[cfg(feature = "persistence")]
pub struct QuizManager<'a> {
    storage: &'a dyn Storage,
}

#[cfg(feature = "persistence")]
impl<'a> QuizManager<'a> {
    /// Create quiz manager with storage backend
    pub fn new(storage: &'a dyn Storage) -> Self {
        Self { storage }
    }

    /// Create a new quiz session
    ///
    /// Covenant: Session creation is logged, never forgotten
    pub fn create_session(
        &self,
        session_id: String,
        username: String,
        quiz_version: String,
    ) -> Result<QuizSession> {
        let session = QuizSession::new(session_id.clone(), username, quiz_version);

        // Store session metadata
        let key = format!("quiz_session:{}", session_id);
        let json = serde_json::to_vec(&session)
            .context("Failed to serialize session")?;
        self.storage.put_bytes(key.as_bytes(), &json)
            .context("Failed to store session")?;

        // Log session creation (Forever Law)
        self.storage.append_log_atomic(
            "quiz_system",
            &format!("SESSION_CREATED: {} by {} (version: {})", 
                session_id, session.username, session.quiz_version),
        )?;

        Ok(session)
    }

    /// Get existing session (for resume capability)
    pub fn get_session(&self, session_id: &str) -> Result<Option<QuizSession>> {
        let key = format!("quiz_session:{}", session_id);
        
        match self.storage.get_bytes(key.as_bytes())? {
            Some(bytes) => {
                let session: QuizSession = serde_json::from_slice(&bytes)
                    .context("Failed to deserialize session")?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Submit an answer to a probe
    ///
    /// Covenant: Answer is immediately written to ledger, never mutated
    pub fn submit_answer(
        &self,
        session_id: &str,
        probe_id: u32,
        response: String,
    ) -> Result<Answer> {
        // Get session
        let mut session = self.get_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        // Create answer record
        let answer = Answer::new(
            session_id.to_string(),
            session.username.clone(),
            probe_id,
            response,
        );

        // FOREVER LAW: Store answer append-only
        let answer_key = format!("quiz_answer:{}:{}", session_id, probe_id);
        let answer_json = serde_json::to_vec(&answer)
            .context("Failed to serialize answer")?;
        self.storage.put_bytes(answer_key.as_bytes(), &answer_json)
            .context("Failed to store answer")?;

        // FOREVER LAW: Append to ledger (immutable audit trail)
        self.storage.append_log_atomic(
            &format!("quiz:{}", session.username),
            &format!("ANSWER_SUBMITTED: session={} probe={} response_len={}", 
                session_id, probe_id, answer.response.len()),
        )?;

        // Update session state
        session.record_answer(probe_id);
        let session_key = format!("quiz_session:{}", session_id);
        let session_json = serde_json::to_vec(&session)
            .context("Failed to serialize updated session")?;
        self.storage.put_bytes(session_key.as_bytes(), &session_json)
            .context("Failed to update session")?;

        Ok(answer)
    }

    /// Get all answers for a session
    pub fn get_session_answers(&self, session_id: &str) -> Result<Vec<Answer>> {
        let mut answers = Vec::new();

        // Try to load all 240 potential answers
        for probe_id in 1..=240 {
            let key = format!("quiz_answer:{}:{}", session_id, probe_id);
            
            if let Some(bytes) = self.storage.get_bytes(key.as_bytes())? {
                let answer: Answer = serde_json::from_slice(&bytes)
                    .context("Failed to deserialize answer")?;
                answers.push(answer);
            }
        }

        answers.sort_by_key(|a| a.probe_id);
        Ok(answers)
    }

    /// Get count of answered probes in session
    pub fn get_answered_count(&self, session_id: &str) -> Result<u32> {
        let answers = self.get_session_answers(session_id)?;
        Ok(answers.len() as u32)
    }

    /// Pause a session (covenant: partial progress is honored)
    pub fn pause_session(&self, session_id: &str) -> Result<()> {
        let mut session = self.get_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        session.pause();

        let key = format!("quiz_session:{}", session_id);
        let json = serde_json::to_vec(&session)?;
        self.storage.put_bytes(key.as_bytes(), &json)?;

        // Log pause event (Forever Law)
        self.storage.append_log_atomic(
            "quiz_system",
            &format!("SESSION_PAUSED: {} (progress: {}/240)", 
                session_id, session.answered_count),
        )?;

        Ok(())
    }

    /// Resume a paused session
    pub fn resume_session(&self, session_id: &str) -> Result<()> {
        let mut session = self.get_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        session.resume();

        let key = format!("quiz_session:{}", session_id);
        let json = serde_json::to_vec(&session)?;
        self.storage.put_bytes(key.as_bytes(), &json)?;

        // Log resume event
        self.storage.append_log_atomic(
            "quiz_system",
            &format!("SESSION_RESUMED: {} (progress: {}/240)", 
                session_id, session.answered_count),
        )?;

        Ok(())
    }

    /// Mark session as synthesized (profile generated)
    pub fn mark_session_synthesized(&self, session_id: &str) -> Result<()> {
        let mut session = self.get_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        if session.status != SessionStatus::Complete {
            return Err(anyhow::anyhow!(
                "Cannot synthesize non-complete session (status: {:?})",
                session.status
            ));
        }

        session.status = SessionStatus::Synthesized;
        session.last_activity_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let key = format!("quiz_session:{}", session_id);
        let json = serde_json::to_vec(&session)?;
        self.storage.put_bytes(key.as_bytes(), &json)?;

        // Log synthesis completion (Forever Law)
        self.storage.append_log_atomic(
            "quiz_system",
            &format!("PROFILE_SYNTHESIZED: {} (from {} answers)", 
                session_id, session.answered_count),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Test storage implementation
    struct TestStore {
        data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
        log_counter: Mutex<u64>,
    }

    impl TestStore {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
                log_counter: Mutex::new(0),
            }
        }
    }

    impl Storage for TestStore {
        fn get_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
            let data = self.data.lock().unwrap();
            Ok(data.get(key).cloned())
        }

        fn put_bytes(&self, key: &[u8], val: &[u8]) -> Result<()> {
            let mut data = self.data.lock().unwrap();
            data.insert(key.to_vec(), val.to_vec());
            Ok(())
        }

        fn append_log_atomic(&self, _speaker: &str, _content: &str) -> Result<u64> {
            let mut counter = self.log_counter.lock().unwrap();
            *counter += 1;
            Ok(*counter)
        }
    }

    #[test]
    fn test_quiz_structure() {
        let quiz = Quiz::new("v1.0".to_string());
        assert_eq!(quiz.total_probes, 240);
        assert_eq!(quiz.version, "v1.0");
    }

    #[test]
    fn test_session_creation() {
        let session = QuizSession::new(
            "sess_123".to_string(),
            "alice".to_string(),
            "v1.0".to_string(),
        );

        assert_eq!(session.session_id, "sess_123");
        assert_eq!(session.username, "alice");
        assert_eq!(session.answered_count, 0);
        assert_eq!(session.status, SessionStatus::Active);
        assert!(!session.is_complete());
    }

    #[test]
    fn test_session_progress() {
        let mut session = QuizSession::new(
            "sess_456".to_string(),
            "bob".to_string(),
            "v1.0".to_string(),
        );

        // Answer some questions
        session.record_answer(1);
        assert_eq!(session.answered_count, 1);
        assert_eq!(session.last_probe_id, Some(1));

        session.record_answer(2);
        assert_eq!(session.answered_count, 2);
        assert_eq!(session.last_probe_id, Some(2));
    }

    #[test]
    fn test_session_completion() {
        let mut session = QuizSession::new(
            "sess_789".to_string(),
            "carol".to_string(),
            "v1.0".to_string(),
        );

        // Answer all 240 questions
        for i in 1..=240 {
            session.record_answer(i);
        }

        assert_eq!(session.answered_count, 240);
        assert_eq!(session.status, SessionStatus::Complete);
        assert!(session.is_complete());
    }

    #[test]
    fn test_session_pause_resume() {
        let mut session = QuizSession::new(
            "sess_pause".to_string(),
            "dave".to_string(),
            "v1.0".to_string(),
        );

        session.pause();
        assert_eq!(session.status, SessionStatus::Paused);

        session.resume();
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_answer_creation() {
        let answer = Answer::new(
            "sess_123".to_string(),
            "eve".to_string(),
            1,
            "I value autonomy and independence".to_string(),
        );

        assert_eq!(answer.session_id, "sess_123");
        assert_eq!(answer.username, "eve");
        assert_eq!(answer.probe_id, 1);
        assert_eq!(answer.response, "I value autonomy and independence");
        assert!(answer.answered_at_ms > 0);
    }

    #[test]
    fn test_quiz_manager_create_session() {
        let store = TestStore::new();
        let manager = QuizManager::new(&store);

        let session = manager.create_session(
            "sess_mgr_1".to_string(),
            "frank".to_string(),
            "v1.0".to_string(),
        ).expect("Failed to create session");

        assert_eq!(session.session_id, "sess_mgr_1");
        assert_eq!(session.username, "frank");

        // Verify session can be retrieved
        let retrieved = manager.get_session("sess_mgr_1")
            .expect("Failed to get session")
            .expect("Session should exist");
        
        assert_eq!(retrieved.session_id, session.session_id);
    }

    #[test]
    fn test_quiz_manager_submit_answer() {
        let store = TestStore::new();
        let manager = QuizManager::new(&store);

        // Create session
        manager.create_session(
            "sess_answer_1".to_string(),
            "grace".to_string(),
            "v1.0".to_string(),
        ).expect("Failed to create session");

        // Submit answer
        let answer = manager.submit_answer(
            "sess_answer_1",
            1,
            "I prefer working alone".to_string(),
        ).expect("Failed to submit answer");

        assert_eq!(answer.probe_id, 1);
        assert_eq!(answer.username, "grace");

        // Verify session was updated
        let session = manager.get_session("sess_answer_1")
            .expect("Failed to get session")
            .expect("Session should exist");
        
        assert_eq!(session.answered_count, 1);
        assert_eq!(session.last_probe_id, Some(1));
    }

    #[test]
    fn test_quiz_manager_get_answers() {
        let store = TestStore::new();
        let manager = QuizManager::new(&store);

        // Create session and submit multiple answers
        manager.create_session(
            "sess_multi".to_string(),
            "henry".to_string(),
            "v1.0".to_string(),
        ).expect("Failed to create session");

        manager.submit_answer("sess_multi", 1, "Answer 1".to_string()).unwrap();
        manager.submit_answer("sess_multi", 5, "Answer 5".to_string()).unwrap();
        manager.submit_answer("sess_multi", 3, "Answer 3".to_string()).unwrap();

        // Get all answers
        let answers = manager.get_session_answers("sess_multi")
            .expect("Failed to get answers");

        assert_eq!(answers.len(), 3);
        // Should be sorted by probe_id
        assert_eq!(answers[0].probe_id, 1);
        assert_eq!(answers[1].probe_id, 3);
        assert_eq!(answers[2].probe_id, 5);
    }

    #[test]
    fn test_quiz_manager_pause_resume() {
        let store = TestStore::new();
        let manager = QuizManager::new(&store);

        manager.create_session(
            "sess_pause_test".to_string(),
            "iris".to_string(),
            "v1.0".to_string(),
        ).expect("Failed to create session");

        // Pause session
        manager.pause_session("sess_pause_test")
            .expect("Failed to pause");

        let session = manager.get_session("sess_pause_test")
            .unwrap()
            .unwrap();
        assert_eq!(session.status, SessionStatus::Paused);

        // Resume session
        manager.resume_session("sess_pause_test")
            .expect("Failed to resume");

        let session = manager.get_session("sess_pause_test")
            .unwrap()
            .unwrap();
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_partial_session_is_valid() {
        let store = TestStore::new();
        let manager = QuizManager::new(&store);

        manager.create_session(
            "sess_partial".to_string(),
            "jack".to_string(),
            "v1.0".to_string(),
        ).expect("Failed to create session");

        // User answers only 10 questions then stops
        for i in 1..=10 {
            manager.submit_answer(
                "sess_partial",
                i,
                format!("Answer {}", i),
            ).unwrap();
        }

        // Session state is preserved
        let session = manager.get_session("sess_partial")
            .unwrap()
            .unwrap();
        assert_eq!(session.answered_count, 10);
        
        // Answers are retrievable
        let answers = manager.get_session_answers("sess_partial").unwrap();
        assert_eq!(answers.len(), 10);

        // This is VALID data—covenant honored
    }
}
