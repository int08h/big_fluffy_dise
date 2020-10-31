use std::io;
use std::io::{Error, ErrorKind};

use crate::traits::BlockSize;

// Ensure that the total big key length is evenly divisible by the block size (no remainder)
pub(crate) fn check_key_evenly_divisible(
    block_size: BlockSize,
    key_len: u64,
) -> Result<(), io::Error> {
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
