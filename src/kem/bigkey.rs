use digest::ExtendableOutput;

use crate::storage::StorageReader;
use crate::traits::types::{KeyMaterial, Locator, SecurityLevel};
use crate::traits::BigKeyError;

/// A BigKey cryptographic key encapsulation scheme
pub trait BigKeyKem: Sized {
    fn new_big_key(
        security_level: SecurityLevel,
        leakage_tolerance: f32,
        storage_scheme: impl StorageReader,
        xof: impl ExtendableOutput,
    ) -> Self;

    fn get_key(self, locator: &Locator) -> Result<KeyMaterial, BigKeyError>;

    fn new_key(self, security_level: SecurityLevel) -> Result<(Locator, KeyMaterial), BigKeyError>;
}
