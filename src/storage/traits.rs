use std::io;
use std::io::Write;

use crate::traits::{BigKeyError, BlockSize};

/// StorageMethod defines a persistent method of storing and reading BigKey cryptographic material.
///
/// The `probe()` method implements a single large-alphabet probe into the BigKey.
///
/// The `BlockSize` should be chosen to maximize the efficiency of random reads (seeks).
pub trait StorageReader: Sized {
    fn open(block_size: BlockSize, storage_location: &str) -> Result<Self, BigKeyError>;

    /// Retrieve the block at `index` writing the value in `output`.
    fn probe(&mut self, index: u64, output: &mut [u8]) -> Result<(), BigKeyError>;

    /// Total BigKey length in bytes
    fn big_key_length(&self) -> u64;

    /// `BlockSize` of underlying storage media
    fn block_size(&self) -> BlockSize;
}

/// StrageWriter generates a new BigKey
pub trait StorageWriter: Sized + Write {
    fn new_writer(
        block_size: BlockSize,
        storage_location: &str,
        expected_size: usize,
    ) -> Result<Self, BigKeyError>;

    /// `BlockSize` of underlying storage media
    fn block_size(&self) -> BlockSize;

    /// Total BigKey length in bytes
    fn expected_big_key_length(&self) -> u64;

    fn finalize(&mut self) -> Result<(), BigKeyError>;
}
