use crate::storage::StorageReader;
use crate::traits::types::{KeyMaterial, Locator, SecurityLevel};
use crate::traits::BigKeyError;
use digest::Digest;

/// A BigKey cryptographic key encapsulation scheme
pub trait BigKeyKem<'a, S, H>
    where S: 'a + StorageReader,
          H: 'a + Digest
{
    fn new_big_key(
        security_level: SecurityLevel,
        leakage_tolerance: f32,
        storage_scheme: &'a S,
        xof: &'a mut H,
    ) -> Self;

    fn get_key(self, locator: &Locator) -> Result<KeyMaterial, BigKeyError>;

    fn new_key(self, security_level: SecurityLevel) -> Result<(Locator, KeyMaterial), BigKeyError>;
}

pub struct BigKey<'a, S: StorageReader, H: Digest> {
    security_level: SecurityLevel,
    leakage_tolerance: f32,
    storage_scheme: &'a S,
    xof: &'a mut H,
}

impl<'a, S1, H1> BigKeyKem<'a, S1, H1> for BigKey<'a, S1, H1>
    where
        S1: 'a + StorageReader,
        H1: 'a + Digest
{
    fn new_big_key(
        security_level: SecurityLevel,
        leakage_tolerance: f32,
        storage_scheme: &'a S1,
        xof: &'a mut H1,
    ) -> Self {
        BigKey {
            security_level,
            leakage_tolerance,
            storage_scheme,
            xof,
        }
    }

    fn get_key(self, locator: &Locator) -> Result<KeyMaterial, BigKeyError> {
        unimplemented!()
    }

    fn new_key(self, security_level: SecurityLevel) -> Result<(Locator, KeyMaterial), BigKeyError> {
        unimplemented!()
    }
}
