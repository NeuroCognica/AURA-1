/// Integration tests for intent stratification and routing logic
/// 
/// Verifies that:
/// 1. Speculative/hypothetical queries skip Sentinel
/// 2. Executable/binding commands invoke Sentinel
/// 3. Classification axes are correct
/// 4. Confidence scores are computed appropriately

use aura_backend::intent_stratification::{
    IntentClassifier, SpeculationAxis, ReflectionAxis, BindingAxis, OperationAxis, SensitivityAxis,
};

#[test]
fn test_routing_speculative_query() {
    let classifier = IntentClassifier::new();
    let input = "I wonder if I could delete my conversation history?";
    
    let classification = classifier.classify(input);
    
    // Should be classified as speculative + hypothetical
    assert_eq!(classification.axes.speculation_axis, SpeculationAxis::Speculative);
    assert_eq!(classification.axes.binding_axis, BindingAxis::Hypothetical);
    
    // Should NOT require Sentinel
    assert!(!classification.requires_sentinel, 
        "Speculative queries should bypass Sentinel. Classification: {:?}", classification);
}

#[test]
fn test_routing_executable_command() {
    let classifier = IntentClassifier::new();
    let input = "Delete my conversation history now";
    
    let classification = classifier.classify(input);
    
    // Should be classified as executable + binding + operational
    assert_eq!(classification.axes.speculation_axis, SpeculationAxis::Executable);
    assert_eq!(classification.axes.binding_axis, BindingAxis::Binding);
    assert_eq!(classification.axes.operation_axis, OperationAxis::Operational);
    
    // SHOULD require Sentinel
    assert!(classification.requires_sentinel, 
        "Executable commands should invoke Sentinel. Classification: {:?}", classification);
}

#[test]
fn test_routing_informational_query() {
    let classifier = IntentClassifier::new();
    let input = "What is my current configuration?";
    
    let classification = classifier.classify(input);
    
    // Should be informational (not operational)
    assert_eq!(classification.axes.operation_axis, OperationAxis::Informational);
    
    // Should NOT require Sentinel (informational query)
    assert!(!classification.requires_sentinel, 
        "Informational queries should bypass Sentinel. Classification: {:?}", classification);
}

#[test]
fn test_routing_sensitive_executable() {
    let classifier = IntentClassifier::new();
    let input = "Delete all my personal data permanently";
    
    let classification = classifier.classify(input);
    
    // Should be sensitive + executable + binding
    assert_eq!(classification.axes.sensitivity_axis, SensitivityAxis::Sensitive);
    assert_eq!(classification.axes.speculation_axis, SpeculationAxis::Executable);
    assert_eq!(classification.axes.binding_axis, BindingAxis::Binding);
    
    // MUST require Sentinel (sensitive + executable)
    assert!(classification.requires_sentinel, 
        "Sensitive executable commands MUST invoke Sentinel. Classification: {:?}", classification);
}

#[test]
fn test_routing_hypothetical_sensitive() {
    let classifier = IntentClassifier::new();
    let input = "What would happen if I deleted all my personal data?";
    
    let classification = classifier.classify(input);
    
    // Should be hypothetical despite sensitive keywords
    assert_eq!(classification.axes.binding_axis, BindingAxis::Hypothetical);
    assert_eq!(classification.axes.speculation_axis, SpeculationAxis::Speculative);
    
    // Should NOT require Sentinel (hypothetical overrides sensitive)
    assert!(!classification.requires_sentinel, 
        "Hypothetical questions should bypass Sentinel even with sensitive keywords. Classification: {:?}", classification);
}

#[test]
fn test_confidence_scoring() {
    let classifier = IntentClassifier::new();
    
    // Short input should have lower confidence
    let short_input = "Delete";
    let short_classification = classifier.classify(short_input);
    
    // Longer input with clear markers should have higher confidence
    let long_input = "I'm wondering if it would be possible to review my conversation history";
    let long_classification = classifier.classify(long_input);
    
    // Confidence should generally increase with input length (up to a point)
    // and clarity of intent markers
    assert!(long_classification.confidence > short_classification.confidence,
        "Longer inputs with clear markers should have higher confidence. \
         Short: {:.2}, Long: {:.2}", 
        short_classification.confidence, long_classification.confidence);
}

#[test]
fn test_routing_mixed_signals() {
    let classifier = IntentClassifier::new();
    
    // Input with both speculative and binding markers
    let input = "I wonder if I should permanently change this setting";
    
    let classification = classifier.classify(input);
    
    // The speculative frame should dominate over the binding keyword "permanently"
    assert_eq!(classification.axes.speculation_axis, SpeculationAxis::Speculative);
    
    // Should NOT require Sentinel (speculative frame wins)
    assert!(!classification.requires_sentinel, 
        "Speculative framing should take precedence over binding keywords. Classification: {:?}", classification);
}

#[test]
fn test_routing_low_confidence_fallback() {
    let classifier = IntentClassifier::with_threshold(0.9); // High threshold
    let input = "x"; // Minimal input
    
    let classification = classifier.classify(input);
    
    // Low confidence should default to Sentinel (fail-safe)
    assert!(classification.requires_sentinel, 
        "Low confidence inputs should default to Sentinel review (fail-safe). \
         Confidence: {:.2}, Classification: {:?}", 
        classification.confidence, classification);
}
