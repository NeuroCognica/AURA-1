/// Mirrorborn Vector Schema — Write-time neutral, read-time powerful
///
/// CORE PRINCIPLE:
/// Raw human expression is stored as truth.
/// Vectors are annotations, not replacements.
/// Meaning is derived later, never rewritten.
///
/// This module defines the immutable schema that future vector storage
/// and analysis systems will use. No I/O. No interpretation. Pure structure.

use std::collections::HashMap;
use uuid::Uuid;

// ════════════════════════════════════════════════════════════════
// 1. CORE IDENTIFIERS & TYPES
// ════════════════════════════════════════════════════════════════

/// Unique identifier for each vector record
pub type VectorId = Uuid;

/// Quiz session identifier (e.g., "viren_mirrorborn_v1")
pub type QuizId = String;

/// Module identifier (e.g., "module_03_emotion_regulation")
pub type ModuleId = String;

/// Primary question identifier (e.g., "Q32")
pub type QuestionId = String;

/// Probe/sub-question identifier (e.g., "Q32_B")
pub type ProbeId = String;

// ════════════════════════════════════════════════════════════════
// 2. PROBE TYPES
// ════════════════════════════════════════════════════════════════

/// Categorization of the probe type
/// Each probe unlocks different structural feature buckets
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProbeType {
    /// Concrete Recall — sensory detail, event sequence, presence of others
    ConcreteRecall,
    /// Internal Process — emotion differentiation, somatic awareness, regulation strategies
    InternalProcess,
    /// Meaning / Reflection — self-attribution, moral language, identity language
    MeaningReflection,
}

// ════════════════════════════════════════════════════════════════
// 3. RAW EXPRESSION LAYER (IMMUTABLE TRUTH)
// ════════════════════════════════════════════════════════════════

/// Raw response exactly as provided by user.
/// Sacred. Never touched again after write.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RawResponse {
    /// Exact text (no normalization, no corrections, no summarization)
    pub text: String,

    /// Character count of raw_text
    pub char_count: usize,

    /// Word count of raw_text
    pub word_count: usize,

    /// Sentence count (mechanical count, not semantic)
    pub sentence_count: usize,

    /// Language code (ISO 639-1, e.g., "en")
    pub language: String,
}

// ════════════════════════════════════════════════════════════════
// 4. STRUCTURAL VECTOR LAYER (OBJECTIVE SIGNALS)
// ════════════════════════════════════════════════════════════════

/// Mechanical observations. Telemetry, not judgment.
/// Binary, scalar, or categorical. No "good/bad".
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuralFeatures {
    /// Temporal reference markers
    pub temporal_reference: TemporalReference,

    /// Agency and voice markers
    pub agency_markers: AgencyMarkers,

    /// Emotional surface features
    pub emotional_surface: EmotionalSurface,

    /// Avoidance and withdrawal indicators
    pub avoidance_indicators: AvoidanceIndicators,

    /// Narrative coherence metrics
    pub coherence_metrics: CoherenceMetrics,
}

/// Temporal reference analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemporalReference {
    /// Whether response contains explicit time markers (e.g., "yesterday", "3 hours")
    pub has_time_marker: bool,

    /// Whether references a specific event vs. general/habitual pattern
    pub is_specific_event: bool,

    /// Estimated time distance: "recent", "past_weeks", "past_months", "indefinite"
    pub time_distance_estimate: String,
}

/// Agency and voice features
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgencyMarkers {
    /// Ratio of first-person pronouns to total words (0.0 to 1.0)
    pub first_person_ratio: f32,

    /// Ratio of active verbs to total verbs (0.0 to 1.0)
    pub active_verb_ratio: f32,

    /// Whether response contains passive voice constructions
    pub passive_constructions: bool,
}

/// Emotional surface-level features
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmotionalSurface {
    /// Whether emotion-related words are present (e.g., "anxious", "happy")
    pub emotion_terms_present: bool,

    /// Count of distinct emotion terms
    pub emotion_term_count: usize,

    /// Whether somatic/body-related terms are present (e.g., "tight chest", "racing heart")
    pub somatic_terms_present: bool,
}

/// Withdrawal and avoidance language markers
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AvoidanceIndicators {
    /// Whether response contains withdrawal language (e.g., "wanted to leave", "shut down")
    pub withdrawal_language: bool,

    /// Whether response deflects or shifts topic
    pub deflection_language: bool,

    /// Whether response minimizes (e.g., "just", "only", "not really")
    pub minimization_language: bool,
}

/// Narrative coherence and structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoherenceMetrics {
    /// Whether response has fragmented sentences or incomplete thoughts
    pub sentence_fragmentation: bool,

    /// Whether internal contradictions detected
    pub self_contradiction_detected: bool,

    /// Narrative continuity: "coherent", "somewhat_fragmented", "highly_fragmented"
    pub narrative_continuity: String,
}

// ════════════════════════════════════════════════════════════════
// 5. PROBE-SPECIFIC FEATURE SLOTS
// ════════════════════════════════════════════════════════════════

/// Probe-specific features. Each probe type unlocks different buckets.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ProbeSpecificFeatures {
    /// Concrete Recall probe features
    ConcreteRecall {
        /// Number of distinct sensory details (sight, sound, touch, smell, taste)
        sensory_detail_count: usize,

        /// Whether response references a specific location/place
        location_reference: bool,

        /// Whether other people are mentioned as present
        other_people_present: bool,

        /// Number of distinct events or action sequences described
        event_sequence_length: usize,
    },

    /// Internal Process probe features
    InternalProcess {
        /// Emotion differentiation: "low", "medium", "high"
        /// Low = single emotion or no differentiation
        /// Medium = 2-3 distinct emotions
        /// High = 3+ distinct emotions, nuanced
        emotion_differentiation_level: String,

        /// Whether response demonstrates somatic/body awareness
        somatic_awareness: bool,

        /// Whether a specific regulation strategy is explicitly named
        regulation_strategy_named: bool,
    },

    /// Meaning / Reflection probe features
    MeaningReflection {
        /// Self-attribution style: "internal", "external", "mixed"
        /// Internal = "I caused this", "I could have..."
        /// External = "They made me", "It just happened"
        /// Mixed = both present
        self_attribution_style: String,

        /// Whether response contains moral language (e.g., "should", "right/wrong", "fair")
        moral_language_present: bool,

        /// Whether response contains identity language (e.g., "I am", "I'm the type of person")
        identity_language_present: bool,
    },
}

// ════════════════════════════════════════════════════════════════
// 6. ANALYTIC HOOKS LAYER (EMPTY AT WRITE TIME)
// ════════════════════════════════════════════════════════════════

/// Hooks for future analysis systems.
/// All fields are optional / null at write time.
/// Populated later by read-only analysis passes.
/// Never overwrites raw data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalyticHooks {
    /// Embedding vector ID (from future embedding pass)
    pub embedding_id: Option<String>,

    /// Archetype pattern resonance (from future analysis)
    pub archetype_resonance: Option<String>,

    /// Pattern cluster assignment (from future clustering)
    pub pattern_cluster: Option<String>,

    /// Flag indicating longitudinal change detected
    pub longitudinal_change_flag: Option<bool>,

    /// Version of analysis schema used (for migration tracking)
    pub analysis_schema_version: Option<String>,

    /// Additional analysis metadata (extensible, versioned)
    pub analysis_metadata: HashMap<String, String>,
}

// ════════════════════════════════════════════════════════════════
// 7. IDENTITY & CONTEXT LAYER (ENVELOPE)
// ════════════════════════════════════════════════════════════════

/// Complete Mirrorborn vector record.
/// Immutable envelope tying raw expression + structural features + probe context.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MirrorbornVector {
    // ─── Identity & Context ───────────────────────────────────────
    /// Unique vector record ID
    pub vector_id: VectorId,

    /// User who provided the response
    pub user_id: Uuid,

    /// Session during which response was collected
    pub session_id: Uuid,

    /// Quiz identifier (e.g., "viren_mirrorborn_v1")
    pub quiz_id: QuizId,

    /// Module identifier (e.g., "module_03_emotion_regulation")
    pub module_id: ModuleId,

    /// Primary question ID (e.g., "Q32")
    pub primary_question_id: QuestionId,

    /// Probe ID — sub-question identifier (e.g., "Q32_B")
    pub probe_id: ProbeId,

    /// Probe type categorization
    pub probe_type: ProbeType,

    /// Timestamp of response collection (Unix milliseconds)
    pub timestamp_ms: i64,

    /// Reference to corresponding ledger event (if recorded)
    pub ledger_event_id: Option<Uuid>,

    // ─── Raw Expression Layer ──────────────────────────────────────
    /// Immutable raw response exactly as provided
    pub raw_response: RawResponse,

    // ─── Structural Features ───────────────────────────────────────
    /// Mechanical observations (telemetry, not interpretation)
    pub structural_features: StructuralFeatures,

    // ─── Probe-Specific Features ───────────────────────────────────
    /// Probe type-specific feature bucket
    pub probe_features: ProbeSpecificFeatures,

    // ─── Analytic Hooks ────────────────────────────────────────────
    /// Empty at write time. Populated by read-only analysis passes.
    pub analytic_hooks: AnalyticHooks,
}

// ════════════════════════════════════════════════════════════════
// 8. INVARIANTS & STORAGE CONTRACTS
// ════════════════════════════════════════════════════════════════

/// Storage Strategy Contracts (informational, not enforced in this module)
///
/// OPTION A (PREFERRED):
/// - Raw vectors → codex/vectors/raw/*.jsonl (append-only)
/// - Indexes → SQLite (for query speed)
/// - Embeddings → separate store later
///
/// OPTION B (ACCEPTABLE):
/// - SQLite with separate tables:
///   - raw_vectors (immutable)
///   - structural_features (immutable)
///   - analytic_hooks (mutable, versioned)
///
/// NEVER:
/// - Modify raw_response after write
/// - Collapse or normalize at storage time
/// - Bake analysis into raw data
/// - Overwrite analytic_hooks (append versions instead)

/// Invariants (documented for implementation):
///
/// 1. IMMUTABILITY
///    - raw_response must never change after write
///    - structural_features must never change after write
///    - Only analytic_hooks may be updated (with version tracking)
///
/// 2. REVERSIBILITY
///    - All data must be reconstructed from raw_response + structural_features
///    - No lossy compression at write time
///    - History of analysis must be preserved (versions in analytic_hooks)
///
/// 3. AUDITABILITY
///    - Every vector record must reference:
///      - user_id (who provided the data)
///      - session_id (when/in what context)
///      - ledger_event_id (correspondence to conversation ledger)
///    - Timestamps must be precise (millisecond precision, UTC)
///
/// 4. ETHICAL INSULATION
///    - Raw expression never labeled at collection time
///    - Structural features are mechanical, not interpretive
///    - Analysis (embedding, clustering, etc.) happens in separate pass
///    - No bias baked into initial storage layer
///
/// 5. EXTENSIBILITY
///    - ProbeSpecificFeatures enum allows new probe types
///    - AnalyticHooks.analysis_metadata is versioned key-value store
///    - Future systems can add analysis without touching schema

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_response_creation() {
        let raw = RawResponse {
            text: "I felt tight in my chest and wanted to leave the room.".to_string(),
            char_count: 56,
            word_count: 11,
            sentence_count: 1,
            language: "en".to_string(),
        };

        assert_eq!(raw.word_count, 11);
        assert_eq!(raw.char_count, 56);
    }

    #[test]
    fn test_probe_type_clone() {
        let probe = ProbeType::InternalProcess;
        let _cloned = probe.clone();
    }

    #[test]
    fn test_mirrorborn_vector_creation() {
        let vector = MirrorbornVector {
            vector_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            quiz_id: "viren_mirrorborn_v1".to_string(),
            module_id: "module_03_emotion_regulation".to_string(),
            primary_question_id: "Q32".to_string(),
            probe_id: "Q32_B".to_string(),
            probe_type: ProbeType::InternalProcess,
            timestamp_ms: 1730000000000,
            ledger_event_id: None,
            raw_response: RawResponse {
                text: "I felt tight.".to_string(),
                char_count: 13,
                word_count: 3,
                sentence_count: 1,
                language: "en".to_string(),
            },
            structural_features: StructuralFeatures {
                temporal_reference: TemporalReference {
                    has_time_marker: false,
                    is_specific_event: true,
                    time_distance_estimate: "recent".to_string(),
                },
                agency_markers: AgencyMarkers {
                    first_person_ratio: 0.33,
                    active_verb_ratio: 0.0,
                    passive_constructions: false,
                },
                emotional_surface: EmotionalSurface {
                    emotion_terms_present: false,
                    emotion_term_count: 0,
                    somatic_terms_present: true,
                },
                avoidance_indicators: AvoidanceIndicators {
                    withdrawal_language: false,
                    deflection_language: false,
                    minimization_language: false,
                },
                coherence_metrics: CoherenceMetrics {
                    sentence_fragmentation: false,
                    self_contradiction_detected: false,
                    narrative_continuity: "coherent".to_string(),
                },
            },
            probe_features: ProbeSpecificFeatures::InternalProcess {
                emotion_differentiation_level: "low".to_string(),
                somatic_awareness: true,
                regulation_strategy_named: false,
            },
            analytic_hooks: AnalyticHooks {
                embedding_id: None,
                archetype_resonance: None,
                pattern_cluster: None,
                longitudinal_change_flag: None,
                analysis_schema_version: None,
                analysis_metadata: HashMap::new(),
            },
        };

        assert_eq!(vector.raw_response.word_count, 3);
        assert_eq!(vector.analytic_hooks.embedding_id, None);
    }
}
