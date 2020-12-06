use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BigKeyError {
    #[error("block length {block_len} does not evenly divide key length {key_len}")]
    KeyLengthIndivisible { block_len: usize, key_len: usize },

    #[error("seed too short; provided {seed_len} bytes < required {req_len} bytes")]
    SeedTooShort { seed_len: usize, req_len: usize },

    #[error("requested output length too long; {out_len} > max {max_len}")]
    OutputLengthTooLong { out_len: usize, max_len: usize },

    #[error("requested output length too short (less than a block); {out_len} < min {min_len}")]
    OutputLengthTooShort { out_len: usize, min_len: usize },

    #[error("did not write all bytes of BigKey; wrote {wrote_len} < expected {expected_len}")]
    FailedToWriteBigKey {
        expected_len: usize,
        wrote_len: usize,
    },

    #[error("probe request out of bounds; offset {offset} + probe {probe_len} > end of key {end_of_key}")]
    ProbeOffsetOutOfBounds {
        end_of_key: usize,
        offset: usize,
        probe_len: usize,
    },

    #[error("output buffer {out_buf_len} != block size {block_len}")]
    ProbeBufferNotEqBlockSize {
        out_buf_len: usize,
        block_len: usize,
    },

    #[error("io error")]
    IoError(#[from] io::Error),
}
