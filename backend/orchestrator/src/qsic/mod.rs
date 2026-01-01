//! Quantum-Seeded Integrity Check (QSIC)
//! 
//! Cryptographic integrity verification using a context-locked quantum integer.
//! The security derives from Proof of Context: verifier must possess the
//! derivation formula and input parameter to recreate the quantum seed.

use sha2::{Sha256, Digest};

mod quantum_integer;
mod provenance;

pub use quantum_integer::{derive_quantum_integer, LAYER_COUNT, QUANTUM_INTEGER};
pub use provenance::{ProvenanceEnvelope, ArtifactType, sign_sentinel_verdict, verify_prompt_integrity};

/// Generate QSIC hash for data integrity verification
pub fn qsic_hash(data: &[u8], salt: &str) -> String {
    let quantum_int = derive_quantum_integer();
    
    // Convert quantum integer to fixed 32-byte representation
    let int_bytes = quantum_int.to_be_bytes();
    
    // Combine: salt || quantum_integer || data
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(&int_bytes);
    hasher.update(data);
    
    hex::encode(hasher.finalize())
}

/// Verify QSIC hash with context proof
pub fn verify_qsic(data: &[u8], salt: &str, expected_hash: &str, context_layers: u64) -> bool {
    // Proof of Context: must provide correct layer count
    if context_layers != LAYER_COUNT {
        return false;
    }
    
    // Verify derivation matches known quantum integer
    let derived = derive_quantum_integer();
    if derived != QUANTUM_INTEGER {
        return false;
    }
    
    // Generate and compare hash
    let computed_hash = qsic_hash(data, salt);
    computed_hash == expected_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_qsic_deterministic() {
        let data = b"test data";
        let salt = "AURA-Sentinel-v1";
        
        let hash1 = qsic_hash(data, salt);
        let hash2 = qsic_hash(data, salt);
        
        assert_eq!(hash1, hash2, "QSIC must be deterministic");
    }
    
    #[test]
    fn test_qsic_verification() {
        let data = b"constitutional prompt";
        let salt = "AURA-CONST";
        
        let hash = qsic_hash(data, salt);
        
        // Correct context: passes
        assert!(verify_qsic(data, salt, &hash, LAYER_COUNT));
        
        // Wrong context: fails
        assert!(!verify_qsic(data, salt, &hash, LAYER_COUNT + 1));
        
        // Tampered data: fails
        let tampered = b"constitutional prompt TAMPERED";
        assert!(!verify_qsic(tampered, salt, &hash, LAYER_COUNT));
    }
    
    #[test]
    fn test_context_locked() {
        let data = b"sentinel_system.txt";
        let salt = "AURA-PROMPT";
        let hash = qsic_hash(data, salt);
        
        // Without correct context, cannot verify
        let wrong_context = 11_894_142; // off by 1
        assert!(!verify_qsic(data, salt, &hash, wrong_context));
    }
}
