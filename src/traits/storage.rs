use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, SeekFrom};

use crate::traits::types::BlockSize;

/// StorageMethod defines a persistent method of storing BigKey cryptographic material.
pub trait StorageMethod: Sized {
    fn open(block_size: BlockSize, storage_location: &str) -> Result<Self, io::Error>;

    /// Retrieve the block at `index` writing the value in `output`
    fn probe(&mut self, index: u64, output: &mut [u8]) -> Result<(), io::Error>;

    /// Key length in bytes
    fn big_key_length(&self) -> u64;

    /// Storage media block size in bytes
    fn block_size(&self) -> BlockSize;
}


// Ensure that the total big key length is evenly divisible by the block size (no remainder)
pub(crate) fn check_key_evenly_divisible(block_size: BlockSize, key_len: u64) -> Result<(), io::Error> {
    if (key_len % block_size.byte_len as u64) != 0 {
        let msg = format!(
            "{:?} does not evenly divide key length {}",
            block_size, key_len
        );
        Err(Error::new(ErrorKind::InvalidInput, msg))
    } else {
        Ok(())
    }
}

