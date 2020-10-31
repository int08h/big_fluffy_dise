use std::io;
use std::io::{Error, ErrorKind, Write};

use crate::traits::BlockSize;

/// StorageMethod defines a persistent method of storing BigKey cryptographic material.
///
/// The `BlockSize` should be chosen to maximize the efficiency of random reads (seeks).
pub trait StorageReader: Sized {
    fn open(block_size: BlockSize, storage_location: &str) -> Result<Self, io::Error>;

    /// Retrieve the block at `index` writing the value in `output`.
    fn probe(&mut self, index: u64, output: &mut [u8]) -> Result<(), io::Error>;

    /// Total BigKey length in bytes
    fn big_key_length(&self) -> u64;

    /// `BlockSize` of underlying storage media
    fn block_size(&self) -> BlockSize;
}

pub trait StorageWriter: Sized + Write {
    fn new_writer(
        block_size: BlockSize,
        storage_location: &str,
        expected_size: usize,
    ) -> Result<Self, io::Error>;

    /// `BlockSize` of underlying storage media
    fn block_size(&self) -> BlockSize;

    /// Total BigKey length in bytes
    fn expected_big_key_length(&self) -> u64;

    fn finalize(&mut self) -> Result<(), io::Error>;
}
