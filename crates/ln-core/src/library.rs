//! In-memory library of novels, chapters, and reading progress.
//!
//! This is the core data structure used by the daemon to track what
//! the user has in their library. It backs onto persistent storage
//! via `ln-store` but provides the authoritative in-memory state.

use std::collections::BTreeMap;

use crate::chapter::Chapter;
use crate::id::{ChapterId, NodeId, NovelId};
use crate::novel::{Novel, NovelStatus};
use crate::progress::ReadingProgress;

/// In-memory library tracking novels, chapters, and reading progress.
#[derive(Debug, Default)]
pub struct Library {
    novels: BTreeMap<NovelId, Novel>,
    chapters: BTreeMap<ChapterId, Chapter>,
    /// Index: novel_id -> list of chapter_ids in index order
    novel_chapters: BTreeMap<NovelId, Vec<ChapterId>>,
    progress: BTreeMap<(NovelId, NodeId), ReadingProgress>,
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Novels ---

    pub fn add_novel(&mut self, novel: Novel) -> &Novel {
        let id = novel.id;
        self.novels.insert(id, novel);
        self.novels.get(&id).unwrap()
    }

    pub fn remove_novel(&mut self, novel_id: &NovelId) -> Option<Novel> {
        self.chapters
            .retain(|_, ch| ch.novel_id != *novel_id);
        self.novel_chapters.remove(novel_id);
        self.novels.remove(novel_id)
    }

    pub fn get_novel(&self, novel_id: &NovelId) -> Option<&Novel> {
        self.novels.get(novel_id)
    }

    pub fn novels(&self) -> impl Iterator<Item = &Novel> {
        self.novels.values()
    }

    pub fn novel_count(&self) -> usize {
        self.novels.len()
    }

    pub fn search_by_title(&self, query: &str) -> Vec<&Novel> {
        let lower = query.to_lowercase();
        self.novels
            .values()
            .filter(|n| n.title.to_lowercase().contains(&lower))
            .collect()
    }

    pub fn novels_by_status(&self, status: NovelStatus) -> Vec<&Novel> {
        self.novels
            .values()
            .filter(|n| n.status == status)
            .collect()
    }

    pub fn novels_by_tag(&self, tag: &str) -> Vec<&Novel> {
        self.novels
            .values()
            .filter(|n| n.tags.iter().any(|t| t.name.eq_ignore_ascii_case(tag)))
            .collect()
    }

    // --- Chapters ---

    pub fn add_chapter(&mut self, chapter: Chapter) {
        let novel_id = chapter.novel_id;
        self.novel_chapters
            .entry(novel_id)
            .or_default()
            .push(chapter.id);
        self.chapters.insert(chapter.id, chapter);
    }

    pub fn add_chapters(&mut self, chapters: Vec<Chapter>) {
        for chapter in chapters {
            self.add_chapter(chapter);
        }
    }

    pub fn get_chapter(&self, chapter_id: &ChapterId) -> Option<&Chapter> {
        self.chapters.get(chapter_id)
    }

    pub fn chapters_for_novel(&self, novel_id: &NovelId) -> Vec<&Chapter> {
        self.novel_chapters
            .get(novel_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.chapters.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn chapter_count_for_novel(&self, novel_id: &NovelId) -> usize {
        self.novel_chapters
            .get(novel_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    // --- Reading Progress ---

    pub fn set_progress(&mut self, progress: ReadingProgress) {
        let key = (progress.novel_id, progress.device_id);
        self.progress.insert(key, progress);
    }

    pub fn get_progress(
        &self,
        novel_id: &NovelId,
        device_id: &NodeId,
    ) -> Option<&ReadingProgress> {
        self.progress.get(&(*novel_id, *device_id))
    }

    pub fn all_progress_for_device(&self, device_id: &NodeId) -> Vec<&ReadingProgress> {
        self.progress
            .values()
            .filter(|p| p.device_id == *device_id)
            .collect()
    }

    // --- Stats ---

    pub fn chapter_count(&self) -> usize {
        self.chapters.len()
    }

    pub fn progress_count(&self) -> usize {
        self.progress.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::NovelId;
    use crate::progress::ReadingProgress;
    use chrono::Utc;

    fn test_node() -> NodeId {
        NodeId([0u8; 32])
    }

    fn test_novel() -> Novel {
        Novel::new("Test Novel", "https://test.com/novel")
    }

    fn test_chapter(novel_id: NovelId, index: u32, content: &[u8]) -> Chapter {
        Chapter::new(novel_id, index, format!("Chapter {index}"), blake3::hash(content))
    }

    #[test]
    fn library_add_and_get_novel() {
        let mut lib = Library::new();
        let novel = test_novel();
        let id = novel.id;
        lib.add_novel(novel);

        assert_eq!(lib.novel_count(), 1);
        let found = lib.get_novel(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Novel");
    }

    #[test]
    fn library_add_chapters() {
        let mut lib = Library::new();
        let novel = test_novel();
        let nid = novel.id;
        lib.add_novel(novel);

        lib.add_chapter(test_chapter(nid, 0, b"content 0"));
        lib.add_chapter(test_chapter(nid, 1, b"content 1"));
        lib.add_chapter(test_chapter(nid, 2, b"content 2"));

        let chapters = lib.chapters_for_novel(&nid);
        assert_eq!(chapters.len(), 3);
    }

    #[test]
    fn library_remove_novel_cleans_up_chapters() {
        let mut lib = Library::new();
        let novel = test_novel();
        let nid = novel.id;
        lib.add_novel(novel);
        lib.add_chapter(test_chapter(nid, 0, b"content"));

        lib.remove_novel(&nid);
        assert_eq!(lib.novel_count(), 0);
        assert_eq!(lib.chapter_count(), 0);
        assert!(lib.chapters_for_novel(&nid).is_empty());
    }

    #[test]
    fn library_search_by_title() {
        let mut lib = Library::new();
        lib.add_novel(Novel::new("One Piece", "https://test.com/one-piece"));
        lib.add_novel(Novel::new("Naruto", "https://test.com/naruto"));
        lib.add_novel(Novel::new("Bleach", "https://test.com/bleach"));

        let results = lib.search_by_title("piece");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "One Piece");

        let results = lib.search_by_title("a");
        assert_eq!(results.len(), 2); // N**a**ruto, Ble**a**ch
    }

    #[test]
    fn library_reading_progress() {
        let mut lib = Library::new();
        let novel = test_novel();
        let nid = novel.id;
        lib.add_novel(novel);

        let ch = test_chapter(nid, 0, b"content");
        let cid = ch.id;
        lib.add_chapter(ch);

        let device = test_node();
        let progress = ReadingProgress {
            novel_id: nid,
            chapter_id: cid,
            scroll_offset: 42,
            updated_at: Utc::now(),
            device_id: device,
        };

        lib.set_progress(progress);
        let found = lib.get_progress(&nid, &device);
        assert!(found.is_some());
        assert_eq!(found.unwrap().scroll_offset, 42);

        let all = lib.all_progress_for_device(&device);
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn library_novels_by_tag() {
        let mut lib = Library::new();
        lib.add_novel(
            Novel::new("Fantasy Novel", "https://test.com/fantasy")
                .with_tag("Fantasy"),
        );
        lib.add_novel(
            Novel::new("Sci-Fi Novel", "https://test.com/scifi")
                .with_tag("Sci-Fi"),
        );

        assert_eq!(lib.novels_by_tag("Fantasy").len(), 1);
        assert_eq!(lib.novels_by_tag("Sci-Fi").len(), 1);
        assert_eq!(lib.novels_by_tag("Romance").len(), 0);
    }
}
