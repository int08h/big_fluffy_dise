/// Cryptographic security level
#[derive(Debug, Copy, Clone)]
pub enum SecurityLevel {
    /// 128-bit security level
    Bits128 = 128,

    /// 256-bit security level
    Bits256 = 256,
}

/// Native and atomic unit of storage
#[derive(Debug, Copy, Clone)]
pub struct BlockSize {
    pub bit_len: usize,
    pub byte_len: usize,
}

pub const BLOCK_8: BlockSize = BlockSize {
    bit_len: 8,
    byte_len: 1,
};
pub const BLOCK_32: BlockSize = BlockSize {
    bit_len: 32,
    byte_len: 4,
};
pub const BLOCK_64: BlockSize = BlockSize {
    bit_len: 64,
    byte_len: 8,
};
pub const BLOCK_512: BlockSize = BlockSize {
    bit_len: 512,
    byte_len: 64,
};
pub const BLOCK_4096: BlockSize = BlockSize {
    bit_len: 4096,
    byte_len: 512,
};

/// An opaque locator into a BigKey
pub type Locator = Box<[u8]>;

/// Sensitive/secret cryptographic information; treat with caution!
pub type KeyMaterial = Box<[u8]>;

// pub type XOF<T:  Digest + ExtendableOutput>
