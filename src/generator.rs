//! Generate the contents of a BigKey
use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read, Write};
use std::num::NonZeroU32;

use digest::{Digest, ExtendableOutput, Update, XofReader};
use rand_core::{CryptoRng, RngCore, SeedableRng};
use sha3::{Sha3XofReader, Shake256};

use crate::types::KeyMaterial;

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

pub struct Sha3ShakeGenerator {
    xof: Sha3XofReader,
}

impl Sha3ShakeGenerator {
    fn from_seed(seed: &[u8]) -> Result<Self, io::Error> {
        if seed.len() < 32 {
            return Err(io::Error::new(ErrorKind::InvalidInput, "seed too short"));
        }

        let mut hash = Shake256::default();
        hash.update(seed);

        Ok(Sha3ShakeGenerator {
            xof: hash.finalize_xof(),
        })
    }
}

impl RngCore for Sha3ShakeGenerator {
    fn next_u32(&mut self) -> u32 {
        let mut dest = [0u8; 4];
        u32::from_le_bytes(dest)
    }

    fn next_u64(&mut self) -> u64 {
        let mut dest = [0u8; 8];
        u64::from_le_bytes(dest)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.xof.read_exact(dest).unwrap()
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::ErrorKind;

    use rand_core::RngCore;

    use crate::generator::Sha3ShakeGenerator;

    #[test]
    fn shake_256_known_answer_test() {
        let seed = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let mut gen = Sha3ShakeGenerator::from_seed(seed).unwrap();
        let expected = [0x5a, 0x81, 0x82, 0xc1, 0xe3, 0x72, 0x89, 0xf4];

        let mut buf = [0u8; 8];
        gen.fill_bytes(buf.as_mut());
        assert_eq!(buf, expected);

        // Second fill must be different
        gen.fill_bytes(buf.as_mut());
        assert_ne!(buf, expected);
    }

    #[test]
    fn shake_256_short_seed_fails() {
        let seed = b"too short";
        if let Err(e) = Sha3ShakeGenerator::from_seed(seed) {
            assert_eq!(e.kind(), ErrorKind::InvalidInput);
            assert!(e.to_string().contains("too short"));
        } else {
            panic!("expected seed too short, but didn't get it");
        }
    }
} // mod test
