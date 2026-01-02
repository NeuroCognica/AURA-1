//! Quiz loader utility - loads Quiz from JSON or parses from master_quiz.md

use crate::quiz::Quiz;
use crate::quiz_parser::parse_full_quiz;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Load quiz from JSON file (fast path)
///
/// If the JSON file doesn't exist, falls back to parsing master_quiz.md
pub fn load_quiz_json(json_path: &Path) -> Result<Quiz> {
    let content = fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read quiz JSON: {:?}", json_path))?;

    let quiz: Quiz = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse quiz JSON: {:?}", json_path))?;

    Ok(quiz)
}

/// Parse quiz from master_quiz.md and save as JSON
///
/// This is the slow path used for initial generation or updates
pub fn generate_quiz_json(
    master_quiz_path: &Path,
    json_output_path: &Path,
    version: String,
) -> Result<Quiz> {
    let content = fs::read_to_string(master_quiz_path)
        .with_context(|| format!("Failed to read master quiz: {:?}", master_quiz_path))?;

    let result = parse_full_quiz(&content, version)
        .context("Failed to parse master quiz")?;

    // Write warnings to stderr if present
    if !result.warnings.is_empty() {
        eprintln!("⚠ Quiz parser warnings:");
        for warning in &result.warnings {
            eprintln!("  - {}", warning);
        }
    }

    // Serialize to JSON with pretty formatting
    let json = serde_json::to_string_pretty(&result.quiz)
        .context("Failed to serialize quiz to JSON")?;

    // Ensure parent directory exists
    if let Some(parent) = json_output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    fs::write(json_output_path, json)
        .with_context(|| format!("Failed to write quiz JSON: {:?}", json_output_path))?;

    eprintln!("✓ Generated quiz JSON: {:?}", json_output_path);
    eprintln!("  {} probes", result.quiz.probes.len());

    Ok(result.quiz)
}

/// Load quiz with automatic fallback:
/// 1. Try to load from JSON (fast)
/// 2. If missing, parse from master_quiz.md and generate JSON
///
/// This provides the best of both worlds: fast startup with automatic
/// regeneration when needed.
pub fn load_quiz_with_fallback(
    json_path: &Path,
    master_quiz_path: &Path,
    version: String,
) -> Result<Quiz> {
    match load_quiz_json(json_path) {
        Ok(quiz) => {
            eprintln!("✓ Loaded quiz from JSON cache: {:?}", json_path);
            Ok(quiz)
        }
        Err(_) => {
            eprintln!("⚠ Quiz JSON not found, parsing from source...");
            generate_quiz_json(master_quiz_path, json_path, version)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_generate_and_load_quiz_json() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("quiz.json");
        let master_quiz_path = PathBuf::from("../mirrorborn_quiz/master_quiz.md");

        // Generate JSON
        let quiz1 = generate_quiz_json(&master_quiz_path, &json_path, "1.0.0".to_string())
            .expect("Failed to generate quiz JSON");

        assert_eq!(quiz1.probes.len(), 240);
        assert!(json_path.exists());

        // Load from JSON
        let quiz2 = load_quiz_json(&json_path)
            .expect("Failed to load quiz JSON");

        assert_eq!(quiz2.probes.len(), 240);
        assert_eq!(quiz1.version, quiz2.version);

        // Verify probe data preserved
        assert_eq!(quiz1.probes[0].id, quiz2.probes[0].id);
        assert_eq!(quiz1.probes[0].text, quiz2.probes[0].text);
        assert_eq!(quiz1.probes[239].id, quiz2.probes[239].id);
    }

    #[test]
    fn test_load_quiz_with_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("quiz_fallback.json");
        let master_quiz_path = PathBuf::from("../mirrorborn_quiz/master_quiz.md");

        // First call: JSON doesn't exist, should parse and generate
        let quiz1 = load_quiz_with_fallback(&json_path, &master_quiz_path, "1.0.0".to_string())
            .expect("Failed to load quiz with fallback");

        assert_eq!(quiz1.probes.len(), 240);
        assert!(json_path.exists());

        // Second call: JSON exists, should load from cache
        let quiz2 = load_quiz_with_fallback(&json_path, &master_quiz_path, "1.0.0".to_string())
            .expect("Failed to load quiz from cache");

        assert_eq!(quiz2.probes.len(), 240);
        assert_eq!(quiz1.probes[0].id, quiz2.probes[0].id);
    }
}
