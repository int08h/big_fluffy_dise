use std::io;

use digest::ExtendableOutput;

use crate::storage::StorageReader;
use crate::traits::types::{KeyMaterial, Locator, SecurityLevel};

/// A BigKey cryptographic key encapsulation scheme
pub trait BigKey: Sized {
    fn new_big_key(
        security_level: SecurityLevel,
        leakage_tolerance: f32,
        storage_scheme: impl StorageReader,
        xof: impl ExtendableOutput,
    ) -> Self;

    fn get_key(self, locator: &Locator) -> Result<KeyMaterial, io::Error>;

    fn new_key(self, security_level: SecurityLevel) -> Result<(Locator, KeyMaterial), io::Error>;
}
