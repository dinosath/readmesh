//! Library state: the user's collection, favorites, read/unread tracking and
//! reading progress. Backed by the existing `readmesh-core` in-memory `Library`
//! store so persistence (`readmesh-store`) can be dropped in later.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use readmesh_core::{Chapter, ChapterId, Library, NodeId, Novel, NovelId, ReadingProgress};

/// A "continue reading" entry: where the user left off in a novel.
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueReadingItem {
    pub novel_id: NovelId,
    pub chapter_id: ChapterId,
    /// Fraction of the novel's chapters marked read, `0.0..=1.0`.
    pub progress: f32,
    pub updated_at: DateTime<Utc>,
}

/// The user's library plus reading metadata.
#[derive(Debug)]
pub struct LibraryState {
    /// Novel + chapter storage (reused from `readmesh-core`).
    pub library: Library,
    /// Bookmarked novels, ordered for deterministic display.
    favorites: BTreeSet<NovelId>,
    /// Chapters the user has finished reading.
    read_chapters: BTreeSet<ChapterId>,
    /// The device id attached to progress records (readmesh-sync compatible).
    device_id: NodeId,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryState {
    pub fn new() -> Self {
        Self {
            library: Library::new(),
            favorites: BTreeSet::new(),
            read_chapters: BTreeSet::new(),
            device_id: NodeId([0u8; 32]),
        }
    }

    pub fn with_device_id(device_id: NodeId) -> Self {
        Self {
            device_id,
            ..Self::new()
        }
    }

    // ---- membership ------------------------------------------------------

    pub fn add_to_library(&mut self, novel: Novel, chapters: Vec<Chapter>) {
        self.library.add_chapters(chapters);
        self.library.add_novel(novel);
    }

    pub fn remove_from_library(&mut self, novel_id: &NovelId) -> bool {
        self.favorites.remove(novel_id);
        self.library.remove_novel(novel_id).is_some()
    }

    pub fn contains(&self, novel_id: &NovelId) -> bool {
        self.library.get_novel(novel_id).is_some()
    }

    pub fn novel_count(&self) -> usize {
        self.library.novel_count()
    }

    // ---- favorites --------------------------------------------------------

    /// Toggle the favorite flag; returns the new value.
    pub fn toggle_favorite(&mut self, novel_id: &NovelId) -> bool {
        if !self.favorites.remove(novel_id) {
            self.favorites.insert(*novel_id);
            true
        } else {
            false
        }
    }

    pub fn is_favorite(&self, novel_id: &NovelId) -> bool {
        self.favorites.contains(novel_id)
    }

    pub fn favorites(&self) -> Vec<NovelId> {
        self.favorites
            .iter()
            .filter(|id| self.contains(id))
            .copied()
            .collect()
    }

    // ---- read / unread ------------------------------------------------------

    pub fn mark_read(&mut self, chapter_id: &ChapterId) {
        self.read_chapters.insert(*chapter_id);
    }

    pub fn mark_unread(&mut self, chapter_id: &ChapterId) {
        self.read_chapters.remove(chapter_id);
    }

    pub fn is_read(&self, chapter_id: &ChapterId) -> bool {
        self.read_chapters.contains(chapter_id)
    }

    /// Mark every chapter of a novel up to and including `index` as read.
    pub fn mark_read_through(&mut self, novel_id: &NovelId, index: u32) {
        for chapter in self.library.chapters_for_novel(novel_id) {
            if chapter.index <= index {
                self.read_chapters.insert(chapter.id);
            }
        }
    }

    pub fn read_count_for(&self, novel_id: &NovelId) -> usize {
        self.library
            .chapters_for_novel(novel_id)
            .iter()
            .filter(|c| self.read_chapters.contains(&c.id))
            .count()
    }

    /// Fraction of a novel's chapters that are read, `0.0..=1.0`.
    pub fn progress_for(&self, novel_id: &NovelId) -> f32 {
        let total = self.library.chapter_count_for_novel(novel_id);
        if total == 0 {
            return 0.0;
        }
        self.read_count_for(novel_id) as f32 / total as f32
    }

    // ---- progress ----------------------------------------------------------

    /// Record reading progress (also marks earlier chapters read).
    pub fn record_progress(
        &mut self,
        novel_id: &NovelId,
        chapter_id: &ChapterId,
        scroll_offset: u64,
    ) {
        if let Some(chapter) = self.library.get_chapter(chapter_id) {
            self.mark_read_through(novel_id, chapter.index.saturating_sub(1));
        }
        self.library.set_progress(ReadingProgress {
            novel_id: *novel_id,
            chapter_id: *chapter_id,
            scroll_offset,
            updated_at: Utc::now(),
            device_id: self.device_id,
        });
    }

    /// Continue-reading list: novels with saved progress, most recent first.
    pub fn continue_reading(&self) -> Vec<ContinueReadingItem> {
        let mut items: Vec<ContinueReadingItem> = self
            .library
            .all_progress_for_device(&self.device_id)
            .iter()
            .filter(|p| self.contains(&p.novel_id))
            .map(|p| ContinueReadingItem {
                novel_id: p.novel_id,
                chapter_id: p.chapter_id,
                progress: self.progress_for(&p.novel_id),
                updated_at: p.updated_at,
            })
            .collect();
        items.sort_by_key(|i| std::cmp::Reverse(i.updated_at));
        items
    }

    /// Whether the user has saved reading progress for this novel.
    pub fn has_saved_progress(&self, novel_id: &NovelId) -> bool {
        self.library
            .get_progress(novel_id, &self.device_id)
            .is_some()
    }

    /// The chapter to resume in a novel: the saved progress chapter, or the
    /// first chapter when nothing was read yet.
    pub fn resume_chapter(&self, novel_id: &NovelId) -> Option<ChapterId> {
        if let Some(p) = self.library.get_progress(novel_id, &self.device_id) {
            return Some(p.chapter_id);
        }
        self.library
            .chapters_for_novel(novel_id)
            .first()
            .map(|c| c.id)
    }

    // ---- listings -----------------------------------------------------------

    /// Novels in the library, most recently added first.
    pub fn recently_added(&self, limit: usize) -> Vec<NovelId> {
        let mut novels: Vec<&Novel> = self.library.novels().collect();
        novels.sort_by_key(|n| std::cmp::Reverse(n.added_at));
        novels.into_iter().take(limit).map(|n| n.id).collect()
    }

    /// Novels in the library with the newest content updates first.
    pub fn recently_updated(&self, limit: usize) -> Vec<NovelId> {
        let mut novels: Vec<&Novel> = self.library.novels().collect();
        novels.sort_by_key(|n| std::cmp::Reverse(n.updated_at));
        novels.into_iter().take(limit).map(|n| n.id).collect()
    }

    /// Library novels grouped by their primary tag (first tag), for the
    /// categories section. Groups are ordered by name.
    pub fn categories(&self) -> BTreeMap<String, Vec<NovelId>> {
        let mut map: BTreeMap<String, Vec<NovelId>> = BTreeMap::new();
        for novel in self.library.novels() {
            let category = novel
                .tags
                .first()
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "Uncategorized".to_string());
            map.entry(category).or_default().push(novel.id);
        }
        map
    }

    /// All library novels for the main grid, alphabetically.
    pub fn all_sorted(&self) -> Vec<NovelId> {
        let mut novels: Vec<&Novel> = self.library.novels().collect();
        novels.sort_by_key(|n| n.title.to_lowercase());
        novels.into_iter().map(|n| n.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockCatalog;
    use crate::repository::ContentRepository;

    fn seed() -> (MockCatalog, LibraryState, NovelId, NovelId) {
        let catalog = MockCatalog::demo();
        let mut state = LibraryState::new();
        let ids = catalog.all_novels();
        let a = ids[0].id;
        let b = ids[1].id;
        for id in [a, b] {
            let novel = catalog.novel(&id).unwrap();
            let chapters = catalog.chapters(&id);
            state.add_to_library(novel, chapters);
        }
        (catalog, state, a, b)
    }

    #[test]
    fn add_and_remove_items() {
        let (_c, mut state, a, _b) = seed();
        assert_eq!(state.novel_count(), 2);
        assert!(state.contains(&a));

        assert!(state.remove_from_library(&a));
        assert!(!state.contains(&a));
        assert_eq!(state.novel_count(), 1);
        assert!(
            !state.remove_from_library(&a),
            "double remove returns false"
        );
    }

    #[test]
    fn bookmarking_toggles() {
        let (_c, mut state, a, _b) = seed();
        assert!(!state.is_favorite(&a));
        assert!(state.toggle_favorite(&a));
        assert!(state.is_favorite(&a));
        assert_eq!(state.favorites(), vec![a]);
        assert!(!state.toggle_favorite(&a));
        assert!(state.favorites().is_empty());
    }

    #[test]
    fn removing_from_library_drops_favorite() {
        let (_c, mut state, a, _b) = seed();
        state.toggle_favorite(&a);
        state.remove_from_library(&a);
        assert!(state.favorites().is_empty());
    }

    #[test]
    fn continue_reading_tracks_progress() {
        let (catalog, mut state, a, b) = seed();
        let ch_a = catalog.chapters(&a)[2].id;
        let ch_b = catalog.chapters(&b)[0].id;

        state.record_progress(&b, &ch_b, 0);
        state.record_progress(&a, &ch_a, 128);

        let cr = state.continue_reading();
        assert_eq!(cr.len(), 2);
        // Most recent first: novel a was updated last.
        assert_eq!(cr[0].novel_id, a);
        assert_eq!(cr[0].chapter_id, ch_a);
        assert_eq!(cr[1].novel_id, b);
    }

    #[test]
    fn progress_marks_earlier_chapters_read() {
        let (catalog, mut state, a, _b) = seed();
        let chapters = catalog.chapters(&a);
        let target = chapters[4].id; // index 4 (0-based chapter list index)
        state.record_progress(&a, &target, 0);

        // Chapters before the current one are considered read.
        for ch in &chapters[..4] {
            assert!(state.is_read(&ch.id), "chapter {} should be read", ch.index);
        }
        assert!(!state.is_read(&target));
        assert_eq!(state.resume_chapter(&a), Some(target));
    }

    #[test]
    fn reading_progress_fraction() {
        let (catalog, mut state, a, _b) = seed();
        assert!((state.progress_for(&a) - 0.0).abs() < f32::EPSILON);
        let chapters = catalog.chapters(&a);
        let half = chapters.len() / 2;
        for ch in &chapters[..half] {
            state.mark_read(&ch.id);
        }
        let expected = half as f32 / chapters.len() as f32;
        assert!((state.progress_for(&a) - expected).abs() < 0.001);
    }

    #[test]
    fn mark_read_unread_roundtrip() {
        let (catalog, mut state, a, _b) = seed();
        let ch = catalog.chapters(&a)[0].id;
        state.mark_read(&ch);
        assert!(state.is_read(&ch));
        state.mark_unread(&ch);
        assert!(!state.is_read(&ch));
    }

    #[test]
    fn categories_group_by_primary_tag() {
        let (catalog, state, a, b) = seed();
        let cats = state.categories();
        assert!(!cats.is_empty());
        let novel_a = catalog.novel(&a).unwrap();
        let tag_a = &novel_a.tags[0].name;
        assert!(cats.get(tag_a).unwrap().contains(&a));
        let _ = b;
    }

    #[test]
    fn recently_added_orders_newest_first() {
        let (_c, state, a, b) = seed();
        let recent = state.recently_added(10);
        assert_eq!(recent.len(), 2);
        assert!(recent.contains(&a) && recent.contains(&b));
        let first = state.library.get_novel(&recent[0]).unwrap();
        let second = state.library.get_novel(&recent[1]).unwrap();
        assert!(first.added_at >= second.added_at);
    }

    #[test]
    fn resume_defaults_to_first_chapter() {
        let (catalog, state, a, _b) = seed();
        let first = catalog.chapters(&a)[0].id;
        assert_eq!(state.resume_chapter(&a), Some(first));
    }
}
