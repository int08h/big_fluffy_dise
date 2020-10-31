use std::io;

use crate::traits::KeyMaterial;

pub use self::blake3::Blake3Generator;
pub use self::shake256::Shake256Generator;

mod blake3;
mod shake256;
mod traits;

