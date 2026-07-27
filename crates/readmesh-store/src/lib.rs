pub mod blob;
pub mod db;

use readmesh_core::chapter::Chapter;
use readmesh_core::id::{ChapterId, NodeId, NovelId};
use readmesh_core::novel::Novel;
use readmesh_core::progress::ReadingProgress;
use readmesh_core::source::PluginManifest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("blob store error: {0}")]
    Blob(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// Main store handle combining metadata DB and blob storage.
pub struct Store {
    db: db::Database,
    blobs: blob::BlobStore,
}

impl Store {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let db_path = path.join("readmesh.db");
        let blob_path = path.join("blobs");
        Ok(Self {
            db: db::Database::open(&db_path)?,
            blobs: blob::BlobStore::new(&blob_path)?,
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        Ok(Self {
            db: db::Database::open_in_memory()?,
            blobs: blob::BlobStore::new(std::path::Path::new("/tmp/readmesh-blobs"))?,
        })
    }

    // --- Novels ---

    pub fn insert_novel(&self, novel: &Novel) -> Result<()> {
        self.db.insert_novel(novel)
    }

    pub fn get_novel(&self, id: &NovelId) -> Result<Option<Novel>> {
        self.db.get_novel(id)
    }

    pub fn list_novels(&self) -> Result<Vec<Novel>> {
        self.db.list_novels()
    }

    pub fn delete_novel(&self, id: &NovelId) -> Result<()> {
        self.db.delete_novel(id)
    }

    // --- Chapters ---

    pub fn insert_chapter(&self, chapter: &Chapter) -> Result<()> {
        self.db.insert_chapter(chapter)
    }

    pub fn get_chapter(&self, id: &ChapterId) -> Result<Option<Chapter>> {
        self.db.get_chapter(id)
    }

    pub fn list_chapters_for_novel(&self, novel_id: &NovelId) -> Result<Vec<Chapter>> {
        self.db.list_chapters_for_novel(novel_id)
    }

    // --- Reading Progress ---

    pub fn set_progress(&self, progress: &ReadingProgress) -> Result<()> {
        self.db.set_progress(progress)
    }

    pub fn get_progress(
        &self,
        novel_id: &NovelId,
        device_id: &NodeId,
    ) -> Result<Option<ReadingProgress>> {
        self.db.get_progress(novel_id, device_id)
    }

    // --- Plugins ---

    pub fn insert_plugin(&self, manifest: &PluginManifest) -> Result<()> {
        self.db.insert_plugin(manifest)
    }

    pub fn list_plugins(&self) -> Result<Vec<PluginManifest>> {
        self.db.list_plugins()
    }

    // --- Blob store ---

    pub async fn put_blob(&self, hash: &blake3::Hash, data: &[u8]) -> Result<()> {
        self.blobs.put(hash, data).await
    }

    pub async fn get_blob(&self, hash: &blake3::Hash) -> Result<Option<Vec<u8>>> {
        self.blobs.get(hash).await
    }
}
