//! LLM-based answer tagging and feature weight extraction
//!
//! This module provides silent analysis of quiz answers:
//! - Extracts cognitive/psychological tags from free-text responses
//! - Accumulates feature weights per answer
//! - Updates Answer struct with extracted_tags and feature_weights
//!
//! Covenant: Analysis happens asynchronously, never blocks user interaction

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tag extraction result from LLM analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagExtractionResult {
    /// Cognitive/psychological tags identified
    pub tags: Vec<String>,
    
    /// Feature weights derived from this answer (0.0-1.0)
    pub feature_weights: HashMap<String, f64>,
    
    /// Confidence in extraction (0.0-1.0)
    pub confidence: f64,
}

/// Extract tags from a quiz answer using LLM analysis
///
/// This is a silent operation—does not respond to user, only analyzes.
/// 
/// Example prompt structure:
/// ```text
/// Analyze this quiz answer for psychological markers.
/// Question: [probe text]
/// Answer: [user response]
/// 
/// Extract tags from these categories:
/// - Agency: autonomous, pushed, carried
/// - Emotion: regulated, suppressed, expressed
/// - Temporal: past-focused, present, future
/// - Coherence: fragmented, sequential, integrated
/// 
/// Return JSON: { "tags": [...], "confidence": 0.8 }
/// ```
pub async fn extract_tags(
    probe_text: &str,
    answer_text: &str,
    probe_features: &[String],
) -> Result<TagExtractionResult> {
    // Build analysis prompt
    let prompt = build_tagging_prompt(probe_text, answer_text, probe_features);
    
    // Call LLM (Ollama) for analysis
    // TODO: Integrate with Ollama client when implemented
    // For now, return stub with basic heuristics
    
    Ok(extract_tags_heuristic(answer_text, probe_features))
}

/// Build prompt for LLM tag extraction
fn build_tagging_prompt(
    probe_text: &str,
    answer_text: &str,
    probe_features: &[String],
) -> String {
    format!(
        r#"Analyze this quiz answer for psychological markers. Be concise and precise.

Question: {}

User's Answer: {}

Target Features: {}

Extract tags indicating:
1. Agency level (autonomous, guided, pushed, passive)
2. Emotional awareness (differentiated, surface, suppressed, avoided)
3. Temporal orientation (past-focused, present-centered, future-oriented)
4. Narrative coherence (fragmented, sequential, integrated, reflective)

Return JSON with this exact structure:
{{
  "tags": ["tag1", "tag2", ...],
  "confidence": 0.85
}}

Tags should be lowercase, underscore-separated (e.g., "high_agency", "emotion_suppressed").
Only include tags with strong evidence. Empty array is valid."#,
        probe_text,
        answer_text,
        probe_features.join(", ")
    )
}

/// Heuristic tag extraction (fallback/stub until LLM integration complete)
///
/// Basic pattern matching to provide functional tagging without LLM
fn extract_tags_heuristic(answer_text: &str, probe_features: &[String]) -> TagExtractionResult {
    let lower = answer_text.to_lowercase();
    let mut tags = Vec::new();
    let mut weights = HashMap::new();
    
    // Agency markers
    if lower.contains("i chose") || lower.contains("i decided") || lower.contains("i made") {
        tags.push("high_agency".to_string());
        weights.insert("agency".to_string(), 0.8);
    } else if lower.contains("had to") || lower.contains("forced") || lower.contains("no choice") {
        tags.push("low_agency".to_string());
        weights.insert("agency".to_string(), 0.2);
    } else {
        weights.insert("agency".to_string(), 0.5);
    }
    
    // Emotional awareness
    if lower.contains("feel") || lower.contains("felt") || lower.contains("emotion") {
        tags.push("emotion_aware".to_string());
        weights.insert("emotional_awareness".to_string(), 0.7);
    } else if lower.contains("don't know") || lower.contains("unclear") {
        tags.push("emotion_unclear".to_string());
        weights.insert("emotional_awareness".to_string(), 0.3);
    } else {
        weights.insert("emotional_awareness".to_string(), 0.5);
    }
    
    // Detail orientation (response length as proxy)
    let word_count = answer_text.split_whitespace().count();
    if word_count > 50 {
        tags.push("high_detail".to_string());
        weights.insert("detail_orientation".to_string(), 0.8);
    } else if word_count < 10 {
        tags.push("low_detail".to_string());
        weights.insert("detail_orientation".to_string(), 0.2);
    } else {
        weights.insert("detail_orientation".to_string(), 0.5);
    }
    
    // Collaboration indicators (question marks, asking for input)
    if lower.contains('?') || lower.contains("what do you think") {
        tags.push("collaborative".to_string());
        weights.insert("collaboration".to_string(), 0.7);
    } else {
        weights.insert("collaboration".to_string(), 0.5);
    }
    
    // Match probe-specific features
    for feature in probe_features {
        let feature_lower = feature.to_lowercase();
        if feature_lower.contains("agency") {
            // Already handled above
        } else if feature_lower.contains("temporal") {
            if lower.contains("yesterday") || lower.contains("last") || lower.contains("past") {
                tags.push("past_focused".to_string());
            } else if lower.contains("tomorrow") || lower.contains("will") || lower.contains("future") {
                tags.push("future_oriented".to_string());
            } else {
                tags.push("present_centered".to_string());
            }
        }
    }
    
    // Confidence based on answer length and clarity
    let confidence = if word_count > 5 && word_count < 200 {
        0.7
    } else if word_count <= 5 {
        0.4 // Very short answers lower confidence
    } else {
        0.6
    };
    
    TagExtractionResult {
        tags,
        feature_weights: weights,
        confidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_tags_high_agency() {
        let answer = "I chose to leave my job because I decided it was the right move.";
        let result = extract_tags_heuristic(answer, &[]);
        
        assert!(result.tags.contains(&"high_agency".to_string()));
        assert!(result.feature_weights.get("agency").unwrap() > &0.6);
    }
    
    #[test]
    fn test_extract_tags_low_agency() {
        let answer = "I had to quit because I had no choice, they forced me out.";
        let result = extract_tags_heuristic(answer, &[]);
        
        assert!(result.tags.contains(&"low_agency".to_string()));
        assert!(result.feature_weights.get("agency").unwrap() < &0.4);
    }
    
    #[test]
    fn test_extract_tags_emotional_awareness() {
        let answer = "I felt really anxious about the whole situation and the emotion was overwhelming.";
        let result = extract_tags_heuristic(answer, &[]);
        
        assert!(result.tags.contains(&"emotion_aware".to_string()));
    }
    
    #[test]
    fn test_extract_tags_detail_oriented() {
        let long_answer = "Well, there were many factors at play. First, the context was complex because of the overlapping timelines. Second, I had to consider multiple perspectives from different stakeholders. Third, the technical constraints were significant. Fourth, the budget limitations created additional pressure. Finally, the deadline was approaching rapidly which added urgency to every decision we made.";
        let result = extract_tags_heuristic(long_answer, &[]);
        
        assert!(result.tags.contains(&"high_detail".to_string()));
        assert!(result.feature_weights.get("detail_orientation").unwrap() > &0.6);
    }
    
    #[test]
    fn test_extract_tags_temporal() {
        let answer = "Yesterday I thought about my past experiences and how they shaped me.";
        let features = vec!["TemporalReference".to_string()];
        let result = extract_tags_heuristic(answer, &features);
        
        assert!(result.tags.contains(&"past_focused".to_string()));
    }
    
    #[test]
    fn test_build_tagging_prompt() {
        let prompt = build_tagging_prompt(
            "What did you do?",
            "I decided to leave.",
            &["AgencyMarkers".to_string()],
        );
        
        assert!(prompt.contains("What did you do?"));
        assert!(prompt.contains("I decided to leave"));
        assert!(prompt.contains("AgencyMarkers"));
        assert!(prompt.contains("JSON"));
    }
}
