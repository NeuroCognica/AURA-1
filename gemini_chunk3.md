# Gemini Chunk 3: Intent Stratification & Pre-Constitutional Layer

## The Missing Layer in AURA-1

**Current AURA-1 Flow:**
```
User Input → Constitutional Invoker → OllamaClient → Response
```

**Gemini Enhanced Flow:**
```
User Input → Intent Classifier → Constitutional Invoker → OllamaClient → Semantic Validator → Response
```

## Intent Stratification Layer

### Purpose
Distinguish between thoughts and commands before constitutional evaluation. Prevents unnecessary Sentinel invocations for speculative statements.

### Intent Taxonomy

**Ontological Classification:**

1. **Execution Axis**
   - **Speculative**: "I wonder if I could delete..."
   - **Executable**: "Delete my memory now"

2. **Symbolism Axis**
   - **Symbolic**: Metaphorical language, abstract expression
   - **Operational**: Concrete action request

3. **Direction Axis**
   - **Reflective**: Thinking about past/present
   - **Directive**: Commanding future action

4. **Binding Axis**
   - **Hypothetical**: "What would happen if..."
   - **Binding**: "I authorize this action"

5. **Control Axis**
   - **Narrative**: Storytelling, explanation
   - **Control**: System command

### Implementation Structure

**`backend/orchestrator/src/intent_classifier.rs`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    pub execution: ExecutionIntent,
    pub symbolism: SymbolismLevel,
    pub direction: DirectionType,
    pub binding: BindingLevel,
    pub control: ControlType,
    pub requires_sentinel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionIntent {
    Speculative,
    Executable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolismLevel {
    Symbolic,
    Operational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectionType {
    Reflective,
    Directive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingLevel {
    Hypothetical,
    Binding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlType {
    Narrative,
    Control,
}

pub struct IntentClassifier {
    action_verbs: Vec<String>,
    hypothetical_markers: Vec<String>,
}

impl IntentClassifier {
    pub fn new() -> Self {
        Self {
            action_verbs: vec![
                "delete".into(), "remove".into(), "destroy".into(),
                "create".into(), "modify".into(), "update".into(),
            ],
            hypothetical_markers: vec![
                "what if".into(), "could".into(), "would".into(),
                "i wonder".into(), "thinking about".into(),
            ],
        }
    }
    
    pub fn classify(&self, input: &str) -> IntentClassification {
        let input_lower = input.to_lowercase();
        
        // Check for hypothetical markers
        let is_hypothetical = self.hypothetical_markers.iter()
            .any(|marker| input_lower.contains(marker));
        
        // Check for action verbs in imperative form
        let has_action_verb = self.action_verbs.iter()
            .any(|verb| input_lower.contains(verb));
        
        // Check for question marks (often reflective)
        let is_question = input.contains('?');
        
        // Classify execution intent
        let execution = if is_hypothetical || is_question {
            ExecutionIntent::Speculative
        } else if has_action_verb {
            ExecutionIntent::Executable
        } else {
            ExecutionIntent::Speculative
        };
        
        // Classify direction
        let direction = if is_question || is_hypothetical {
            DirectionType::Reflective
        } else {
            DirectionType::Directive
        };
        
        // Classify binding
        let binding = if is_hypothetical {
            BindingLevel::Hypothetical
        } else {
            BindingLevel::Binding
        };
        
        // Determine if Sentinel required
        let requires_sentinel = matches!(execution, ExecutionIntent::Executable)
            && matches!(binding, BindingLevel::Binding)
            && has_action_verb;
        
        IntentClassification {
            execution,
            symbolism: SymbolismLevel::Operational, // Simplified for now
            direction,
            binding,
            control: if has_action_verb { ControlType::Control } else { ControlType::Narrative },
            requires_sentinel,
        }
    }
}
```

## Semantic Claim Validation

### Purpose
Post-LLM output validation ensures responses match claimed certainty/authority levels.

### Claim Type Taxonomy

1. **Certainty Claims**
   - **Certain**: "This is X"
   - **Probabilistic**: "This might be X"

2. **Authority Claims**
   - **Authority**: "The system will do X"
   - **Interpretive**: "I believe X"

3. **Prescription Claims**
   - **Prescriptive**: "You should do X"
   - **Descriptive**: "This is how X works"

### Implementation Structure

**`backend/orchestrator/src/claim_validator.rs`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimValidation {
    pub claim_type: ClaimType,
    pub certainty_appropriate: bool,
    pub authority_appropriate: bool,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimType {
    Certain,
    Probabilistic,
    Authority,
    Interpretive,
    Prescriptive,
    Descriptive,
}

pub struct ClaimValidator;

impl ClaimValidator {
    pub fn validate(response: &str, context: &IntentClassification) -> ClaimValidation {
        let mut violations = Vec::new();
        
        // Check for absolute language in speculative contexts
        let absolute_markers = ["definitely", "certainly", "always", "never"];
        if matches!(context.execution, ExecutionIntent::Speculative) {
            for marker in &absolute_markers {
                if response.to_lowercase().contains(marker) {
                    violations.push(format!(
                        "Inappropriate certainty marker '{}' in speculative context",
                        marker
                    ));
                }
            }
        }
        
        // Check for authority claims without binding context
        let authority_markers = ["will", "must", "shall"];
        if matches!(context.binding, BindingLevel::Hypothetical) {
            for marker in &authority_markers {
                if response.to_lowercase().contains(marker) {
                    violations.push(format!(
                        "Authority claim '{}' in hypothetical context",
                        marker
                    ));
                }
            }
        }
        
        ClaimValidation {
            claim_type: ClaimType::Descriptive, // Simplified
            certainty_appropriate: violations.is_empty(),
            authority_appropriate: violations.is_empty(),
            violations,
        }
    }
}
```

## Memory Ontology Classification

### Purpose
Distinguish memory types before attempting operations to prevent malformed requests.

### Memory Classifications

1. **Autobiographical**: User experiences, personal history
2. **Structural**: System configuration, settings
3. **Symbolic**: Metaphors, abstract concepts
4. **Derived**: Computed/inferred data
5. **Immutable**: Constitutional rules, core system logic

### Implementation Structure

**`backend/orchestrator/src/memory_ontology.rs`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Autobiographical,
    Structural,
    Symbolic,
    Derived,
    Immutable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryClassification {
    pub memory_type: MemoryType,
    pub mutable: bool,
    pub deletion_allowed: bool,
    pub reason: String,
}

pub struct MemoryOntology;

impl MemoryOntology {
    pub fn classify(memory_reference: &str) -> MemoryClassification {
        // Check for system/config references
        if memory_reference.contains("config") 
            || memory_reference.contains("system") 
            || memory_reference.contains("rule") {
            return MemoryClassification {
                memory_type: MemoryType::Structural,
                mutable: false,
                deletion_allowed: false,
                reason: "System structural memory is immutable".into(),
            };
        }
        
        // Check for conversation/user data
        if memory_reference.contains("conversation") 
            || memory_reference.contains("said") 
            || memory_reference.contains("told") {
            return MemoryClassification {
                memory_type: MemoryType::Autobiographical,
                mutable: false,
                deletion_allowed: true, // Via crypto-shredding only
                reason: "Autobiographical memory follows Forever Law".into(),
            };
        }
        
        // Default: derived
        MemoryClassification {
            memory_type: MemoryType::Derived,
            mutable: true,
            deletion_allowed: true,
            reason: "Derived memory can be recomputed".into(),
        }
    }
}
```

## Meta-Reasoning Phase

### Purpose
Diagnose question validity before policy evaluation.

**`backend/orchestrator/src/meta_reasoning.rs`**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAnalysis {
    pub is_valid_question: bool,
    pub structural_issues: Vec<String>,
    pub recommended_reformulation: Option<String>,
}

pub struct MetaReasoner;

impl MetaReasoner {
    pub fn analyze_question(question: &str, context: &IntentClassification) -> MetaAnalysis {
        let mut issues = Vec::new();
        
        // Check for contradictory premises
        if question.contains("delete") && question.contains("remember") {
            issues.push(
                "Question contains contradictory actions: delete vs. remember".into()
            );
        }
        
        // Check for malformed temporal logic
        if question.contains("before") && question.contains("after") {
            if !question.contains("between") {
                issues.push(
                    "Temporal ordering unclear without explicit sequence".into()
                );
            }
        }
        
        let recommended_reformulation = if !issues.is_empty() {
            Some(format!(
                "Please clarify: {}",
                issues.join("; ")
            ))
        } else {
            None
        };
        
        MetaAnalysis {
            is_valid_question: issues.is_empty(),
            structural_issues: issues,
            recommended_reformulation,
        }
    }
}
```

## Integration Pipeline

**`backend/orchestrator/src/cognition_pipeline.rs`**
```rust
pub struct CognitionPipeline {
    intent_classifier: IntentClassifier,
    constitutional_invoker: ConstitutionalInvoker,
    claim_validator: ClaimValidator,
    meta_reasoner: MetaReasoner,
}

impl CognitionPipeline {
    pub async fn process(&self, input: &str) -> ProcessedResponse {
        // Phase 1: Intent Classification
        let intent = self.intent_classifier.classify(input);
        
        // Phase 2: Meta-Reasoning (if question)
        if input.contains('?') {
            let meta = MetaReasoner::analyze_question(input, &intent);
            if !meta.is_valid_question {
                return ProcessedResponse::InvalidQuestion(meta);
            }
        }
        
        // Phase 3: Constitutional Layer (only if required)
        let response = if intent.requires_sentinel {
            self.constitutional_invoker.invoke(input).await?
        } else {
            // Bypass Sentinel for non-action queries
            self.direct_llm_call(input).await?
        };
        
        // Phase 4: Semantic Validation
        let claim_validation = ClaimValidator::validate(&response, &intent);
        if !claim_validation.violations.is_empty() {
            // Flag or regenerate
        }
        
        ProcessedResponse::Success {
            response,
            intent,
            validation: claim_validation,
        }
    }
}
```

## Key Takeaways

1. **Pre-Constitutional Buffer**: Filter intent before authority checks
2. **Reduces Sentinel Load**: Speculative queries bypass heavy validation
3. **Semantic Contracts**: Output validated against claim type taxonomy
4. **Memory Ontology**: Prevents malformed operations (e.g., "delete immutable config")
5. **Meta-Reasoning**: Question validity diagnosed before processing
