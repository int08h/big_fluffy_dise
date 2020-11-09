use std::io::{Error, Read};

use digest::{Digest, ExtendableOutput, Update};
use sha3::{Sha3XofReader, Shake256};

use crate::generation::traits::BigKeyGenerator;
use crate::storage::{StorageReader, StorageWriter};
use crate::traits::{BigKeyError, KeyMaterial};

// Minimum acceptable seed length in bytes
const MIN_SEED_LENGTH: usize = 32;

// Shake256 has no restriction on output length. We'll arbitrarily limit it at 2^64 which would
// be a very large BigKey indeed. See the SHA3 standard for details:
//   https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf#page=31
const MAX_OUTPUT_LENGTH: usize = u64::max_value() as usize;

/// Generate the contents of a BigKey using Shake256 from SHA3
pub struct Shake256Generator {
    xof: Sha3XofReader,
}

impl BigKeyGenerator for Shake256Generator {
    fn generate(
        storage_method: &mut impl StorageWriter,
        optional_seed: Option<KeyMaterial>,
        length_bytes: usize,
    ) -> Result<(), BigKeyError> {
        if length_bytes > MAX_OUTPUT_LENGTH {
            return Err(BigKeyError::OutputLengthTooLong {
                out_len: length_bytes,
                max_len: MAX_OUTPUT_LENGTH,
            });
        }

        let seed = optional_seed.unwrap();
        let mut generator = Shake256Generator::from_seed(&seed)?;

        let mut buf = vec![0u8; storage_method.block_size().byte_len];
        let mut total_written = 0usize;

        while total_written < length_bytes {
            generator.fill_bytes(buf.as_mut_slice())?;
            storage_method.write_all(&buf)?;
            total_written += buf.capacity();
        }

        storage_method.finalize()?;

        Ok(())
    }
}

impl Shake256Generator {
    fn from_seed(seed: &[u8]) -> Result<Self, BigKeyError> {
        if seed.len() < MIN_SEED_LENGTH {
            return Err(BigKeyError::SeedTooShort {
                seed_len: seed.len(),
                req_len: MIN_SEED_LENGTH,
            });
        }

        let mut hash = Shake256::default();
        hash.update(seed);

        Ok(Shake256Generator {
            xof: hash.finalize_xof(),
        })
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.xof.read_exact(dest)
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{Error, ErrorKind, Read};

    use crate::generation::shake256::Shake256Generator;
    use crate::generation::traits::BigKeyGenerator;
    use crate::storage::{DiskStorage, StorageWriter};
    use crate::traits::{BigKeyError, BLOCK_8};
    use crate::util::tempfile::tempfile;

    #[test]
    fn shake_256_known_answer_test() {
        let seed = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let mut gen = Shake256Generator::from_seed(seed).unwrap();
        let expected = [0x5a, 0x81, 0x82, 0xc1, 0xe3, 0x72, 0x89, 0xf4];

        let mut buf = [0u8; 8];
        gen.fill_bytes(buf.as_mut()).unwrap();
        assert_eq!(buf, expected);

        // Second fill must be different
        gen.fill_bytes(buf.as_mut()).unwrap();
        assert_ne!(buf, expected);
    }

    #[test]
    fn shake_256_short_seed_fails() {
        let seed = b"01234".to_vec();
        match Shake256Generator::from_seed(&seed) {
            Err(BigKeyError::SeedTooShort { .. }) => {}
            _ => panic!("expected seed too short, but didn't get it"),
        }
    }

    #[test]
    fn generate_known_answer_test() {
        let seed = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_vec();
        let expected = [0x5a, 0x81, 0x82, 0xc1, 0xe3, 0x72, 0x89, 0xf4];

        let tmp = tempfile();
        let mut storage = DiskStorage::new_writer(BLOCK_8, tmp.to_str(), 8).unwrap();

        Shake256Generator::generate(&mut storage, Some(seed.into_boxed_slice()), 8).unwrap();

        let mut infile = File::open(tmp.as_path()).unwrap();
        let mut buf = [0u8; 8];
        infile.read_exact(buf.as_mut()).unwrap();

        assert_eq!(buf, expected);
    }
} // mod test
