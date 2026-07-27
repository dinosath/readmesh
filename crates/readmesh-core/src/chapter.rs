use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{ChapterId, NovelId};

/// Represents a chapter of a novel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: ChapterId,
    pub novel_id: NovelId,
    pub index: u32,
    pub title: String,
    pub published_at: Option<DateTime<Utc>>,
    pub content_hash: blake3::Hash,
    pub metalink_hash: Option<blake3::Hash>,
    pub source_url: Option<String>,
}

impl Chapter {
    /// Create a new chapter, computing the ChapterId deterministically
    /// from the chapter URL, novel ID, and index.
    pub fn new(
        novel_id: NovelId,
        index: u32,
        title: impl Into<String>,
        content_hash: blake3::Hash,
    ) -> Self {
        let title: String = title.into();
        let source_url = String::new();
        let id = ChapterId::compute(&source_url, &novel_id, index);
        Self {
            id,
            novel_id,
            index,
            title,
            published_at: None,
            content_hash,
            metalink_hash: None,
            source_url: None,
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        let url: String = url.into();
        self.source_url = Some(url.clone());
        self.id = ChapterId::compute(&url, &self.novel_id, self.index);
        self
    }

    pub fn with_published_date(mut self, date: DateTime<Utc>) -> Self {
        self.published_at = Some(date);
        self
    }

    pub fn with_metalink_hash(mut self, hash: blake3::Hash) -> Self {
        self.metalink_hash = Some(hash);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::NovelId;

    #[test]
    fn chapter_constructor() {
        let nid = NovelId::compute("https://example.com", "Test Novel");
        let content = b"Chapter 1 content";
        let hash = blake3::hash(content);
        let ch = Chapter::new(nid, 0, "Chapter 1", hash);

        assert_eq!(ch.novel_id, nid);
        assert_eq!(ch.index, 0);
        assert_eq!(ch.title, "Chapter 1");
        assert_eq!(ch.content_hash, hash);
    }

    #[test]
    fn chapter_with_url() {
        let nid = NovelId::compute("https://example.com", "Test");
        let hash = blake3::hash(b"content");
        let ch = Chapter::new(nid, 0, "Ch1", hash).with_url("https://example.com/ch1");

        assert_eq!(ch.source_url.as_deref(), Some("https://example.com/ch1"));
    }

    #[test]
    fn different_chapters_have_different_ids() {
        let nid = NovelId::compute("https://example.com", "Test");
        let hash = blake3::hash(b"content");
        let ch0 = Chapter::new(nid, 0, "Ch0", hash);
        let ch1 = Chapter::new(nid, 1, "Ch1", hash);
        assert_ne!(ch0.id, ch1.id);
    }
}
