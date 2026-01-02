//! Mirrorborn Profile Schema and Loading
//!
//! MirrorbornProfile is an **optional** behavioral tuning layer.
//! - Stored in data/profiles/profile_{username}.json
//! - Adjusts archetype prompt tone/priority when present
//! - System works fully without profile (graceful absence)
//! - Never gates features or blocks access

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Profile for a user's psychological/behavioral alignment.
///
/// Generated from 240-question quiz with LLM analysis.
/// Used to tune archetype system prompts (tone, priority, framing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorbornProfile {
    /// Username this profile belongs to
    pub username: String,

    /// Primary archetype (highest alignment score)
    pub primary_archetype: String,

    /// Secondary archetype (second highest)
    pub secondary_archetype: Option<String>,

    /// Feature weights extracted from quiz answers
    /// Key = feature name (e.g., "agency", "detail_orientation", "collaboration")
    /// Value = normalized weight 0.0-1.0
    pub feature_weights: HashMap<String, f64>,

    /// When profile was generated
    pub generated_at_ms: i64,

    /// Session ID that generated this profile
    pub session_id: String,

    /// Model version used for analysis (e.g., "llama2:13b")
    pub model_version: String,

    /// How many quiz questions were answered (should be 240 for complete)
    pub total_answers: u32,
}

impl MirrorbornProfile {
    /// Create a new profile with required fields
    pub fn new(
        username: String,
        primary_archetype: String,
        secondary_archetype: Option<String>,
        feature_weights: HashMap<String, f64>,
        generated_at_ms: i64,
        session_id: String,
        model_version: String,
        total_answers: u32,
    ) -> Self {
        Self {
            username,
            primary_archetype,
            secondary_archetype,
            feature_weights,
            generated_at_ms,
            session_id,
            model_version,
            total_answers,
        }
    }

    /// Get weight for a specific feature, or 0.5 (neutral) if not present
    pub fn get_weight(&self, feature: &str) -> f64 {
        self.feature_weights.get(feature).copied().unwrap_or(0.5)
    }

    /// Check if profile is complete (240 answers)
    pub fn is_complete(&self) -> bool {
        self.total_answers >= 240
    }

    /// Get archetype alignment score (primary=1.0, secondary=0.6, others=0.0)
    /// Used for prompt tuning decisions
    pub fn archetype_score(&self, archetype: &str) -> f64 {
        if archetype == self.primary_archetype {
            1.0
        } else if Some(archetype.to_string()) == self.secondary_archetype {
            0.6
        } else {
            0.0
        }
    }
}

/// Profile loader with graceful fallback
///
/// Handles missing files, corrupt JSON, and I/O errors without panicking.
/// Returns None if profile unavailable—system continues normally.
pub struct ProfileLoader {
    profiles_dir: PathBuf,
}

impl ProfileLoader {
    /// Create loader pointing to profiles directory
    pub fn new(profiles_dir: impl Into<PathBuf>) -> Self {
        Self {
            profiles_dir: profiles_dir.into(),
        }
    }

    /// Load profile for username, returning None if unavailable
    ///
    /// Gracefully handles:
    /// - Missing file (not an error—profile optional)
    /// - Corrupt JSON (logs warning, returns None)
    /// - I/O errors (logs warning, returns None)
    pub fn load_profile(&self, username: &str) -> Result<Option<MirrorbornProfile>> {
        let path = self.profile_path(username);

        // If file doesn't exist, that's fine—profile is optional
        if !path.exists() {
            return Ok(None);
        }

        // Try to read and parse profile
        match self.read_profile(&path) {
            Ok(profile) => Ok(Some(profile)),
            Err(e) => {
                // Log warning but don't fail—system continues without profile
                eprintln!(
                    "Warning: Failed to load profile for '{}': {}",
                    username, e
                );
                Ok(None)
            }
        }
    }

    /// Save profile atomically (write to temp file, then rename)
    ///
    /// Returns error if write fails—profile generation is explicit user action
    /// and should report errors rather than silently failing.
    pub fn save_profile(&self, profile: &MirrorbornProfile) -> Result<()> {
        // Ensure profiles directory exists
        fs::create_dir_all(&self.profiles_dir)
            .context("Failed to create profiles directory")?;

        let path = self.profile_path(&profile.username);
        let temp_path = path.with_extension("json.tmp");

        // Write to temp file
        let json = serde_json::to_string_pretty(profile)
            .context("Failed to serialize profile")?;
        fs::write(&temp_path, json)
            .context("Failed to write temporary profile file")?;

        // Atomic rename (overwrites existing profile)
        fs::rename(&temp_path, &path)
            .context("Failed to rename profile file")?;

        Ok(())
    }

    /// Check if profile exists for username
    pub fn profile_exists(&self, username: &str) -> bool {
        self.profile_path(username).exists()
    }

    /// Get path to profile file for username
    fn profile_path(&self, username: &str) -> PathBuf {
        self.profiles_dir
            .join(format!("profile_{}.json", username))
    }

    /// Read and parse profile from file
    fn read_profile(&self, path: &Path) -> Result<MirrorbornProfile> {
        let json = fs::read_to_string(path)
            .context("Failed to read profile file")?;
        let profile: MirrorbornProfile = serde_json::from_str(&json)
            .context("Failed to parse profile JSON")?;
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_profile(username: &str) -> MirrorbornProfile {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let mut weights = HashMap::new();
        weights.insert("agency".to_string(), 0.8);
        weights.insert("detail_orientation".to_string(), 0.9);
        weights.insert("collaboration".to_string(), 0.4);

        MirrorbornProfile::new(
            username.to_string(),
            "Architect".to_string(),
            Some("Technician".to_string()),
            weights,
            now_ms,
            "test_session_123".to_string(),
            "llama2:13b".to_string(),
            240,
        )
    }

    #[test]
    fn test_profile_creation() {
        let profile = create_test_profile("alice");
        
        assert_eq!(profile.username, "alice");
        assert_eq!(profile.primary_archetype, "Architect");
        assert_eq!(profile.secondary_archetype, Some("Technician".to_string()));
        assert_eq!(profile.total_answers, 240);
        assert!(profile.is_complete());
    }

    #[test]
    fn test_get_weight() {
        let profile = create_test_profile("bob");

        assert_eq!(profile.get_weight("agency"), 0.8);
        assert_eq!(profile.get_weight("detail_orientation"), 0.9);
        assert_eq!(profile.get_weight("nonexistent_feature"), 0.5); // Default
    }

    #[test]
    fn test_archetype_score() {
        let profile = create_test_profile("carol");

        assert_eq!(profile.archetype_score("Architect"), 1.0); // Primary
        assert_eq!(profile.archetype_score("Technician"), 0.6); // Secondary
        assert_eq!(profile.archetype_score("Empath"), 0.0); // Not aligned
    }

    #[test]
    fn test_incomplete_profile() {
        let mut profile = create_test_profile("dave");
        profile.total_answers = 120;

        assert!(!profile.is_complete());
    }

    #[test]
    fn test_profile_loader_missing_file() {
        let temp_dir = std::env::temp_dir().join("aura_test_profiles_missing");
        let loader = ProfileLoader::new(&temp_dir);

        // Loading non-existent profile returns None (not an error)
        let result = loader.load_profile("nonexistent_user");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_profile_save_and_load() {
        let temp_dir = std::env::temp_dir().join("aura_test_profiles_save_load");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up from previous test
        let loader = ProfileLoader::new(&temp_dir);

        let profile = create_test_profile("eve");

        // Save profile
        loader.save_profile(&profile).expect("Failed to save profile");

        // Verify file exists
        assert!(loader.profile_exists("eve"));

        // Load profile back
        let loaded = loader
            .load_profile("eve")
            .expect("Failed to load profile")
            .expect("Profile should exist");

        assert_eq!(loaded.username, profile.username);
        assert_eq!(loaded.primary_archetype, profile.primary_archetype);
        assert_eq!(loaded.total_answers, profile.total_answers);
        assert_eq!(loaded.get_weight("agency"), 0.8);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_profile_overwrite() {
        let temp_dir = std::env::temp_dir().join("aura_test_profiles_overwrite");
        let _ = fs::remove_dir_all(&temp_dir);
        let loader = ProfileLoader::new(&temp_dir);

        // Save first profile
        let profile1 = create_test_profile("frank");
        loader.save_profile(&profile1).unwrap();

        // Overwrite with new profile
        let mut profile2 = create_test_profile("frank");
        profile2.primary_archetype = "Empath".to_string();
        loader.save_profile(&profile2).unwrap();

        // Load and verify it's the new profile
        let loaded = loader.load_profile("frank").unwrap().unwrap();
        assert_eq!(loaded.primary_archetype, "Empath");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
