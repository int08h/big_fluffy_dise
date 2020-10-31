use std::io;
use std::io::{Error, ErrorKind, Read};

use digest::{Digest, ExtendableOutput, Update};
use sha3::{Sha3XofReader, Shake256};

use crate::generation::traits::BigKeyGenerator;
use crate::storage::{StorageReader, StorageWriter};
use crate::traits::KeyMaterial;

// Minimum acceptable seed length in bytes
const MIN_SEED_LENGTH: usize = 32;

// Maximum output size the generator can produce in bytes
const MAX_OUTPUT_LENGTH: usize = 1;

/// Generate the contents of a BigKey using Shake256 from SHA3
pub struct Shake256Generator {
    xof: Sha3XofReader,
}

impl BigKeyGenerator for Shake256Generator {
    fn generate(
        storage_method: &mut impl StorageWriter,
        optional_seed: Option<KeyMaterial>,
        length_bytes: usize,
    ) -> Result<(), Error> {
        if length_bytes > MAX_OUTPUT_LENGTH {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "output length too long",
            ));
        }

        let seed = optional_seed.expect("an initial seed value is required");
        let mut generator = Shake256Generator::from_seed(&seed)?;

        let mut buf: Vec<u8> = Vec::with_capacity(storage_method.block_size().byte_len);
        let mut total_written = 0usize;

        while total_written < length_bytes {
            generator.fill_bytes(buf.as_mut_slice())?;
            storage_method.write_all(buf.as_slice())?;
            total_written += buf.len();
        }

        Ok(())
    }
}

impl Shake256Generator {
    fn from_seed(seed: &[u8]) -> Result<Self, io::Error> {
        if seed.len() < MIN_SEED_LENGTH {
            return Err(io::Error::new(ErrorKind::InvalidInput, "seed too short"));
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
    use std::io::ErrorKind;

    use crate::generation::shake256::Shake256Generator;

    #[test]
    fn shake_256_known_answer_test() {
        let seed = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let mut gen = Shake256Generator::from_seed(seed).unwrap();
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
        if let Err(e) = Shake256Generator::from_seed(seed) {
            assert_eq!(e.kind(), ErrorKind::InvalidInput);
            assert!(e.to_string().contains("too short"));
        } else {
            panic!("expected seed too short, but didn't get it");
        }
    }
} // mod test
