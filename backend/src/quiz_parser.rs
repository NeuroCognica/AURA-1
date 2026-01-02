//! Quiz parser for master_quiz.md
//!
//! Parses the 240-probe Mirrorborn quiz in two phases:
//! Phase 1: Extract 80 main questions with metadata
//! Phase 2: Add 3 sub-probes per question (160 probes total)
//!
//! Format:
//! ```markdown
//! Q1 — Present Moment Anchoring
//!
//! Primary:
//! [Main question text]
//!
//! Probe A — Concrete Recall
//! [Sub-question text]
//! → [Features]
//! → [Archetypes]
//! ```

use crate::quiz::{Probe, Quiz};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;

/// Parse result from Phase 1 (main questions only)
#[derive(Debug)]
pub struct ParsedMainQuestions {
    pub questions: Vec<MainQuestion>,
    pub total_count: usize,
}

/// A main question before probes are added
#[derive(Debug, Clone)]
pub struct MainQuestion {
    pub number: u32,
    pub title: String,
    pub primary_text: String,
}

/// Parse result from Phase 2 (with all probes)
#[derive(Debug)]
pub struct ParsedQuiz {
    pub quiz: Quiz,
    pub warnings: Vec<String>,
}

/// Parse the quiz file in Phase 1: extract main questions only
///
/// Returns 80 main questions with titles and primary text.
/// Does not parse sub-probes yet for efficiency.
pub fn parse_main_questions(content: &str) -> Result<ParsedMainQuestions> {
    let mut questions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for question header: "Q1 — Present Moment Anchoring"
        if let Some(header) = parse_question_header(line) {
            let (number, title) = header;

            // Find "Primary:" section
            let mut primary_text = String::new();
            let mut j = i + 1;
            let mut found_primary = false;

            while j < lines.len() {
                let current = lines[j].trim();

                if current == "Primary:" {
                    found_primary = true;
                    j += 1;
                    break;
                }

                // Stop if we hit next question
                if current.starts_with("Q") && current.contains("—") {
                    break;
                }

                j += 1;
            }

            if !found_primary {
                return Err(anyhow!(
                    "Question {} '{}' missing 'Primary:' section",
                    number,
                    title
                ));
            }

            // Extract primary question text (until "Probe A" or next "Q")
            while j < lines.len() {
                let current = lines[j].trim();

                // Stop at probe or next question
                if current.starts_with("Probe ") || (current.starts_with("Q") && current.contains("—")) {
                    break;
                }

                // Skip empty lines at start
                if !primary_text.is_empty() || !current.is_empty() {
                    if !primary_text.is_empty() && !current.is_empty() {
                        primary_text.push(' ');
                    }
                    primary_text.push_str(current);
                }

                j += 1;
            }

            if primary_text.is_empty() {
                return Err(anyhow!(
                    "Question {} '{}' has empty primary text",
                    number,
                    title
                ));
            }

            questions.push(MainQuestion {
                number,
                title,
                primary_text: primary_text.trim().to_string(),
            });

            i = j; // Skip to next question
        } else {
            i += 1;
        }
    }

    if questions.is_empty() {
        return Err(anyhow!("No questions found in quiz file"));
    }

    Ok(ParsedMainQuestions {
        total_count: questions.len(),
        questions,
    })
}

/// Parse the quiz file in Phase 2: extract all 240 probes
///
/// Returns full Quiz structure with 240 probes (80 main × 3 sub-probes).
/// Includes feature tags and archetype mappings from probe annotations.
pub fn parse_full_quiz(content: &str, version: String) -> Result<ParsedQuiz> {
    let mut probes = Vec::new();
    let mut warnings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut current_question_number = 0u32;
    let mut current_question_title = String::new();

    while i < lines.len() {
        let line = lines[i].trim();

        // Question header
        if let Some(header) = parse_question_header(line) {
            current_question_number = header.0;
            current_question_title = header.1;
            i += 1;
            continue;
        }

        // Probe header: "Probe A — Concrete Recall"
        if let Some(probe_header) = parse_probe_header(line) {
            let (probe_letter, probe_title) = probe_header;

            // Calculate probe ID: Q1 ProbeA=1, ProbeB=2, ProbeC=3; Q2 ProbeA=4, etc.
            let probe_id = ((current_question_number - 1) * 3)
                + match probe_letter {
                    'A' => 1,
                    'B' => 2,
                    'C' => 3,
                    _ => {
                        warnings.push(format!(
                            "Q{}: Unknown probe letter '{}'",
                            current_question_number, probe_letter
                        ));
                        continue;
                    }
                };

            // Sub-probe number (1-3)
            let sub_probe_num = match probe_letter {
                'A' => 1,
                'B' => 2,
                'C' => 3,
                _ => 1,
            };

            // Extract probe text (until arrow annotations)
            let mut probe_text = String::new();
            let mut j = i + 1;

            while j < lines.len() {
                let current = lines[j].trim();

                // Stop at arrows or next probe/question
                if current.starts_with("→")
                    || current.starts_with("Probe ")
                    || (current.starts_with("Q") && current.contains("—"))
                {
                    break;
                }

                if !probe_text.is_empty() || !current.is_empty() {
                    if !probe_text.is_empty() && !current.is_empty() {
                        probe_text.push(' ');
                    }
                    probe_text.push_str(current);
                }

                j += 1;
            }

            if probe_text.is_empty() {
                warnings.push(format!(
                    "Q{} Probe {}: Empty probe text",
                    current_question_number, probe_letter
                ));
                i = j;
                continue;
            }

            // Extract feature tags and archetype mappings from arrows
            let mut features = Vec::new();
            let mut archetypes = Vec::new();

            while j < lines.len() {
                let current = lines[j].trim();

                // Stop at next probe or question
                if current.starts_with("Probe ") || (current.starts_with("Q") && current.contains("—")) {
                    break;
                }

                // Parse arrow annotations
                if current.starts_with("→") {
                    let annotation = current.trim_start_matches("→").trim();

                    // Check if it's features (comma-separated identifiers)
                    // vs structured data (contains braces or parens)
                    if annotation.contains('{') || annotation.contains('(') {
                        // Structured metadata - extract archetype if present
                        // Format: ConcreteRecall { sensory_detail_count, ... }
                        if let Some(archetype_name) = annotation.split('{').next() {
                            let archetype = archetype_name.trim().to_string();
                            if !archetype.is_empty() && !archetypes.contains(&archetype) {
                                archetypes.push(archetype);
                            }
                        }
                    } else {
                        // Feature tags (comma-separated)
                        for tag in annotation.split(',') {
                            let feature = tag.trim().to_string();
                            if !feature.is_empty() && !features.contains(&feature) {
                                features.push(feature);
                            }
                        }
                    }
                }

                j += 1;
            }

            // Determine decade (psychological domain clustering)
            let decade = calculate_decade(current_question_number);

            probes.push(Probe {
                id: probe_id,
                text: probe_text.trim().to_string(),
                primary_question: current_question_number,
                sub_probe: sub_probe_num,
                features,
                archetypes,
                decade,
            });

            i = j;
        } else {
            i += 1;
        }
    }

    if probes.is_empty() {
        return Err(anyhow!("No probes found in quiz file"));
    }

    // Validate probe count
    if probes.len() != 240 {
        warnings.push(format!(
            "Expected 240 probes, found {}. Check for missing questions or probes.",
            probes.len()
        ));
    }

    // Validate probe IDs are sequential 1-240
    let mut probe_map: HashMap<u32, &Probe> = HashMap::new();
    for probe in &probes {
        if probe.id < 1 || probe.id > 240 {
            warnings.push(format!("Probe ID {} out of range (1-240)", probe.id));
        }
        if probe_map.insert(probe.id, probe).is_some() {
            warnings.push(format!("Duplicate probe ID: {}", probe.id));
        }
    }

    // Check for missing IDs
    for expected_id in 1..=240 {
        if !probe_map.contains_key(&expected_id) {
            warnings.push(format!("Missing probe ID: {}", expected_id));
        }
    }

    Ok(ParsedQuiz {
        quiz: Quiz {
            probes,
            version,
            total_probes: 240,
        },
        warnings,
    })
}

/// Parse question header: "Q1 — Present Moment Anchoring"
/// Returns (number, title)
fn parse_question_header(line: &str) -> Option<(u32, String)> {
    if !line.starts_with('Q') {
        return None;
    }

    let parts: Vec<&str> = line.splitn(2, "—").collect();
    if parts.len() != 2 {
        return None;
    }

    let number_part = parts[0].trim().trim_start_matches('Q');
    let number = number_part.parse::<u32>().ok()?;

    let title = parts[1].trim().to_string();

    Some((number, title))
}

/// Parse probe header: "Probe A — Concrete Recall"
/// Returns (letter, title)
fn parse_probe_header(line: &str) -> Option<(char, String)> {
    if !line.starts_with("Probe ") {
        return None;
    }

    let rest = line.trim_start_matches("Probe ").trim();
    let parts: Vec<&str> = rest.splitn(2, "—").collect();

    if parts.len() != 2 {
        return None;
    }

    let letter = parts[0].trim().chars().next()?;
    let title = parts[1].trim().to_string();

    Some((letter, title))
}

/// Calculate decade (psychological domain) from question number
///
/// Decades organize questions into thematic clusters:
/// - Decade 1 (Q1-16): Foundational awareness (time, agency, emotion)
/// - Decade 2 (Q17-32): Self-regulation (attention, boundaries, autonomy)
/// - Decade 3 (Q33-48): Relational patterns (belonging, conflict, vulnerability)
/// - Decade 4 (Q49-64): Existential themes (meaning, purpose, responsibility)
/// - Decade 5 (Q65-80): Integration (inner authority, growth, self-understanding)
fn calculate_decade(question_number: u32) -> u32 {
    match question_number {
        1..=16 => 1,
        17..=32 => 2,
        33..=48 => 3,
        49..=64 => 4,
        65..=80 => 5,
        _ => 0, // Invalid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_question_header() {
        assert_eq!(
            parse_question_header("Q1 — Present Moment Anchoring"),
            Some((1, "Present Moment Anchoring".to_string()))
        );
        assert_eq!(
            parse_question_header("Q42 — Test Title"),
            Some((42, "Test Title".to_string()))
        );
        assert_eq!(parse_question_header("Not a question"), None);
        assert_eq!(parse_question_header("Q — No number"), None);
    }

    #[test]
    fn test_parse_probe_header() {
        assert_eq!(
            parse_probe_header("Probe A — Concrete Recall"),
            Some(('A', "Concrete Recall".to_string()))
        );
        assert_eq!(
            parse_probe_header("Probe B — Internal Process"),
            Some(('B', "Internal Process".to_string()))
        );
        assert_eq!(parse_probe_header("Not a probe"), None);
    }

    #[test]
    fn test_calculate_decade() {
        assert_eq!(calculate_decade(1), 1);
        assert_eq!(calculate_decade(16), 1);
        assert_eq!(calculate_decade(17), 2);
        assert_eq!(calculate_decade(32), 2);
        assert_eq!(calculate_decade(33), 3);
        assert_eq!(calculate_decade(48), 3);
        assert_eq!(calculate_decade(49), 4);
        assert_eq!(calculate_decade(64), 4);
        assert_eq!(calculate_decade(65), 5);
        assert_eq!(calculate_decade(80), 5);
        assert_eq!(calculate_decade(0), 0);
        assert_eq!(calculate_decade(81), 0);
    }

    #[test]
    fn test_parse_main_questions_minimal() {
        let content = r#"
Q1 — Test Question One

Primary:

What is your first question?

Probe A — Sub Question
Details here

Q2 — Test Question Two

Primary:

What is your second question?

Probe A — Another Sub
More details
"#;

        let result = parse_main_questions(content).unwrap();
        assert_eq!(result.total_count, 2);
        assert_eq!(result.questions.len(), 2);

        assert_eq!(result.questions[0].number, 1);
        assert_eq!(result.questions[0].title, "Test Question One");
        assert_eq!(
            result.questions[0].primary_text,
            "What is your first question?"
        );

        assert_eq!(result.questions[1].number, 2);
        assert_eq!(result.questions[1].title, "Test Question Two");
        assert_eq!(
            result.questions[1].primary_text,
            "What is your second question?"
        );
    }

    #[test]
    fn test_parse_full_quiz_minimal() {
        let content = r#"
Q1 — Test Question

Primary:
What is the question?

Probe A — Concrete Recall
Describe a specific example.
→ TemporalReference, AgencyMarkers
→ ConcreteRecall { sensory_detail_count }

Probe B — Internal Process
What happens inside?
→ EmotionalSurface
→ InternalProcess { emotion_level }

Probe C — Meaning Reflection
What does this mean?
→ CoherenceMetrics
→ MeaningReflection { self_attribution }
"#;

        let result = parse_full_quiz(content, "test-1.0.0".to_string()).unwrap();
        assert_eq!(result.quiz.probes.len(), 3);

        // Check probe 1
        assert_eq!(result.quiz.probes[0].id, 1);
        assert_eq!(result.quiz.probes[0].primary_question, 1);
        assert_eq!(result.quiz.probes[0].sub_probe, 1);
        assert!(result.quiz.probes[0].features.contains(&"TemporalReference".to_string()));
        assert!(result.quiz.probes[0].archetypes.contains(&"ConcreteRecall".to_string()));
        assert_eq!(result.quiz.probes[0].decade, 1);

        // Check probe 2
        assert_eq!(result.quiz.probes[1].id, 2);
        assert_eq!(result.quiz.probes[1].sub_probe, 2);
        assert!(result.quiz.probes[1].features.contains(&"EmotionalSurface".to_string()));

        // Check probe 3
        assert_eq!(result.quiz.probes[2].id, 3);
        assert_eq!(result.quiz.probes[2].sub_probe, 3);
    }
}
