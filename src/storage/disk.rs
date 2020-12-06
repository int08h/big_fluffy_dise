//! StorageMethod defines how BigKeys are read from permanent media.

use std::fs::{File};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::storage::traits::StorageReader;
use crate::storage::util::check_key_evenly_divisible;
use crate::storage::StorageWriter;
use crate::traits::types::BlockSize;
use crate::traits::BigKeyError;

/// Stores BigKey material in a file on a conventional filesystem. Assumes underlying storage
/// medium provides efficient random access to the big key contents (think NVMe or SSD, not HDD).
///
/// Probes are made one-at-a-time, reading `BlockSize` bytes each `probe()`
pub struct DiskStorage {
    block_size: BlockSize,
    big_key_length: u64,
    big_key_file: File,
}

// Differentiate which trait DiskStorage is implementing
enum IoMode {
    READ,
    WRITE,
}

impl DiskStorage {
    fn new(
        block_size: BlockSize,
        storage_location: &str,
        expected_size: Option<usize>,
        mode: IoMode,
    ) -> Result<DiskStorage, BigKeyError> {
        let big_key_file: File;
        let big_key_length: u64;

        match mode {
            IoMode::READ => {
                big_key_file = File::open(storage_location)?;
                big_key_length = big_key_file.metadata()?.len();
            }
            IoMode::WRITE => {
                big_key_file = File::create(storage_location)?;
                big_key_length = expected_size.unwrap() as u64;
            }
        }

        if let Err(e) = check_key_evenly_divisible(block_size, big_key_length) {
            return Err(e);
        }

        Ok(DiskStorage {
            block_size,
            big_key_length,
            big_key_file,
        })
    }
}

impl StorageReader for DiskStorage {
    fn open(block_size: BlockSize, storage_location: &str) -> Result<DiskStorage, BigKeyError> {
        DiskStorage::new(block_size, storage_location, None, IoMode::READ)
    }

    fn probe(&mut self, index: u64, output: &mut [u8]) -> Result<(), BigKeyError> {
        if output.len() != self.block_size.byte_len {
            return Err(BigKeyError::ProbeBufferNotEqBlockSize {
                out_buf_len: output.len(),
                block_len: self.block_size.byte_len,
            });
        }

        let offset = index * self.block_size.byte_len as u64;

        if offset + self.block_size.byte_len as u64 > self.big_key_length {
            return Err(BigKeyError::ProbeOffsetOutOfBounds {
                end_of_key: self.big_key_length as usize,
                offset: offset as usize,
                probe_len: self.block_size.byte_len,
            });
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

impl StorageWriter for DiskStorage {
    fn new_writer(
        block_size: BlockSize,
        storage_location: &str,
        expected_size: usize,
    ) -> Result<Self, BigKeyError> {
        if expected_size < block_size.byte_len {
            return Err(BigKeyError::OutputLengthTooShort {
                out_len: expected_size,
                min_len: block_size.byte_len,
            });
        }

        DiskStorage::new(
            block_size,
            storage_location,
            Some(expected_size),
            IoMode::WRITE,
        )
    }

    fn block_size(&self) -> BlockSize {
        self.block_size
    }

    fn expected_big_key_length(&self) -> u64 {
        self.big_key_length
    }

    fn finalize(&mut self) -> Result<(), BigKeyError> {
        self.flush()?;

        let metadata = self.big_key_file.metadata()?;

        if metadata.len() != self.big_key_length {
            return Err(BigKeyError::FailedToWriteBigKey {
                expected_len: self.big_key_length as usize,
                wrote_len: metadata.len() as usize,
            });
        } else {
            Ok(())
        }
    }
}

impl Write for DiskStorage {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.big_key_file.write(buf)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.big_key_file.flush()
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{Error, ErrorKind, Write};

    use crate::storage::disk::DiskStorage;
    use crate::storage::{StorageReader, StorageWriter};
    use crate::traits::{BigKeyError, BlockSize, BLOCKS, BLOCK_32};
    use crate::storage::tempfile::tempfile;
    use std::io;

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
            Err(e) => panic!("consistent size didn't pass {}", e),
        }
    }

    #[test]
    fn open_fails_if_file_doesnt_exist() {
        let tmp = tempfile();
        match DiskStorage::open(BLOCK_32, tmp.to_str()) {
            Err(BigKeyError::IoError(e)) => assert_eq!(e.kind(), ErrorKind::NotFound),
            _ => panic!("open() should have failed as {:?} didn't exist", tmp),
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
        for block_size in BLOCKS[1..].iter() {
            match DiskStorage::open(*block_size, tmp.to_str()) {
                Err(BigKeyError::KeyLengthIndivisible { .. }) => {}
                _ => panic!(
                    "expected {:?} to fail due to uneven key file size",
                    block_size
                ),
            }
        }
    }

    #[test]
    fn opened_file_reports_correct_size() {
        let filler = b"0123456789abcdef";

        for block_size in BLOCKS.iter() {
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
        for block_size in BLOCKS.iter() {
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
        for block_size in BLOCKS.iter() {
            let tmp = tempfile();
            let data = [0x88].repeat(block_size.byte_len);
            {
                let mut ofile = File::create(tmp.as_path()).unwrap();
                ofile.write_all(&data).unwrap();
            }

            let mut storage = DiskStorage::open(*block_size, tmp.to_str()).unwrap();
            let mut unused = [0x00].repeat(block_size.byte_len);

            match storage.probe(1, &mut unused) {
                Err(BigKeyError::ProbeOffsetOutOfBounds { .. }) => {}
                _ => panic!("expected an index out of bounds error"),
            }
        }
    }

    #[test]
    fn probe_buffer_length_not_same_as_block_length_fails() {
        for block_size in BLOCKS.iter() {
            let tmp = tempfile();
            let data = [0x99].repeat(block_size.byte_len);
            {
                let mut ofile = File::create(tmp.as_path()).unwrap();
                ofile.write_all(&data).unwrap();
            }

            let mut storage = DiskStorage::open(*block_size, tmp.to_str()).unwrap();
            let mut buf = [0x00].repeat(block_size.byte_len - 1);

            match storage.probe(1, &mut buf) {
                Err(BigKeyError::ProbeBufferNotEqBlockSize { .. }) => {}
                _ => panic!("expected output != block length"),
            }
        }
    }

    #[test]
    fn expected_size_must_be_ge_block_size() {
        for block in BLOCKS.iter() {
            match DiskStorage::new_writer(*block, "/", 0) {
                Err(BigKeyError::OutputLengthTooShort { .. }) => {}
                _ => panic!("expected a zero length key to be rejected"),
            }
        }
    }
} // mod test
