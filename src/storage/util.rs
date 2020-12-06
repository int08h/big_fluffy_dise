use crate::traits::{BigKeyError, BlockSize};

// Ensure that the total big key length is evenly divisible by the block size (no remainder)
pub(crate) fn check_key_evenly_divisible(
    block_size: BlockSize,
    key_len: u64,
) -> Result<(), BigKeyError> {
    if (key_len % block_size.byte_len as u64) != 0 {
        Err(BigKeyError::KeyLengthIndivisible {
            block_len: block_size.byte_len,
            key_len: key_len as usize,
        })
    } else {
        Ok(())
    }
}
