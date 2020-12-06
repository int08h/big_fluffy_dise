use std::str::FromStr;

use big_fluffy_dise::generation::{BigKeyGenerator, Shake256Generator};
use big_fluffy_dise::storage::{DiskStorage, StorageWriter};
use big_fluffy_dise::traits::BLOCK_4K;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("usage: {} LEN_MiB OUTFILE", args[0]);
        return;
    }

    let seed = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_vec();
    let size_bytes = u64::from_str(&args[1]).expect("invalid length");
    let key_file = &args[2];

    let mut writer = DiskStorage::new_writer(BLOCK_4K, key_file, size_bytes as usize).unwrap();
    Shake256Generator::generate(
        &mut writer,
        Some(seed.into_boxed_slice()),
        size_bytes as usize,
    ).unwrap();

    println!("Done.");
}
