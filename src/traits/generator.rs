use std::io;

use rand_core::{CryptoRng, RngCore};

use crate::traits::types::KeyMaterial;

/// A Cryptographically secure seedable random number generator that can be used to generate
/// BigKey material.
pub trait BigKeyGenerator {
    fn generate<RngT: RngCore + CryptoRng>(
        storage_location: &str,
        seed: &KeyMaterial,
        big_key_length: u64,
        rng: RngT,
    ) -> Result<(), io::Error>;
}

