use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use ln_core::chapter::Chapter;
use ln_core::id::{ChapterId, NodeId, NovelId};
use ln_core::novel::{Novel, NovelStatus};
use ln_core::progress::ReadingProgress;
use ln_core::source::PluginManifest;

use super::StoreError;

pub struct Database {
    conn: Mutex<Connection>,
}

fn hash_as_blob(hash: &blake3::Hash) -> &[u8] {
    hash.as_bytes()
}

fn hash_from_blob(blob: &[u8]) -> Option<blake3::Hash> {
    if blob.len() == 32 {
        Some(blake3::Hash::from(<[u8; 32]>::try_from(blob).ok()?))
    } else {
        None
    }
}

fn datetime_to_text(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

fn text_to_datetime(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Mutex::new(Connection::open(path)?);
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Mutex::new(Connection::open_in_memory()?);
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.lock().execute_batch(
            "
            CREATE TABLE IF NOT EXISTS novels (
                id BLOB PRIMARY KEY,
                title TEXT NOT NULL,
                authors_json TEXT NOT NULL DEFAULT '[]',
                tags_json TEXT NOT NULL DEFAULT '[]',
                cover_hash BLOB,
                source_refs_json TEXT NOT NULL DEFAULT '[]',
                summary TEXT,
                status TEXT NOT NULL DEFAULT 'Unknown',
                added_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS chapters (
                id BLOB PRIMARY KEY,
                novel_id BLOB NOT NULL,
                idx INTEGER NOT NULL,
                title TEXT NOT NULL,
                published_at TEXT,
                content_hash BLOB NOT NULL,
                metalink_hash BLOB,
                source_url TEXT,
                FOREIGN KEY (novel_id) REFERENCES novels(id),
                UNIQUE(novel_id, idx)
            );

            CREATE TABLE IF NOT EXISTS reading_progress (
                novel_id BLOB NOT NULL,
                chapter_id BLOB NOT NULL,
                scroll_offset INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                device_id BLOB NOT NULL,
                PRIMARY KEY (novel_id, device_id),
                FOREIGN KEY (novel_id) REFERENCES novels(id),
                FOREIGN KEY (chapter_id) REFERENCES chapters(id)
            );

            CREATE TABLE IF NOT EXISTS plugins (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                manifest_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_chapters_novel ON chapters(novel_id);
            CREATE INDEX IF NOT EXISTS idx_progress_device ON reading_progress(device_id);
            ",
        )?;
        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    // --- Novels ---

    pub fn insert_novel(&self, novel: &Novel) -> Result<(), StoreError> {
        let authors_json = serde_json::to_string(&novel.authors)?;
        let tags_json = serde_json::to_string(&novel.tags)?;
        let source_refs_json = serde_json::to_string(&novel.source_refs)?;
        let cover_blob: Option<&[u8]> = novel.cover_hash.as_ref().map(hash_as_blob);

        self.lock().execute(
            "INSERT OR REPLACE INTO novels (id, title, authors_json, tags_json, cover_hash, source_refs_json, summary, status, added_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                hash_as_blob(&novel.id.0),
                &novel.title,
                &authors_json,
                &tags_json,
                cover_blob,
                &source_refs_json,
                &novel.summary,
                status_to_str(novel.status),
                datetime_to_text(&novel.added_at),
                datetime_to_text(&novel.updated_at),
            ],
        )?;
        Ok(())
    }

    pub fn get_novel(&self, id: &NovelId) -> Result<Option<Novel>, StoreError> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, title, authors_json, tags_json, cover_hash, source_refs_json, summary, status, added_at, updated_at FROM novels WHERE id = ?1",
        )?;

        let result = stmt
            .query_row(params![hash_as_blob(&id.0)], |row| {
                let id_blob: Vec<u8> = row.get(0)?;
                let title: String = row.get(1)?;
                let authors_json: String = row.get(2)?;
                let tags_json: String = row.get(3)?;
                let cover_blob: Option<Vec<u8>> = row.get(4)?;
                let source_refs_json: String = row.get(5)?;
                let summary: Option<String> = row.get(6)?;
                let status_str: String = row.get(7)?;
                let added_at_str: String = row.get(8)?;
                let updated_at_str: String = row.get(9)?;

                let id = hash_from_blob(&id_blob)
                    .ok_or_else(|| rusqlite::Error::InvalidColumnName("id".into()))?;

                let authors = serde_json::from_str(&authors_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
                let tags = serde_json::from_str(&tags_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
                let source_refs = serde_json::from_str(&source_refs_json)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;

                let cover_hash = cover_blob.and_then(|b| hash_from_blob(&b));
                let status = status_from_str(&status_str);
                let added_at = text_to_datetime(&added_at_str)
                    .unwrap_or_else(Utc::now);
                let updated_at = text_to_datetime(&updated_at_str)
                    .unwrap_or_else(Utc::now);

                Ok(Novel {
                    id: NovelId(id),
                    title,
                    authors,
                    tags,
                    cover_hash,
                    source_refs,
                    summary,
                    status,
                    added_at,
                    updated_at,
                })
            })
            .optional()?;

        Ok(result)
    }

    pub fn list_novels(&self) -> Result<Vec<Novel>, StoreError> {
        let ids: Vec<Vec<u8>> = {
            let conn = self.lock();
            let mut stmt = conn.prepare(
                "SELECT id FROM novels ORDER BY updated_at DESC",
            )?;
            stmt.query_map([], |row| row.get(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };

        let mut novels = Vec::with_capacity(ids.len());
        for id_blob in ids {
            if let Some(hash) = hash_from_blob(&id_blob) {
                let id = NovelId(hash);
                if let Some(novel) = self.get_novel(&id)? {
                    novels.push(novel);
                }
            }
        }
        Ok(novels)
    }

    pub fn delete_novel(&self, id: &NovelId) -> Result<(), StoreError> {
        self.lock().execute(
            "DELETE FROM chapters WHERE novel_id = ?1",
            params![hash_as_blob(&id.0)],
        )?;
        self.lock().execute(
            "DELETE FROM reading_progress WHERE novel_id = ?1",
            params![hash_as_blob(&id.0)],
        )?;
        self.lock().execute(
            "DELETE FROM novels WHERE id = ?1",
            params![hash_as_blob(&id.0)],
        )?;
        Ok(())
    }

    // --- Chapters ---

    pub fn insert_chapter(&self, chapter: &Chapter) -> Result<(), StoreError> {
        let metalink_blob: Option<&[u8]> = chapter
            .metalink_hash
            .as_ref()
            .map(hash_as_blob);
        let published = chapter
            .published_at
            .as_ref()
            .map(datetime_to_text);

        self.lock().execute(
            "INSERT OR REPLACE INTO chapters (id, novel_id, idx, title, published_at, content_hash, metalink_hash, source_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                hash_as_blob(&chapter.id.0),
                hash_as_blob(&chapter.novel_id.0),
                chapter.index,
                &chapter.title,
                published,
                hash_as_blob(&chapter.content_hash),
                metalink_blob,
                &chapter.source_url,
            ],
        )?;
        Ok(())
    }

    pub fn get_chapter(&self, id: &ChapterId) -> Result<Option<Chapter>, StoreError> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, novel_id, idx, title, published_at, content_hash, metalink_hash, source_url
             FROM chapters WHERE id = ?1",
        )?;

        stmt.query_row(params![hash_as_blob(&id.0)], |row| {
            let id_blob: Vec<u8> = row.get(0)?;
            let novel_id_blob: Vec<u8> = row.get(1)?;
            let index: u32 = row.get(2)?;
            let title: String = row.get(3)?;
            let published_at: Option<String> = row.get(4)?;
            let content_hash_blob: Vec<u8> = row.get(5)?;
            let metalink_blob: Option<Vec<u8>> = row.get(6)?;
            let source_url: Option<String> = row.get(7)?;

            let id = hash_from_blob(&id_blob)
                .ok_or_else(|| rusqlite::Error::InvalidColumnName("id".into()))?;
            let novel_id = hash_from_blob(&novel_id_blob)
                .ok_or_else(|| rusqlite::Error::InvalidColumnName("novel_id".into()))?;
            let content_hash = hash_from_blob(&content_hash_blob)
                .ok_or_else(|| rusqlite::Error::InvalidColumnName("content_hash".into()))?;
            let metalink_hash = metalink_blob.and_then(|b| hash_from_blob(&b));
            let published_at = published_at.and_then(|s| text_to_datetime(&s));

            Ok(Chapter {
                id: ChapterId(id),
                novel_id: NovelId(novel_id),
                index,
                title,
                published_at,
                content_hash,
                metalink_hash,
                source_url,
            })
        })
        .optional()
        .map_err(StoreError::from)
    }

    pub fn list_chapters_for_novel(
        &self,
        novel_id: &NovelId,
    ) -> Result<Vec<Chapter>, StoreError> {
        let ids: Vec<Vec<u8>> = {
            let conn = self.lock();
            let mut stmt = conn.prepare(
                "SELECT id FROM chapters WHERE novel_id = ?1 ORDER BY idx ASC",
            )?;
            stmt.query_map(params![hash_as_blob(&novel_id.0)], |row| row.get(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };

        let mut chapters = Vec::with_capacity(ids.len());
        for id_blob in ids {
            if let Some(hash) = hash_from_blob(&id_blob)
                && let Some(ch) = self.get_chapter(&ChapterId(hash))? {
                    chapters.push(ch);
                }
        }
        Ok(chapters)
    }

    pub fn delete_chapter(&self, id: &ChapterId) -> Result<(), StoreError> {
        self.lock()
            .execute(
                "DELETE FROM chapters WHERE id = ?1",
                params![hash_as_blob(&id.0)],
            )?;
        Ok(())
    }

    // --- Reading Progress ---

    pub fn set_progress(&self, progress: &ReadingProgress) -> Result<(), StoreError> {
        self.lock().execute(
            "INSERT OR REPLACE INTO reading_progress (novel_id, chapter_id, scroll_offset, updated_at, device_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                hash_as_blob(&progress.novel_id.0),
                hash_as_blob(&progress.chapter_id.0),
                progress.scroll_offset,
                datetime_to_text(&progress.updated_at),
                progress.device_id.as_bytes(),
            ],
        )?;
        Ok(())
    }

    pub fn get_progress(
        &self,
        novel_id: &NovelId,
        device_id: &NodeId,
    ) -> Result<Option<ReadingProgress>, StoreError> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT novel_id, chapter_id, scroll_offset, updated_at, device_id
             FROM reading_progress WHERE novel_id = ?1 AND device_id = ?2",
        )?;

        stmt.query_row(
            params![hash_as_blob(&novel_id.0), device_id.as_bytes()],
            |row| {
                let novel_id_blob: Vec<u8> = row.get(0)?;
                let chapter_id_blob: Vec<u8> = row.get(1)?;
                let scroll_offset: u64 = row.get(2)?;
                let updated_at_str: String = row.get(3)?;
                let device_id_blob: Vec<u8> = row.get(4)?;

                let novel_id = hash_from_blob(&novel_id_blob)
                    .ok_or_else(|| rusqlite::Error::InvalidColumnName("novel_id".into()))?;
                let chapter_id = hash_from_blob(&chapter_id_blob)
                    .ok_or_else(|| rusqlite::Error::InvalidColumnName("chapter_id".into()))?;
                let device_id = device_id_blob.try_into().map_err(|_| {
                    rusqlite::Error::InvalidColumnName("device_id".into())
                })?;
                let updated_at = text_to_datetime(&updated_at_str)
                    .unwrap_or_else(Utc::now);

                Ok(ReadingProgress {
                    novel_id: NovelId(novel_id),
                    chapter_id: ChapterId(chapter_id),
                    scroll_offset,
                    updated_at,
                    device_id: NodeId(device_id),
                })
            },
        )
        .optional()
        .map_err(StoreError::from)
    }

    pub fn list_progress_for_device(
        &self,
        device_id: &NodeId,
    ) -> Result<Vec<ReadingProgress>, StoreError> {
        let rows: Vec<(Vec<u8>, Vec<u8>)> = {
            let conn = self.lock();
            let mut stmt = conn.prepare(
                "SELECT novel_id, device_id FROM reading_progress WHERE device_id = ?1",
            )?;
            stmt.query_map(params![device_id.as_bytes()], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?
        };

        let mut results = Vec::new();
        for (novel_id_blob, device_id_blob) in rows {
            let nid = hash_from_blob(&novel_id_blob);
            let did: Option<[u8; 32]> = device_id_blob.try_into().ok();
            if let (Some(novel_hash), Some(device_bytes)) = (nid, did)
                && let Some(progress) =
                    self.get_progress(&NovelId(novel_hash), &NodeId(device_bytes))?
                {
                    results.push(progress);
                }
        }
        Ok(results)
    }

    // --- Plugins ---

    pub fn insert_plugin(&self, manifest: &PluginManifest) -> Result<(), StoreError> {
        let json = serde_json::to_string(manifest)?;
        self.lock().execute(
            "INSERT OR REPLACE INTO plugins (id, name, version, manifest_json) VALUES (?1, ?2, ?3, ?4)",
            params![&manifest.id.0, &manifest.name, &manifest.version, &json],
        )?;
        Ok(())
    }

    pub fn list_plugins(&self) -> Result<Vec<PluginManifest>, StoreError> {
        let conn = self.lock();
        let mut stmt = conn.prepare("SELECT manifest_json FROM plugins")?;
        let rows: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut manifests = Vec::new();
        for json in rows {
            manifests.push(serde_json::from_str(&json)?);
        }
        Ok(manifests)
    }

    pub fn delete_plugin(&self, id: &str) -> Result<(), StoreError> {
        self.lock()
            .execute("DELETE FROM plugins WHERE id = ?1", params![id])?;
        Ok(())
    }
}

fn status_to_str(status: NovelStatus) -> &'static str {
    match status {
        NovelStatus::Unknown => "Unknown",
        NovelStatus::Ongoing => "Ongoing",
        NovelStatus::Completed => "Completed",
        NovelStatus::Hiatus => "Hiatus",
        NovelStatus::Dropped => "Dropped",
    }
}

fn status_from_str(s: &str) -> NovelStatus {
    match s {
        "Ongoing" => NovelStatus::Ongoing,
        "Completed" => NovelStatus::Completed,
        "Hiatus" => NovelStatus::Hiatus,
        "Dropped" => NovelStatus::Dropped,
        _ => NovelStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ln_core::chapter::Chapter;
    use ln_core::novel::Novel;
    use ln_core::progress::ReadingProgress;

    fn test_node() -> NodeId {
        NodeId([0u8; 32])
    }

    #[test]
    fn insert_and_get_novel() {
        let db = Database::open_in_memory().unwrap();
        let novel = Novel::new("Test Novel", "https://test.com/novel")
            .with_author("Author A")
            .with_tag("fantasy");

        db.insert_novel(&novel).unwrap();
        let found = db.get_novel(&novel.id).unwrap().unwrap();
        assert_eq!(found.title, "Test Novel");
        assert_eq!(found.authors.len(), 1);
        assert_eq!(found.tags.len(), 1);
    }

    #[test]
    fn list_novels() {
        let db = Database::open_in_memory().unwrap();
        db.insert_novel(&Novel::new("A", "https://a.com")).unwrap();
        db.insert_novel(&Novel::new("B", "https://b.com")).unwrap();
        let novels = db.list_novels().unwrap();
        assert_eq!(novels.len(), 2);
    }

    #[test]
    fn delete_novel_cascades() {
        let db = Database::open_in_memory().unwrap();
        let novel = Novel::new("Test", "https://test.com");
        let nid = novel.id;
        db.insert_novel(&novel).unwrap();

        let chapter = Chapter::new(nid, 0, "Ch1", blake3::hash(b"content"));
        db.insert_chapter(&chapter).unwrap();

        db.delete_novel(&nid).unwrap();
        assert!(db.get_novel(&nid).unwrap().is_none());
        assert!(db.get_chapter(&chapter.id).unwrap().is_none());
    }

    #[test]
    fn chapter_crud() {
        let db = Database::open_in_memory().unwrap();
        let novel = Novel::new("Test", "https://test.com");
        db.insert_novel(&novel).unwrap();

        let ch = Chapter::new(novel.id, 0, "Chapter 1", blake3::hash(b"c1"));
        db.insert_chapter(&ch).unwrap();

        let found = db.get_chapter(&ch.id).unwrap().unwrap();
        assert_eq!(found.title, "Chapter 1");

        let list = db.list_chapters_for_novel(&novel.id).unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn reading_progress() {
        let db = Database::open_in_memory().unwrap();
        let novel = Novel::new("Test", "https://test.com");
        db.insert_novel(&novel).unwrap();

        let ch = Chapter::new(novel.id, 0, "Ch1", blake3::hash(b"c1"));
        db.insert_chapter(&ch).unwrap();

        let device = test_node();
        let progress = ReadingProgress {
            novel_id: novel.id,
            chapter_id: ch.id,
            scroll_offset: 100,
            updated_at: Utc::now(),
            device_id: device,
        };
        db.set_progress(&progress).unwrap();

        let found = db.get_progress(&novel.id, &device).unwrap().unwrap();
        assert_eq!(found.scroll_offset, 100);

        let device_list = db.list_progress_for_device(&device).unwrap();
        assert_eq!(device_list.len(), 1);
    }

    #[test]
    fn plugin_crud() {
        let db = Database::open_in_memory().unwrap();
        let manifest = PluginManifest {
            id: ln_core::id::PluginId("test-plugin".into()),
            name: "Test Plugin".into(),
            version: "1.0.0".into(),
            supported_sites: vec!["example.com".into()],
            capabilities: Default::default(),
        };

        db.insert_plugin(&manifest).unwrap();
        let plugins = db.list_plugins().unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "Test Plugin");

        db.delete_plugin("test-plugin").unwrap();
        assert_eq!(db.list_plugins().unwrap().len(), 0);
    }
}
