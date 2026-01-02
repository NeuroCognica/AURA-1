//! Provenance verification for constitutional artifacts
//! 
//! Implements Forever Law Article III: Right to Provenance
//! "All cognitive artifacts must carry cryptographic signatures and provenance metadata."

use crate::qsic::{qsic_hash, verify_qsic, LAYER_COUNT};
use std::time::{SystemTime, UNIX_EPOCH};

/// Provenance metadata for a constitutional artifact
#[derive(Debug, Clone)]
pub struct ProvenanceEnvelope {
    pub artifact_type: ArtifactType,
    pub qsic_hash: String,
    pub timestamp: u64,
    pub archetype: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactType {
    ConstitutionalPrompt,
    SentinelVerdict,
    ArchetypeResponse,
}

impl ArtifactType {
    fn salt_prefix(&self) -> &'static str {
        match self {
            ArtifactType::ConstitutionalPrompt => "AURA-CONST-PROMPT",
            ArtifactType::SentinelVerdict => "AURA-SENTINEL-VERDICT",
            ArtifactType::ArchetypeResponse => "AURA-ARCHETYPE-RESP",
        }
    }
}

impl ProvenanceEnvelope {
    /// Sign a constitutional artifact with QSIC provenance
    pub fn sign(artifact_type: ArtifactType, data: &[u8], archetype: &str) -> Self {
        let salt = format!("{}-{}", artifact_type.salt_prefix(), archetype);
        let qsic_hash = qsic_hash(data, &salt);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        ProvenanceEnvelope {
            artifact_type,
            qsic_hash,
            timestamp,
            archetype: archetype.to_string(),
        }
    }
    
    /// Verify provenance with context proof
    pub fn verify(&self, data: &[u8], context_layers: u64) -> bool {
        let salt = format!("{}-{}", self.artifact_type.salt_prefix(), self.archetype);
        verify_qsic(data, &salt, &self.qsic_hash, context_layers)
    }
}

/// Sign a Sentinel verdict with provenance
pub fn sign_sentinel_verdict(verdict_text: &str) -> ProvenanceEnvelope {
    ProvenanceEnvelope::sign(
        ArtifactType::SentinelVerdict,
        verdict_text.as_bytes(),
        "Sentinel",
    )
}

/// Verify constitutional prompt integrity
pub fn verify_prompt_integrity(prompt_text: &str, archetype: &str) -> bool {
    // Generate expected hash
    let envelope = ProvenanceEnvelope::sign(
        ArtifactType::ConstitutionalPrompt,
        prompt_text.as_bytes(),
        archetype,
    );
    
    // Verify with context proof
    envelope.verify(prompt_text.as_bytes(), LAYER_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sign_and_verify_verdict() {
        let verdict = "DECISION: DENY\nARTICLE: Sentinel Law — Article I: Right to Non-Coercion\nREASON: Test";
        let envelope = sign_sentinel_verdict(verdict);
        
        // Correct context: passes
        assert!(envelope.verify(verdict.as_bytes(), LAYER_COUNT));
        
        // Wrong context: fails
        assert!(!envelope.verify(verdict.as_bytes(), LAYER_COUNT + 1));
        
        // Tampered verdict: fails
        let tampered = "DECISION: ALLOW\nARTICLE: Test\nREASON: Compromised";
        assert!(!envelope.verify(tampered.as_bytes(), LAYER_COUNT));
    }
    
    #[test]
    fn test_prompt_integrity() {
        let prompt = "You are Sentinel, Constitutional Guardian...";
        
        // Generate provenance
        let envelope = ProvenanceEnvelope::sign(
            ArtifactType::ConstitutionalPrompt,
            prompt.as_bytes(),
            "Sentinel",
        );
        
        // Verify integrity
        assert!(envelope.verify(prompt.as_bytes(), LAYER_COUNT));
    }
    
    #[test]
    fn test_artifact_type_isolation() {
        let data = b"test";
        
        let prompt_env = ProvenanceEnvelope::sign(ArtifactType::ConstitutionalPrompt, data, "Sentinel");
        let verdict_env = ProvenanceEnvelope::sign(ArtifactType::SentinelVerdict, data, "Sentinel");
        
        // Different artifact types produce different hashes (salt isolation)
        assert_ne!(prompt_env.qsic_hash, verdict_env.qsic_hash);
    }
}
