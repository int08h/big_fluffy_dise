//! Temporary file helper utility for tests

use std::fs::Metadata;
use std::io::Error;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const TEMP_PREFIX: &str = "big_fluffy_dise_";

#[cfg(test)]
pub(crate) fn tempfile() -> TempFile {
    let mut tmp = std::env::temp_dir();
    tmp.push(format!("{}{}", TEMP_PREFIX, timestamp()));
    TempFile { pb: tmp }
}

#[derive(Debug)]
pub(crate) struct TempFile {
    pb: PathBuf,
}

impl TempFile {
    pub fn as_path(&self) -> &Path {
        self.pb.as_path()
    }

    pub fn to_str(&self) -> &str {
        self.pb.as_path().to_str().unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        match self.pb.metadata() {
            Ok(metadata) => println!("removing {} byte file {:?}", metadata.len(), self.pb),
            Err(_) => println!("removing file {:?}", self.pb),
        }

        match std::fs::remove_file(self.pb.as_path()) {
            _ => {}
        }
    }
}

fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
