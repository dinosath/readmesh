//! Repository traits: the seam between application logic and data sources.
//!
//! The UI and application services depend only on these traits. Today they
//! are backed by [`crate::mock::MockCatalog`]; later they will be backed by
//! the ReadMesh daemon / P2P federation without any UI changes.

use readmesh_core::{Chapter, ChapterId, Novel, NovelId, PluginId};

/// Information about an available content source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceInfo {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub novel_count: usize,
}

/// Read-side access to the content catalog (browse, search, chapters,
/// chapter content).
///
/// All methods return owned data so implementations may be backed by
/// networking or storage without lifetime coupling to the caller.
pub trait ContentRepository {
    /// Available sources, in display order.
    fn sources(&self) -> Vec<SourceInfo>;

    /// Every novel in the catalog.
    fn all_novels(&self) -> Vec<Novel>;

    /// A single novel by id.
    fn novel(&self, id: &NovelId) -> Option<Novel>;

    /// Chapters of a novel, ordered by chapter index ascending.
    fn chapters(&self, novel: &NovelId) -> Vec<Chapter>;

    /// Full text content of a chapter, if available.
    fn chapter_content(&self, chapter: &ChapterId) -> Option<String>;

    /// Size in bytes of a chapter's content (for download estimates).
    fn chapter_size(&self, chapter: &ChapterId) -> u64;

    /// Free-text search across titles, alternative titles, authors and tags.
    /// Returns novel ids ranked by relevance (best first). An empty query
    /// returns all novels in default (recently updated) order.
    fn search(&self, query: &str) -> Vec<NovelId>;

    /// Trending novels (most actively read across the mesh).
    fn trending(&self, limit: usize) -> Vec<NovelId>;

    /// Featured/editorial picks.
    fn featured(&self, limit: usize) -> Vec<NovelId>;

    /// Most recently added novels.
    fn recently_added(&self, limit: usize) -> Vec<NovelId>;

    /// Novels with the most recent chapter updates.
    fn recently_updated(&self, limit: usize) -> Vec<NovelId>;

    /// Distinct genre/tag names in the catalog, sorted.
    fn genres(&self) -> Vec<String>;

    /// Novels carrying a given genre/tag.
    fn novels_by_genre(&self, genre: &str) -> Vec<NovelId>;
}
