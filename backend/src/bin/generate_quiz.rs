//! Generate quiz.json from master_quiz.md

use aura_backend::quiz_loader::generate_quiz_json;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let master_quiz_path = PathBuf::from("../mirrorborn_quiz/master_quiz.md");
    let json_output_path = PathBuf::from("data/quiz.json");
    let version = "1.0.0".to_string();

    println!("Generating quiz JSON from master_quiz.md...");
    println!("  Source: {:?}", master_quiz_path);
    println!("  Output: {:?}", json_output_path);
    println!();

    let quiz = generate_quiz_json(&master_quiz_path, &json_output_path, version)?;

    println!();
    println!("Quiz statistics:");
    println!("  Total probes: {}", quiz.probes.len());
    println!("  Version: {}", quiz.version);

    // Feature coverage
    let probes_with_features = quiz.probes.iter().filter(|p| !p.features.is_empty()).count();
    let total_feature_tags: usize = quiz.probes.iter().map(|p| p.features.len()).sum();
    println!("  Probes with features: {} ({:.1}%)",
        probes_with_features,
        (probes_with_features as f64 / quiz.probes.len() as f64) * 100.0
    );
    println!("  Total feature tags: {}", total_feature_tags);

    // Archetype coverage
    let probes_with_archetypes = quiz.probes.iter().filter(|p| !p.archetypes.is_empty()).count();
    let total_archetype_tags: usize = quiz.probes.iter().map(|p| p.archetypes.len()).sum();
    println!("  Probes with archetypes: {} ({:.1}%)",
        probes_with_archetypes,
        (probes_with_archetypes as f64 / quiz.probes.len() as f64) * 100.0
    );
    println!("  Total archetype tags: {}", total_archetype_tags);

    // Decade distribution
    for decade in 1..=5 {
        let count = quiz.probes.iter().filter(|p| p.decade == decade).count();
        println!("  Decade {}: {} probes", decade, count);
    }

    println!();
    println!("✓ Quiz generation complete!");

    Ok(())
}
