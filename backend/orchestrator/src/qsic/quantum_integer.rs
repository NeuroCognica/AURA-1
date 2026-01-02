//! Quantum integer derivation from Diamond Tetrahedron geometry
//! 
//! The quantum integer is derived from the number of atomic bilayers (n)
//! in a 3mm diamond tetrahedron via the formula: N = floor(n³/3)
//! 
//! This provides a context-locked security parameter: an attacker must
//! possess both the layer count and the derivation formula to verify hashes.

/// Number of atomic bilayers in 3mm diamond tetrahedron
/// Derived from: n = L√2 / a, where L=3mm, a=3.567Å
pub const LAYER_COUNT: u64 = 11_894_143;

/// Quantum integer: total atoms in the theoretical crystal
/// Formula: N = floor(n³/3)
/// Value: 560,890,665,052,636,047,402 (21 digits)
pub const QUANTUM_INTEGER: u128 = 560_890_665_052_636_047_402;

/// Derive quantum integer from layer count (Proof of Context)
pub fn derive_quantum_integer() -> u128 {
    derive_from_layers(LAYER_COUNT)
}

/// Core derivation formula: N = floor(n³/3)
pub fn derive_from_layers(n: u64) -> u128 {
    let n_u128 = n as u128;
    let n_cubed = n_u128.pow(3);
    n_cubed / 3
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quantum_integer_derivation() {
        let derived = derive_quantum_integer();
        assert_eq!(derived, QUANTUM_INTEGER, "Derivation must match constant");
    }
    
    #[test]
    fn test_layer_count_formula() {
        // Verify the exact calculation from the blueprint
        let n = LAYER_COUNT;
        let n_cubed: u128 = (n as u128).pow(3);
        let expected = n_cubed / 3;
        
        assert_eq!(expected, QUANTUM_INTEGER);
    }
    
    #[test]
    fn test_context_sensitivity() {
        // Off-by-one produces completely different integer
        let correct = derive_from_layers(LAYER_COUNT);
        let wrong = derive_from_layers(LAYER_COUNT + 1);
        
        assert_ne!(correct, wrong);
        assert_eq!(correct, QUANTUM_INTEGER);
    }
}
