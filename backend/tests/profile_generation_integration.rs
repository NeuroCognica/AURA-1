//! Integration test: Full quiz → profile generation flow
//!
//! Validates end-to-end covenant-keeping:
//! 1. Create quiz session
//! 2. Submit 240 answers (append-only)
//! 3. Generate profile (atomic write)
//! 4. Verify profile structure and archetype assignment

use aura_backend::quiz::{QuizManager, SessionStatus};
use aura_backend::profile_synthesis::{synthesize_profile, SynthesisConfig};
use aura_backend::storage::Storage;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Test storage implementation
struct TestStore {
    data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    log_counter: Arc<Mutex<u64>>,
}

impl TestStore {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            log_counter: Arc::new(Mutex::new(0)),
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

#[tokio::test]
async fn test_full_quiz_to_profile_flow() -> Result<()> {
    let store = Arc::new(TestStore::new());
    let manager = QuizManager::new(&*store);

    // 1. Create session
    let session = manager.create_session(
        "sess_alice_123".to_string(),
        "alice".to_string(),
        "v1.0".to_string()
    )?;
    let session_id = session.session_id.clone();
    
    // 2. Submit 240 answers with varying psychological patterns
    // First 120: High agency, high detail (Architect pattern)
    for probe_id in 1..=120 {
        manager.submit_answer(
            &session_id,
            probe_id,
            "I chose to do this after careful analysis of all the details.".to_string(),
        )?;
    }
    
    // Next 60: High collaboration, high emotion (Empath pattern)
    for probe_id in 121..=180 {
        manager.submit_answer(
            &session_id,
            probe_id,
            "I feel this is important. What do you think?".to_string(),
        )?;
    }
    
    // Next 60: High agency, low detail (Explorer pattern)
    for probe_id in 181..=240 {
        manager.submit_answer(
            &session_id,
            probe_id,
            "I decided to take action immediately.".to_string(),
        )?;
    }
    
    // 3. Verify session is complete
    let session = manager.get_session(&session_id)?.unwrap();
    assert_eq!(session.status, SessionStatus::Complete);
    assert_eq!(session.answered_count, 240);
    
    // 4. Get all answers
    let answers = manager.get_session_answers(&session_id)?;
    assert_eq!(answers.len(), 240);
    
    // 5. Synthesize profile
    let config = SynthesisConfig::default();
    let profile = synthesize_profile(&session, &answers, &config).await?;
    
    // 6. Validate profile structure
    assert_eq!(profile.username, "alice");
    assert_eq!(profile.total_answers, 240);
    assert!(!profile.primary_archetype.is_empty());
    assert!(profile.generated_at_ms > 0);
    assert_eq!(profile.session_id, session_id);
    
    // 7. Validate feature weights are normalized (0.0-1.0)
    for (_feature, weight) in &profile.feature_weights {
        assert!(*weight >= 0.0 && *weight <= 1.0);
    }
    
    // 8. Validate primary archetype is one of the 7
    let valid_archetypes = [
        "Architect", "Empath", "Explorer", "Mentor", 
        "Jester", "Technician", "Sentinel"
    ];
    assert!(valid_archetypes.contains(&profile.primary_archetype.as_str()));
    
    // 9. If secondary archetype exists, validate it's different from primary
    if let Some(ref secondary) = profile.secondary_archetype {
        assert_ne!(secondary, &profile.primary_archetype);
        assert!(valid_archetypes.contains(&secondary.as_str()));
    }
    
    println!("Profile generated: {} (secondary: {:?})", 
             profile.primary_archetype, profile.secondary_archetype);
    println!("Feature weights: {:?}", profile.feature_weights);
    
    Ok(())
}

#[tokio::test]
async fn test_profile_generation_requires_complete_session() -> Result<()> {
    let store = Arc::new(TestStore::new());
    let manager = QuizManager::new(&*store);

    // Create session and answer only 10 questions
    let session = manager.create_session(
        "sess_bob_456".to_string(),
        "bob".to_string(),
        "v1.0".to_string()
    )?;
    let session_id = session.session_id.clone();
    
    for probe_id in 1..=10 {
        manager.submit_answer(
            &session_id,
            probe_id,
            "Partial answer".to_string(),
        )?;
    }
    
    // Session should not be Complete
    let session = manager.get_session(&session_id)?.unwrap();
    assert_ne!(session.status, SessionStatus::Complete);
    
    // Profile synthesis should fail
    let answers = manager.get_session_answers(&session_id)?;
    let config = SynthesisConfig::default();
    let result = synthesize_profile(&session, &answers, &config).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not complete"));
    
    Ok(())
}

#[tokio::test]
async fn test_profile_generation_covenant_tracking() -> Result<()> {
    let store = Arc::new(TestStore::new());
    let manager = QuizManager::new(&*store);

    // Create session
    let session = manager.create_session(
        "sess_charlie_789".to_string(),
        "charlie".to_string(),
        "v1.0".to_string()
    )?;
    let session_id = session.session_id.clone();
    
    // Submit 240 answers
    for probe_id in 1..=240 {
        manager.submit_answer(
            &session_id,
            probe_id,
            format!("Answer to probe {}", probe_id),
        )?;
    }
    
    // Verify all answers preserved (Forever Law)
    let answers = manager.get_session_answers(&session_id)?;
    assert_eq!(answers.len(), 240);
    
    // Verify provenance on each answer
    for (i, answer) in answers.iter().enumerate() {
        assert_eq!(answer.username, "charlie");
        assert_eq!(answer.session_id, session_id);
        assert_eq!(answer.probe_id, (i + 1) as u32);
        assert!(answer.answered_at_ms > 0);
        assert!(!answer.answer_id.is_empty());
    }
    
    Ok(())
}
