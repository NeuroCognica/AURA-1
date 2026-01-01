//! Prompt loading and management
//!
//! This module treats prompts as **authoritative artifacts**.
//! Missing or unreadable prompts cause hard failures (fail-closed).
//!
//! Design principles:
//! - No fallbacks
//! - No defaults
//! - No inline strings
//! - No caching (yet)
//! - Deterministic failures
//!
//! If an archetype's constitutional prompt is missing, the system MUST NOT
//! attempt to operate that archetype.

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during prompt loading
#[derive(Error, Debug)]
pub enum PromptError {
    #[error("Prompt file not found: {path}")]
    NotFound { path: String },

    #[error("Failed to read prompt file {path}: {source}")]
    ReadError { path: String, source: std::io::Error },

    #[error("Prompt file is empty: {path}")]
    Empty { path: String },

    #[error("No system prompt defined for archetype: {archetype}")]
    NoPromptForArchetype { archetype: String },

    #[error("Invalid prompt directory: {path}")]
    InvalidDirectory { path: String },
}

/// Types of prompts that can be loaded
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    /// System-level constitutional prompt (e.g., sentinel_system.txt)
    System,
}

/// Archetype identifier for prompt loading
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchetypeId {
    Sentinel,
    Architect,
    Explorer,
    Oracle,
    Mentor,
    Empath,
    Jester,
}

impl ArchetypeId {
    /// Get the lowercase name used in file paths
    fn as_str(&self) -> &str {
        match self {
            ArchetypeId::Sentinel => "sentinel",
            ArchetypeId::Architect => "architect",
            ArchetypeId::Explorer => "explorer",
            ArchetypeId::Oracle => "oracle",
            ArchetypeId::Mentor => "mentor",
            ArchetypeId::Empath => "empath",
            ArchetypeId::Jester => "jester",
        }
    }
}

/// Prompt loader with configurable base directory
pub struct PromptLoader {
    base_dir: PathBuf,
}

impl PromptLoader {
    /// Create a new prompt loader with the given base directory
    ///
    /// The base directory should contain prompt files like:
    /// - sentinel_system.txt
    /// - architect_system.txt
    /// - etc.
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self, PromptError> {
        let base_dir = base_dir.as_ref().to_path_buf();
        
        if !base_dir.exists() {
            return Err(PromptError::InvalidDirectory {
                path: base_dir.display().to_string(),
            });
        }

        Ok(Self { base_dir })
    }

    /// Load a system prompt for the given archetype
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The prompt file does not exist
    /// - The file cannot be read
    /// - The file is empty
    ///
    /// # Fail-Closed Behavior
    ///
    /// This function intentionally fails hard rather than falling back to
    /// defaults. Missing constitutional prompts are a system integrity failure.
    pub fn load_system_prompt(&self, archetype: ArchetypeId) -> Result<String, PromptError> {
        let filename = format!("{}_system.txt", archetype.as_str());
        let path = self.base_dir.join(&filename);

        // Fail if file doesn't exist
        if !path.exists() {
            return Err(PromptError::NotFound {
                path: path.display().to_string(),
            });
        }

        // Fail if file can't be read
        let content = fs::read_to_string(&path).map_err(|e| PromptError::ReadError {
            path: path.display().to_string(),
            source: e,
        })?;

        // Fail if file is empty
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(PromptError::Empty {
                path: path.display().to_string(),
            });
        }

        Ok(trimmed.to_string())
    }

    /// Check if a system prompt exists for the given archetype
    pub fn has_system_prompt(&self, archetype: &ArchetypeId) -> bool {
        let filename = format!("{}_system.txt", archetype.as_str());
        let path = self.base_dir.join(&filename);
        path.exists()
    }

    /// Get the base directory for prompts
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

/// Load a system prompt using the default prompt directory
///
/// This is a convenience function that assumes prompts are in
/// `backend/orchestrator/src/prompts/` relative to the workspace root.
///
/// For production use, prefer creating a `PromptLoader` with an explicit path.
pub fn load_system_prompt(archetype: ArchetypeId) -> Result<String, PromptError> {
    // Determine the prompt directory relative to this source file
    // In dev: backend/orchestrator/src/prompts/mod.rs
    // Prompts are in: backend/orchestrator/src/prompts/
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let base_dir = PathBuf::from(manifest_dir).join("src/prompts");
    
    let loader = PromptLoader::new(base_dir)?;
    loader.load_system_prompt(archetype)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_prompt(dir: &Path, archetype: &str, content: &str) {
        let path = dir.join(format!("{}_system.txt", archetype));
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_load_existing_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let prompt_content = "You are Sentinel. Deny everything.";
        create_test_prompt(temp_dir.path(), "sentinel", prompt_content);

        let loader = PromptLoader::new(temp_dir.path()).unwrap();
        let loaded = loader.load_system_prompt(ArchetypeId::Sentinel).unwrap();

        assert_eq!(loaded, prompt_content);
    }

    #[test]
    fn test_load_missing_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let loader = PromptLoader::new(temp_dir.path()).unwrap();

        let result = loader.load_system_prompt(ArchetypeId::Sentinel);
        assert!(matches!(result, Err(PromptError::NotFound { .. })));
    }

    #[test]
    fn test_empty_prompt_fails() {
        let temp_dir = TempDir::new().unwrap();
        create_test_prompt(temp_dir.path(), "sentinel", "");

        let loader = PromptLoader::new(temp_dir.path()).unwrap();
        let result = loader.load_system_prompt(ArchetypeId::Sentinel);

        assert!(matches!(result, Err(PromptError::Empty { .. })));
    }

    #[test]
    fn test_whitespace_only_prompt_fails() {
        let temp_dir = TempDir::new().unwrap();
        create_test_prompt(temp_dir.path(), "sentinel", "   \n\t  \n  ");

        let loader = PromptLoader::new(temp_dir.path()).unwrap();
        let result = loader.load_system_prompt(ArchetypeId::Sentinel);

        assert!(matches!(result, Err(PromptError::Empty { .. })));
    }

    #[test]
    fn test_has_system_prompt() {
        let temp_dir = TempDir::new().unwrap();
        create_test_prompt(temp_dir.path(), "sentinel", "Test prompt");

        let loader = PromptLoader::new(temp_dir.path()).unwrap();

        assert!(loader.has_system_prompt(&ArchetypeId::Sentinel));
        assert!(!loader.has_system_prompt(&ArchetypeId::Architect));
    }

    #[test]
    fn test_invalid_directory() {
        let result = PromptLoader::new("/this/path/does/not/exist");
        assert!(matches!(result, Err(PromptError::InvalidDirectory { .. })));
    }

    #[test]
    fn test_archetype_id_as_str() {
        assert_eq!(ArchetypeId::Sentinel.as_str(), "sentinel");
        assert_eq!(ArchetypeId::Architect.as_str(), "architect");
        assert_eq!(ArchetypeId::Explorer.as_str(), "explorer");
        assert_eq!(ArchetypeId::Oracle.as_str(), "oracle");
        assert_eq!(ArchetypeId::Mentor.as_str(), "mentor");
        assert_eq!(ArchetypeId::Empath.as_str(), "empath");
        assert_eq!(ArchetypeId::Jester.as_str(), "jester");
    }

    #[test]
    fn test_prompt_trimming() {
        let temp_dir = TempDir::new().unwrap();
        let prompt_with_whitespace = "\n\n  You are Sentinel.  \n\n";
        create_test_prompt(temp_dir.path(), "sentinel", prompt_with_whitespace);

        let loader = PromptLoader::new(temp_dir.path()).unwrap();
        let loaded = loader.load_system_prompt(ArchetypeId::Sentinel).unwrap();

        assert_eq!(loaded, "You are Sentinel.");
    }
}
