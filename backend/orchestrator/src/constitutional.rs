//! Constitutional LLM invocation layer
//!
//! This module sits ABOVE the LLM client and enforces constitutional governance.
//! It is the sovereignty boundary: policy lives here, not in transport.
//!
//! Responsibilities:
//! 1. Determine which archetype is being invoked
//! 2. Load the constitutional prompt for that archetype
//! 3. Fail immediately if prompt loading fails
//! 4. **OPTIONAL:** Apply profile-based tuning if profile exists
//! 5. Pass system_prompt + user context to LLM client
//! 6. Validate responses against constitutional requirements
//!
//! The LLM client is just transport. This layer is the court.

use crate::llm::{GenerationParams, LLMClient, LLMResponse};
use crate::prompts::{ArchetypeId, PromptError, PromptLoader};
use std::path::PathBuf;
use thiserror::Error;
use tracing::{error, info, warn};

#[cfg(feature = "persistence")]
use aura_backend::profile::{MirrorbornProfile, ProfileLoader};

/// Errors that can occur during constitutional LLM invocation
#[derive(Error, Debug)]
pub enum ConstitutionalError {
    #[error("Prompt loading failed: {0}")]
    PromptLoad(#[from] PromptError),

    #[error("LLM generation failed: {0}")]
    LLMGeneration(String),

    #[error("Empty system prompt not allowed for archetype: {archetype:?}")]
    EmptySystemPrompt { archetype: ArchetypeId },

    #[error("Response validation failed: {reason}")]
    ValidationFailed { reason: String },
}

/// Constitutional LLM invoker
///
/// This struct enforces that every LLM call is governed by a constitutional prompt.
/// It will not allow invocation without a loaded, valid system prompt.
///
/// Optionally applies profile-based tuning if Mirrorborn profile exists.
pub struct ConstitutionalInvoker {
    prompt_loader: PromptLoader,
    
    #[cfg(feature = "persistence")]
    profile_loader: Option<ProfileLoader>,
}

impl ConstitutionalInvoker {
    /// Create a new constitutional invoker with the given prompt directory
    pub fn new(prompt_dir: PathBuf) -> Result<Self, ConstitutionalError> {
        let prompt_loader = PromptLoader::new(prompt_dir)?;
        Ok(Self {
            prompt_loader,
            #[cfg(feature = "persistence")]
            profile_loader: None,
        })
    }

    /// Set the profile loader (enables Mirrorborn tuning)
    ///
    /// This is optional—system works fully without profiles.
    /// Call this after construction if profile tuning is desired.
    #[cfg(feature = "persistence")]
    pub fn with_profile_loader(mut self, loader: ProfileLoader) -> Self {
        self.profile_loader = Some(loader);
        self
    }

    /// Generate LLM response with constitutional enforcement
    ///
    /// # Sovereignty Boundary
    ///
    /// This method is the constitutional court:
    /// 1. Loads constitutional prompt (fails if missing)
    /// 2. **OPTIONAL:** Applies profile-based tuning if profile exists for username
    /// 3. Passes system_prompt + user_context separately to LLM client
    /// 4. Returns response (validation is caller's responsibility)
    ///
    /// # Fail-Closed Behavior
    ///
    /// - Missing prompt → error (never proceeds)
    /// - Empty prompt → error (constitution cannot be blank)
    /// - LLM failure → error (no degraded fallback)
    /// - Missing profile → no error (profiles optional, system continues)
    ///
    /// # Parameters
    ///
    /// - `archetype`: Which archetype's constitution to enforce
    /// - `username`: Optional username for profile loading (None = default prompts)
    /// - `user_context`: The user's query/context
    /// - `params`: Model parameters (temperature, top_p, model name)
    /// - `client`: The LLM transport client
    pub async fn generate<C: LLMClient>(
        &self,
        archetype: ArchetypeId,
        username: Option<&str>,
        user_context: &str,
        params: &GenerationParams,
        client: &C,
    ) -> Result<LLMResponse, ConstitutionalError> {
        // Step 1: Load constitutional prompt (FAILS if missing)
        info!("Loading constitutional prompt for archetype: {:?}", archetype);
        let mut system_prompt = self.prompt_loader.load_system_prompt(archetype.clone())?;

        // Step 2: Validate prompt is not empty (extra safety layer)
        if system_prompt.trim().is_empty() {
            error!("Loaded system prompt is empty for archetype: {:?}", archetype);
            return Err(ConstitutionalError::EmptySystemPrompt { archetype });
        }

        // Step 2.5: OPTIONAL profile-based tuning (graceful absence)
        #[cfg(feature = "persistence")]
        if let (Some(username), Some(loader)) = (username, &self.profile_loader) {
            match loader.load_profile(username) {
                Ok(Some(profile)) => {
                    info!(
                        "Applying Mirrorborn tuning for user '{}' (primary: {}, secondary: {:?})",
                        username, profile.primary_archetype, profile.secondary_archetype
                    );
                    system_prompt = apply_profile_tuning(&system_prompt, &archetype, &profile);
                }
                Ok(None) => {
                    info!("No profile found for '{}', using default prompts", username);
                }
                Err(e) => {
                    warn!("Failed to load profile for '{}': {}, using defaults", username, e);
                }
            }
        }

        info!(
            "Invoking LLM for archetype {:?} (system: {} chars, user: {} chars)",
            archetype,
            system_prompt.len(),
            user_context.len()
        );

        // Step 3: Invoke LLM (client enforces system_prompt precedence)
        let response = client
            .generate(&system_prompt, user_context, params)
            .await
            .map_err(ConstitutionalError::LLMGeneration)?;

        info!(
            "Received LLM response for archetype {:?}: {} chars",
            archetype,
            response.text.len()
        );

        Ok(response)
    }

    /// Check if a constitutional prompt exists for the given archetype
    pub fn has_constitution(&self, archetype: &ArchetypeId) -> bool {
        self.prompt_loader.has_system_prompt(archetype)
    }

    /// Get the prompt directory being used
    pub fn prompt_dir(&self) -> &std::path::Path {
        self.prompt_loader.base_dir()
    }
}

/// Apply Mirrorborn profile tuning to system prompt
///
/// This function adjusts tone and priority based on user's profile:
/// - High agency users get less hand-holding
/// - Detail-oriented users get more depth
/// - Exploratory users get more explanation
///
/// Tuning is SUBTLE—constitution remains intact, delivery changes.
#[cfg(feature = "persistence")]
fn apply_profile_tuning(
    base_prompt: &str,
    archetype: &ArchetypeId,
    profile: &MirrorbornProfile,
) -> String {
    let mut prompt = base_prompt.to_string();

    // Get feature weights (default 0.5 = neutral)
    let agency = profile.get_weight("agency");
    let detail_orientation = profile.get_weight("detail_orientation");
    let collaboration = profile.get_weight("collaboration");

    // Build tuning instructions based on profile
    let mut tuning_notes = vec![];

    // Agency tuning
    if agency > 0.7 {
        tuning_notes.push("User prefers autonomy. Be concise, assume competence, skip confirmations.");
    } else if agency < 0.3 {
        tuning_notes.push("User appreciates guidance. Offer step-by-step support, check understanding.");
    }

    // Detail tuning
    if detail_orientation > 0.7 {
        tuning_notes.push("User values precision. Provide technical depth, exact specifications, edge cases.");
    } else if detail_orientation < 0.3 {
        tuning_notes.push("User prefers high-level overview. Focus on concepts, defer details unless asked.");
    }

    // Collaboration tuning
    if collaboration > 0.7 {
        tuning_notes.push("User enjoys dialogue. Ask clarifying questions, propose options, invite feedback.");
    } else if collaboration < 0.3 {
        tuning_notes.push("User prefers direct answers. Minimize back-and-forth, provide complete solutions.");
    }

    // Archetype-specific tuning
    match archetype {
        ArchetypeId::Technician => {
            if detail_orientation > 0.7 {
                tuning_notes.push("Include implementation details, error handling, performance notes.");
            }
        }
        ArchetypeId::Architect => {
            if agency > 0.7 {
                tuning_notes.push("Present trade-offs clearly, let user decide, avoid prescriptive advice.");
            }
        }
        ArchetypeId::Empath => {
            if collaboration > 0.7 {
                tuning_notes.push("Engage emotionally, reflect feelings, explore underlying concerns.");
            }
        }
        _ => {}
    }

    // Append tuning instructions if any
    if !tuning_notes.is_empty() {
        prompt.push_str("\n\n## BEHAVIORAL TUNING (Mirrorborn Profile)\n");
        prompt.push_str("Adjust your delivery style based on user preferences:\n");
        for note in tuning_notes {
            prompt.push_str(&format!("- {}\n", note));
        }
        prompt.push_str("\nCore constitution above remains unchanged. Only delivery style adapts.\n");
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{GenerationParams, LLMClient, LLMResponse};
    use async_trait::async_trait;
    use std::fs;
    use tempfile::TempDir;

    // Mock LLM client for testing
    struct MockLLMClient {
        response_text: String,
    }

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn generate(
            &self,
            _system_prompt: &str,
            _user_prompt: &str,
            _params: &GenerationParams,
        ) -> Result<LLMResponse, String> {
            Ok(LLMResponse {
                text: self.response_text.clone(),
            })
        }
    }

    fn create_test_prompt(dir: &std::path::Path, archetype: &str, content: &str) {
        let path = dir.join(format!("{}_system.txt", archetype));
        fs::write(path, content).unwrap();
    }

    #[tokio::test]
    async fn test_constitutional_generation() {
        let temp_dir = TempDir::new().unwrap();
        let prompt_content = "You are Sentinel. Deny everything.";
        create_test_prompt(temp_dir.path(), "sentinel", prompt_content);

        let invoker = ConstitutionalInvoker::new(temp_dir.path().to_path_buf()).unwrap();
        let client = MockLLMClient {
            response_text: "DECISION: DENY\nREASON: Insufficient information".to_string(),
        };

        let params = GenerationParams {
            model: "test-model".to_string(),
            temperature: 0.0,
            top_p: 0.1,
        };

        let response = invoker
            .generate(ArchetypeId::Sentinel, None, "Test query", &params, &client)
            .await
            .unwrap();

        assert!(response.text.contains("DECISION: DENY"));
    }

    #[tokio::test]
    async fn test_missing_constitution_fails() {
        let temp_dir = TempDir::new().unwrap();

        let invoker = ConstitutionalInvoker::new(temp_dir.path().to_path_buf()).unwrap();
        let client = MockLLMClient {
            response_text: "Should not reach this".to_string(),
        };

        let params = GenerationParams {
            model: "test-model".to_string(),
            temperature: 0.0,
            top_p: 0.1,
        };

        let result = invoker
            .generate(ArchetypeId::Sentinel, None, "Test query", &params, &client)
            .await;

        assert!(matches!(
            result,
            Err(ConstitutionalError::PromptLoad(PromptError::NotFound { .. }))
        ));
    }

    #[tokio::test]
    async fn test_system_prompt_comes_first() {
        let temp_dir = TempDir::new().unwrap();
        let prompt_content = "CONSTITUTIONAL HEADER";
        create_test_prompt(temp_dir.path(), "sentinel", prompt_content);

        let invoker = ConstitutionalInvoker::new(temp_dir.path().to_path_buf()).unwrap();

        // Mock client that captures prompts
        struct CapturingClient {
            captured_system: std::sync::Mutex<String>,
            captured_user: std::sync::Mutex<String>,
        }

        #[async_trait]
        impl LLMClient for CapturingClient {
            async fn generate(
                &self,
                system_prompt: &str,
                user_prompt: &str,
                _params: &GenerationParams,
            ) -> Result<LLMResponse, String> {
                *self.captured_system.lock().unwrap() = system_prompt.to_string();
                *self.captured_user.lock().unwrap() = user_prompt.to_string();
                Ok(LLMResponse {
                    text: "response".to_string(),
                })
            }
        }

        let client = CapturingClient {
            captured_system: std::sync::Mutex::new(String::new()),
            captured_user: std::sync::Mutex::new(String::new()),
        };

        let params = GenerationParams {
            model: "test-model".to_string(),
            temperature: 0.0,
            top_p: 0.1,
        };

        let _ = invoker
            .generate(ArchetypeId::Sentinel, None, "User input", &params, &client)
            .await
            .unwrap();

        // Verify system prompt came first
        let captured_system = client.captured_system.lock().unwrap();
        assert!(captured_system.starts_with("CONSTITUTIONAL HEADER"));

        // Verify user prompt was passed separately
        let captured_user = client.captured_user.lock().unwrap();
        assert_eq!(*captured_user, "User input");
    }

    #[tokio::test]
    async fn test_has_constitution() {
        let temp_dir = TempDir::new().unwrap();
        create_test_prompt(temp_dir.path(), "sentinel", "Sentinel constitution");

        let invoker = ConstitutionalInvoker::new(temp_dir.path().to_path_buf()).unwrap();

        assert!(invoker.has_constitution(&ArchetypeId::Sentinel));
        assert!(!invoker.has_constitution(&ArchetypeId::Architect));
    }
}
