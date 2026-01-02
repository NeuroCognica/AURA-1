//! Test quiz parser against actual master_quiz.md file

use aura_backend::quiz_parser::{parse_main_questions, parse_full_quiz};
use std::fs;

#[test]
fn test_parse_master_quiz_phase1_main_questions() {
    let content = fs::read_to_string("../mirrorborn_quiz/master_quiz.md")
        .expect("Failed to read master_quiz.md");

    let result = parse_main_questions(&content)
        .expect("Failed to parse main questions");

    // Should have exactly 80 main questions
    assert_eq!(result.total_count, 80, "Expected 80 main questions");
    assert_eq!(result.questions.len(), 80);

    // Check first question
    assert_eq!(result.questions[0].number, 1);
    assert_eq!(result.questions[0].title, "Present Moment Anchoring");
    assert!(result.questions[0].primary_text.contains("what feels most real"));

    // Check last question
    assert_eq!(result.questions[79].number, 80);
    assert_eq!(result.questions[79].title, "Relationship to Self-Understanding");

    // Check a middle question
    let q40 = result.questions.iter().find(|q| q.number == 40).expect("Q40 not found");
    assert_eq!(q40.title, "Self-Respect");

    println!("✓ Phase 1: Parsed {} main questions successfully", result.total_count);
}

#[test]
fn test_parse_master_quiz_phase2_full_probes() {
    let content = fs::read_to_string("../mirrorborn_quiz/master_quiz.md")
        .expect("Failed to read master_quiz.md");

    let result = parse_full_quiz(&content, "1.0.0".to_string())
        .expect("Failed to parse full quiz");

    // Should have exactly 240 probes (80 questions × 3 probes)
    assert_eq!(result.quiz.probes.len(), 240, "Expected 240 probes");
    assert_eq!(result.quiz.total_probes, 240);

    // Check probe IDs are sequential 1-240
    for i in 0..240 {
        assert_eq!(result.quiz.probes[i].id, (i + 1) as u32,
            "Probe {} has wrong ID", i);
    }

    // Check first probe (Q1 Probe A)
    assert_eq!(result.quiz.probes[0].id, 1);
    assert_eq!(result.quiz.probes[0].primary_question, 1);
    assert_eq!(result.quiz.probes[0].sub_probe, 1);
    assert_eq!(result.quiz.probes[0].decade, 1);
    assert!(!result.quiz.probes[0].text.is_empty());

    // Check probe 2 (Q1 Probe B)
    assert_eq!(result.quiz.probes[1].primary_question, 1);
    assert_eq!(result.quiz.probes[1].sub_probe, 2);

    // Check probe 3 (Q1 Probe C)
    assert_eq!(result.quiz.probes[2].primary_question, 1);
    assert_eq!(result.quiz.probes[2].sub_probe, 3);

    // Check probe 4 (Q2 Probe A)
    assert_eq!(result.quiz.probes[3].primary_question, 2);
    assert_eq!(result.quiz.probes[3].sub_probe, 1);

    // Check last probe (Q80 Probe C)
    assert_eq!(result.quiz.probes[239].id, 240);
    assert_eq!(result.quiz.probes[239].primary_question, 80);
    assert_eq!(result.quiz.probes[239].sub_probe, 3);
    assert_eq!(result.quiz.probes[239].decade, 5);

    // Check decade distribution
    let decade1_count = result.quiz.probes.iter().filter(|p| p.decade == 1).count();
    let decade2_count = result.quiz.probes.iter().filter(|p| p.decade == 2).count();
    let decade5_count = result.quiz.probes.iter().filter(|p| p.decade == 5).count();

    assert_eq!(decade1_count, 48, "Decade 1 should have 48 probes (Q1-16 × 3)");
    assert_eq!(decade2_count, 48, "Decade 2 should have 48 probes (Q17-32 × 3)");
    assert_eq!(decade5_count, 48, "Decade 5 should have 48 probes (Q65-80 × 3)");

    // Check features extracted
    let probes_with_features = result.quiz.probes.iter().filter(|p| !p.features.is_empty()).count();
    println!("✓ {} probes have feature tags", probes_with_features);

    // Check archetypes extracted
    let probes_with_archetypes = result.quiz.probes.iter().filter(|p| !p.archetypes.is_empty()).count();
    println!("✓ {} probes have archetype mappings", probes_with_archetypes);

    // Print warnings if any
    if !result.warnings.is_empty() {
        println!("⚠ Warnings during parsing:");
        for warning in &result.warnings {
            println!("  - {}", warning);
        }
    }

    println!("✓ Phase 2: Parsed {} probes successfully", result.quiz.probes.len());
}
