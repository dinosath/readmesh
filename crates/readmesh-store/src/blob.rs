use std::path::{Path, PathBuf};

use super::StoreError;

/// Filesystem-based blob store keyed by BLAKE3 hash.
pub struct BlobStore {
    root: PathBuf,
}

impl BlobStore {
    pub fn new(root: &Path) -> Result<Self, StoreError> {
        std::fs::create_dir_all(root).map_err(|e| StoreError::Blob(e.to_string()))?;
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    fn path_for(&self, hash: &blake3::Hash) -> PathBuf {
        let hex = hash.to_hex().to_string();
        let dir = self.root.join(&hex[..2]).join(&hex[2..4]);
        dir.join(&hex)
    }

    pub async fn get(&self, hash: &blake3::Hash) -> Result<Option<Vec<u8>>, StoreError> {
        let path = self.path_for(hash);
        if !path.exists() {
            return Ok(None);
        }
        std::fs::read(&path)
            .map(Some)
            .map_err(|e| StoreError::Blob(e.to_string()))
    }

    pub async fn put(&self, hash: &blake3::Hash, data: &[u8]) -> Result<(), StoreError> {
        let path = self.path_for(hash);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| StoreError::Blob(e.to_string()))?;
        }
        std::fs::write(&path, data).map_err(|e| StoreError::Blob(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, hash: &blake3::Hash) -> Result<(), StoreError> {
        let path = self.path_for(hash);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| StoreError::Blob(e.to_string()))?;
        }
        Ok(())
    }
}
