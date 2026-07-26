use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{AuthorId, NovelId, PluginId};

/// Represents a light novel tracked in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Novel {
    pub id: NovelId,
    pub title: String,
    pub authors: Vec<Author>,
    pub tags: Vec<Tag>,
    pub cover_hash: Option<blake3::Hash>,
    pub source_refs: Vec<SourceRef>,
    pub summary: Option<String>,
    pub status: NovelStatus,
    pub added_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An author of a novel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: AuthorId,
    pub name: String,
}

/// A tag/category/genre applied to a novel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
}

/// Status of a novel (ongoing, completed, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NovelStatus {
    Unknown,
    Ongoing,
    Completed,
    Hiatus,
    Dropped,
}

/// A reference to a novel on a specific source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRef {
    pub plugin_id: PluginId,
    pub remote_url: String,
    pub last_checked: Option<DateTime<Utc>>,
}

impl Novel {
    pub fn new(title: impl Into<String>, source_url: impl Into<String>) -> Self {
        let title: String = title.into();
        let source_url: String = source_url.into();
        let id = NovelId::compute(&source_url, &title);
        let now = Utc::now();
        Self {
            id,
            title,
            authors: vec![],
            tags: vec![],
            cover_hash: None,
            source_refs: vec![SourceRef {
                plugin_id: PluginId("builtin".into()),
                remote_url: source_url,
                last_checked: None,
            }],
            summary: None,
            status: NovelStatus::Unknown,
            added_at: now,
            updated_at: now,
        }
    }

    pub fn with_author(mut self, name: impl Into<String>) -> Self {
        let name: String = name.into();
        let id = AuthorId::compute(&name);
        self.authors.push(Author { id, name });
        self
    }

    pub fn with_tag(mut self, name: impl Into<String>) -> Self {
        self.tags.push(Tag {
            name: name.into(),
        });
        self
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_status(mut self, status: NovelStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_cover(mut self, cover_hash: blake3::Hash) -> Self {
        self.cover_hash = Some(cover_hash);
        self
    }

    pub fn add_source_ref(&mut self, plugin_id: PluginId, url: String) {
        self.source_refs.push(SourceRef {
            plugin_id,
            remote_url: url,
            last_checked: None,
        });
        self.updated_at = Utc::now();
    }
}

impl Author {
    pub fn new(name: impl Into<String>) -> Self {
        let name: String = name.into();
        let id = AuthorId::compute(&name);
        Self { id, name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn novel_builder() {
        let novel = Novel::new("My Novel", "https://source.com/novel")
            .with_author("Author One")
            .with_author("Author Two")
            .with_tag("fantasy")
            .with_tag("adventure")
            .with_summary("A great novel")
            .with_status(NovelStatus::Ongoing);

        assert_eq!(novel.title, "My Novel");
        assert_eq!(novel.authors.len(), 2);
        assert_eq!(novel.tags.len(), 2);
        assert_eq!(novel.summary.as_deref(), Some("A great novel"));
        assert_eq!(novel.status, NovelStatus::Ongoing);
        assert_eq!(novel.source_refs.len(), 1);
    }

    #[test]
    fn novel_id_stable_for_same_input() {
        let a = Novel::new("Title", "https://example.com/novel");
        let b = Novel::new("Title", "https://example.com/novel");
        assert_eq!(a.id, b.id, "Same title and URL should produce same ID");
    }

    #[test]
    fn author_constructor() {
        let author = Author::new("Test Author");
        assert_eq!(author.name, "Test Author");
        let same = Author::new("Test Author");
        assert_eq!(author.id, same.id, "Same name should produce same AuthorId");
    }
}
