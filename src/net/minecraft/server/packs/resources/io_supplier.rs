//! net/minecraft/server/packs/resources/IoSupplier.java
//!
//! What a pack hands out for a resource: `IoSupplier.create(path)`, a file opened when it's read.
//! A stream is read whole, which is all its callers do, so `get` is the file's bytes.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IoSupplier {
    path: PathBuf,
}

impl IoSupplier {
    pub fn create(path: &Path) -> IoSupplier {
        IoSupplier {
            path: path.to_path_buf(),
        }
    }

    pub fn get(&self) -> std::io::Result<Vec<u8>> {
        std::fs::read(&self.path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
