//! StorageMethod defines how BigKeys are read from permanent media.

use std::fs::{File, Metadata};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::traits::storage::check_key_evenly_divisible;
use crate::traits::StorageMethod;
use crate::traits::types::BlockSize;

/// Stores BigKey material in a file on a conventional filesystem. Assumes underlying storage
/// medium provides efficient random access to the big key contents (think NVMe or SSD, not HDD).
///
/// Probes are made one-at-a-time, reading `BlockSize` bytes each `probe()`
pub struct DiskStorage {
    block_size: BlockSize,
    big_key_length: u64,
    big_key_file: File,
}

impl StorageMethod for DiskStorage {
    fn open(block_size: BlockSize, storage_location: &str) -> Result<DiskStorage, io::Error> {
        let big_key_file = File::open(storage_location)?;
        let big_key_length = big_key_file.metadata()?.len();

        if let Err(e) = check_key_evenly_divisible(block_size, big_key_length) {
            return Err(e);
        }

        Ok(DiskStorage {
            block_size,
            big_key_length,
            big_key_file,
        })
    }

    fn probe(&mut self, index: u64, output: &mut [u8]) -> Result<(), Error> {
        if output.len() != self.block_size.byte_len {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "output buffer != block length",
            ));
        }

        let offset = index * self.block_size.byte_len as u64;
        if offset + self.block_size.byte_len as u64 > self.big_key_length {
            return Err(Error::new(ErrorKind::InvalidInput, "out of bounds"));
        }

        self.big_key_file.seek(SeekFrom::Start(offset))?;
        self.big_key_file.read_exact(output)?;

        Ok(())
    }

    fn big_key_length(&self) -> u64 {
        self.big_key_length
    }

    fn block_size(&self) -> BlockSize {
        self.block_size
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{Error, ErrorKind, Write};

    use crate::storage::disk::DiskStorage;
    use crate::traits::{BLOCK_32, BLOCK_4096, BLOCK_512, BLOCK_64, BLOCK_8, BlockSize, StorageMethod};

    static BLOCKS: &[BlockSize] = &[BLOCK_8, BLOCK_32, BLOCK_64, BLOCK_512, BLOCK_4096];

    #[test]
    fn open_succeeds_when_size_matches() {
        let tmp = tempfile();
        {
            let mut ofile = File::create(tmp.as_path()).unwrap();
            let data = [0u8; 2048];
            ofile.write_all(&data).unwrap();
        }

        match DiskStorage::open(BLOCK_32, tmp.to_str()) {
            Ok(storage) => assert_eq!(storage.big_key_length(), 2048),
            Err(e) => panic!("consistent size didn't pass"),
        }
    }

    #[test]
    fn open_fails_if_file_doesnt_exist() {
        let tmp = tempfile();
        match DiskStorage::open(BLOCK_32, tmp.to_str()) {
            Ok(_) => panic!("open() should have failed as {:?} didn't exist", tmp),
            Err(e) => assert_eq!(e.kind(), ErrorKind::NotFound),
        }
    }

    #[test]
    fn open_fails_when_key_file_length_not_evenly_divisible_by_block() {
        let tmp = tempfile();
        {
            let mut ofile = File::create(tmp.as_path()).unwrap();
            let data = [0u8; 4097];
            ofile.write_all(&data).unwrap();
        }

        // Skip BLOCK_8 since it's a single byte and by definition evenly divides everything
        for block_size in vec![BLOCK_32, BLOCK_64, BLOCK_512, BLOCK_4096] {
            match DiskStorage::open(block_size, tmp.to_str()) {
                Ok(_) => panic!(
                    "expected {:?} to fail due to uneven key file size",
                    block_size
                ),
                Err(e) => {
                    assert_eq!(e.kind(), ErrorKind::InvalidInput);
                    assert!(e.to_string().contains("does not evenly divide"));
                }
            }
        }
    }

    #[test]
    fn opened_file_reports_correct_size() {
        let filler = b"0123456789abcdef";

        for block_size in BLOCKS {
            let tmp = tempfile();
            let data = filler.repeat(block_size.byte_len);
            {
                let mut ofile = File::create(tmp.as_path()).unwrap();
                ofile.write_all(&data).unwrap();
            }

            let storage = DiskStorage::open(*block_size, tmp.to_str()).unwrap();
            assert_eq!(
                storage.big_key_length() as usize,
                filler.len() as usize * block_size.byte_len
            );
        }
    }

    #[test]
    fn successfully_read_blocks_in_random_order() {
        let tmp = tempfile();
        for block_size in BLOCKS {
            let data1 = [0x11].repeat(block_size.byte_len);
            let data2 = [0x22].repeat(block_size.byte_len);
            let data3 = [0x33].repeat(block_size.byte_len);
            {
                let mut ofile = File::create(tmp.as_path()).unwrap();
                ofile.write_all(&data1).unwrap();
                ofile.write_all(&data2).unwrap();
                ofile.write_all(&data3).unwrap();
            }

            let mut storage = DiskStorage::open(*block_size, tmp.to_str()).unwrap();
            let mut buf = [0x00].repeat(block_size.byte_len);

            // Read 2nd block of 0x22
            storage.probe(1, &mut buf).unwrap();
            assert_eq!(buf, data2);

            // Read 1st block of 0x11
            storage.probe(0, &mut buf).unwrap();
            assert_eq!(buf, data1);

            // Read 3rd block of 0x33
            storage.probe(2, &mut buf).unwrap();
            assert_eq!(buf, data3);
        }
    }

    #[test]
    fn attempt_to_read_past_end_of_file_fails() {
        for block_size in BLOCKS {
            let tmp = tempfile();
            let data = [0x88].repeat(block_size.byte_len);
            {
                let mut ofile = File::create(tmp.as_path()).unwrap();
                ofile.write_all(&data).unwrap();
            }

            let mut storage = DiskStorage::open(*block_size, tmp.to_str()).unwrap();
            let mut unused = [0x00].repeat(block_size.byte_len);

            match storage.probe(1, &mut unused) {
                Ok(_) => panic!("expected an index out of bounds error"),
                Err(e) => {
                    assert_eq!(e.kind(), ErrorKind::InvalidInput);
                    assert!(e.to_string().contains("out of bounds"));
                }
            }
        }
    }
} // mod test
