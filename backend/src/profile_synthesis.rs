//! Profile synthesis from quiz answers
//!
//! Generates Mirrorborn profiles from 240 accumulated quiz answers.
//! Analyzes feature weights and determines archetype alignment.

use crate::profile::MirrorbornProfile;
use crate::quiz::{Answer, QuizSession, SessionStatus};
use crate::quiz_tagging::{extract_tags, TagExtractionResult};
use crate::quiz_loader::load_quiz_json;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Profile synthesis configuration
pub struct SynthesisConfig {
    /// Path to quiz.json for probe metadata
    pub quiz_json_path: String,
    
    /// Model version string (for provenance)
    pub model_version: String,
    
    /// Minimum answers required (default: 240)
    pub min_answers: u32,
}

impl Default for SynthesisConfig {
    fn default() -> Self {
        Self {
            quiz_json_path: "data/quiz.json".to_string(),
            model_version: "1.0.0".to_string(),
            min_answers: 240,
        }
    }
}

/// Synthesize a Mirrorborn profile from quiz session answers
///
/// Process:
/// 1. Validate session is complete (240 answers)
/// 2. Load quiz metadata for probe features
/// 3. Accumulate feature weights across all answers
/// 4. Determine primary/secondary archetypes from patterns
/// 5. Create profile with provenance
pub async fn synthesize_profile(
    session: &QuizSession,
    answers: &[Answer],
    config: &SynthesisConfig,
) -> Result<MirrorbornProfile> {
    // Validate session completion
    if session.status != SessionStatus::Complete {
        return Err(anyhow!(
            "Session {} not complete (status: {:?})",
            session.session_id,
            session.status
        ));
    }
    
    if answers.len() < config.min_answers as usize {
        return Err(anyhow!(
            "Insufficient answers: {} < {}",
            answers.len(),
            config.min_answers
        ));
    }
    
    // Load quiz for probe metadata
    let quiz = load_quiz_json(Path::new(&config.quiz_json_path))
        .context("Failed to load quiz metadata")?;
    
    // Accumulate feature weights across all answers
    let mut accumulated_weights: HashMap<String, Vec<f64>> = HashMap::new();
    
    for answer in answers {
        // Get probe metadata
        let probe = quiz.probes.iter()
            .find(|p| p.id == answer.probe_id)
            .ok_or_else(|| anyhow!("Probe {} not found in quiz", answer.probe_id))?;
        
        // Use answer's feature_weights if already populated
        // (from async tagging), otherwise extract inline
        let weights = if !answer.feature_weights.is_empty() {
            answer.feature_weights.clone()
        } else {
            // Extract tags for this answer
            let extraction = extract_tags(&probe.text, &answer.response, &probe.features)
                .await
                .context("Failed to extract tags")?;
            extraction.feature_weights
        };
        
        // Accumulate weights
        for (feature, weight) in weights {
            accumulated_weights.entry(feature).or_insert_with(Vec::new).push(weight);
        }
    }
    
    // Normalize accumulated weights (average)
    let mut normalized_weights: HashMap<String, f64> = HashMap::new();
    for (feature, weights) in accumulated_weights {
        let avg = weights.iter().sum::<f64>() / weights.len() as f64;
        normalized_weights.insert(feature, avg);
    }
    
    // Determine primary/secondary archetypes from weight patterns
    let (primary, secondary) = determine_archetypes(&normalized_weights);
    
    // Create profile with provenance
    Ok(MirrorbornProfile {
        username: session.username.clone(),
        primary_archetype: primary,
        secondary_archetype: secondary,
        feature_weights: normalized_weights,
        generated_at_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
        session_id: session.session_id.clone(),
        model_version: config.model_version.clone(),
        total_answers: answers.len() as u32,
    })
}

/// Determine primary and secondary archetypes from feature weights
///
/// Mapping logic:
/// - High agency + high detail → Architect
/// - High detail + low agency → Technician
/// - High collaboration + emotional awareness → Empath
/// - High agency + low detail → Explorer
/// - Low agency + emotional awareness → Mentor
/// - High agency + low emotional awareness → Jester
/// - Balanced weights → Sentinel (default)
fn determine_archetypes(weights: &HashMap<String, f64>) -> (String, Option<String>) {
    let agency = weights.get("agency").copied().unwrap_or(0.5);
    let detail = weights.get("detail_orientation").copied().unwrap_or(0.5);
    let collaboration = weights.get("collaboration").copied().unwrap_or(0.5);
    let emotion = weights.get("emotional_awareness").copied().unwrap_or(0.5);
    
    // Score each archetype based on feature alignment
    let mut scores: HashMap<&str, f64> = HashMap::new();
    
    // Architect: High agency + high detail
    scores.insert("Architect", agency * detail * 2.0);
    
    // Technician: High detail + lower agency
    scores.insert("Technician", detail * (1.0 - agency) * 2.0);
    
    // Empath: High collaboration + high emotional awareness (favored at low/neutral agency)
    scores.insert("Empath", collaboration * emotion * (1.2 - agency * 0.4) * 2.0);
    
    // Explorer: High agency + low detail (big picture)
    scores.insert("Explorer", agency * (1.0 - detail) * 2.0);
    
    // Mentor: High emotional awareness + collaboration (favored at high agency)
    scores.insert("Mentor", emotion * collaboration * (0.6 + agency * 0.6) * 2.0);
    
    // Jester: High agency + low emotional awareness (action-oriented)
    scores.insert("Jester", agency * (1.0 - emotion) * 2.0);
    
    // Sentinel: Balanced (low variance) - multiply by 1.0 (no boost)
    let variance = [agency, detail, collaboration, emotion]
        .iter()
        .map(|&w| (w - 0.5).abs())
        .sum::<f64>() / 4.0;
    scores.insert("Sentinel", (1.0 - variance) * 1.0);
    
    // Sort by score descending
    let mut sorted: Vec<_> = scores.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    let primary = sorted[0].0.to_string();
    let secondary = if sorted[1].1 > 0.4 {
        Some(sorted[1].0.to_string())
    } else {
        None
    };
    
    (primary, secondary)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_determine_archetypes_architect() {
        let mut weights = HashMap::new();
        weights.insert("agency".to_string(), 0.8);
        weights.insert("detail_orientation".to_string(), 0.8);
        weights.insert("collaboration".to_string(), 0.5);
        weights.insert("emotional_awareness".to_string(), 0.5);
        
        let (primary, _) = determine_archetypes(&weights);
        assert_eq!(primary, "Architect");
    }
    
    #[test]
    fn test_determine_archetypes_empath() {
        let mut weights = HashMap::new();
        weights.insert("agency".to_string(), 0.5);
        weights.insert("detail_orientation".to_string(), 0.5);
        weights.insert("collaboration".to_string(), 0.9);
        weights.insert("emotional_awareness".to_string(), 0.9);
        
        let (primary, _) = determine_archetypes(&weights);
        assert_eq!(primary, "Empath");
    }
    
    #[test]
    fn test_determine_archetypes_explorer() {
        let mut weights = HashMap::new();
        weights.insert("agency".to_string(), 0.9);
        weights.insert("detail_orientation".to_string(), 0.2);
        weights.insert("collaboration".to_string(), 0.5);
        weights.insert("emotional_awareness".to_string(), 0.5);
        
        let (primary, _) = determine_archetypes(&weights);
        assert_eq!(primary, "Explorer");
    }
    
    #[test]
    fn test_determine_archetypes_sentinel_balanced() {
        let mut weights = HashMap::new();
        weights.insert("agency".to_string(), 0.5);
        weights.insert("detail_orientation".to_string(), 0.5);
        weights.insert("collaboration".to_string(), 0.5);
        weights.insert("emotional_awareness".to_string(), 0.5);
        
        let (primary, _) = determine_archetypes(&weights);
        assert_eq!(primary, "Sentinel");
    }
    
    #[test]
    fn test_determine_archetypes_with_secondary() {
        let mut weights = HashMap::new();
        weights.insert("agency".to_string(), 0.8);
        weights.insert("detail_orientation".to_string(), 0.8);
        weights.insert("collaboration".to_string(), 0.7);
        weights.insert("emotional_awareness".to_string(), 0.7);
        
        let (primary, secondary) = determine_archetypes(&weights);
        assert_eq!(primary, "Architect");
        assert!(secondary.is_some());
    }
}
