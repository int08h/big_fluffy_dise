use crate::storage::StorageWriter;
use crate::traits::{BigKeyError, KeyMaterial};

/// A Cryptographically secure random number generator that can be used to generate BigKey material.
///
/// Deterministic implementations of `BigKeyGenerator` will use the value from `Some(seed)` to
/// establish their initial conditions.
pub trait BigKeyGenerator {
    fn generate(
        storage_method: &mut impl StorageWriter,
        seed: Option<KeyMaterial>,
        length_bytes: usize,
    ) -> Result<(), BigKeyError>;
}
