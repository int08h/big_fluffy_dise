/// Cryptographic security level
#[derive(Debug, Copy, Clone)]
pub enum SecurityLevel {
    /// 128-bit security level
    Bits128 = 128,

    /// 256-bit security level
    Bits256 = 256,
}

/// Native unit of capacity for a given StorageMethod.
#[derive(Debug, Copy, Clone)]
pub struct BlockSize {
    pub bit_len: usize,
    pub byte_len: usize,
}

pub static BLOCK_8: BlockSize = BlockSize {
    bit_len: 8,
    byte_len: 1,
};
pub static BLOCK_32: BlockSize = BlockSize {
    bit_len: 32,
    byte_len: 4,
};
pub static BLOCK_64: BlockSize = BlockSize {
    bit_len: 64,
    byte_len: 8,
};
pub static BLOCK_512: BlockSize = BlockSize {
    bit_len: 512,
    byte_len: 64,
};
pub static BLOCK_4096: BlockSize = BlockSize {
    bit_len: 4096,
    byte_len: 512,
};
pub static BLOCK_4K: BlockSize = BlockSize {
    bit_len: 32768,
    byte_len: 4096,
};

pub static BLOCKS: [BlockSize; 6] = [
    BLOCK_8, BLOCK_32, BLOCK_64, BLOCK_512, BLOCK_4096, BLOCK_4K
];

/// An opaque locator into a BigKey
pub type Locator = Box<[u8]>;

/// Sensitive/secret cryptographic information; treat with caution!
pub type KeyMaterial = Box<[u8]>;
