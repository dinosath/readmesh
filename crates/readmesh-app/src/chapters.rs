//! Chapter list presentation state: sorting, filtering and selection.
//!
//! Pure functions operate on `readmesh-core` chapters plus read/downloaded lookup
//! closures, keeping this module independent of both the UI and storage.

use readmesh_core::{Chapter, ChapterId};

/// Chapter list ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChapterSort {
    /// Reading order (default).
    #[default]
    IndexAsc,
    IndexDesc,
    /// Newest publication date first (undated chapters last).
    DateDesc,
}

/// Chapter list filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChapterFilter {
    pub unread_only: bool,
    pub downloaded_only: bool,
}

/// Sorting + filtering state for the chapter list screen.
#[derive(Debug, Clone, Default)]
pub struct ChapterListState {
    pub sort: ChapterSort,
    pub filter: ChapterFilter,
    /// Currently selected chapter (highlighted in the list).
    pub selected: Option<ChapterId>,
}

impl ChapterListState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort = match self.sort {
            ChapterSort::IndexAsc => ChapterSort::IndexDesc,
            ChapterSort::IndexDesc => ChapterSort::IndexAsc,
            ChapterSort::DateDesc => ChapterSort::IndexAsc,
        };
    }

    pub fn select(&mut self, chapter: ChapterId) {
        self.selected = Some(chapter);
    }
}

/// Sort chapters (borrowed slice) according to `sort`.
pub fn sort_chapters(chapters: &mut Vec<&Chapter>, sort: ChapterSort) {
    match sort {
        ChapterSort::IndexAsc => chapters.sort_by_key(|c| c.index),
        ChapterSort::IndexDesc => chapters.sort_by_key(|c| std::cmp::Reverse(c.index)),
        ChapterSort::DateDesc => {
            chapters.sort_by_key(|c| std::cmp::Reverse(c.published_at));
        }
    }
}

/// Filter chapters by read/downloaded state.
pub fn filter_chapters<'a>(
    chapters: impl IntoIterator<Item = &'a Chapter>,
    filter: &ChapterFilter,
    is_read: impl Fn(&ChapterId) -> bool,
    is_downloaded: impl Fn(&ChapterId) -> bool,
) -> Vec<&'a Chapter> {
    chapters
        .into_iter()
        .filter(|c| !filter.unread_only || !is_read(&c.id))
        .filter(|c| !filter.downloaded_only || is_downloaded(&c.id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockCatalog;
    use crate::repository::ContentRepository;
    use std::collections::HashSet;

    fn fixture() -> (MockCatalog, Vec<Chapter>) {
        let catalog = MockCatalog::demo();
        let novel = catalog.all_novels()[0].id;
        let chapters = catalog.chapters(&novel);
        (catalog, chapters)
    }

    #[test]
    fn sorts_by_index_ascending_and_descending() {
        let (_c, chapters) = fixture();
        let mut refs: Vec<&Chapter> = chapters.iter().collect();
        sort_chapters(&mut refs, ChapterSort::IndexAsc);
        let asc: Vec<u32> = refs.iter().map(|c| c.index).collect();
        let mut sorted = asc.clone();
        sorted.sort_unstable();
        assert_eq!(asc, sorted);

        sort_chapters(&mut refs, ChapterSort::IndexDesc);
        let desc: Vec<u32> = refs.iter().map(|c| c.index).collect();
        let mut sorted = desc.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(desc, sorted);
    }

    #[test]
    fn sorts_by_publication_date() {
        let (_c, chapters) = fixture();
        let mut refs: Vec<&Chapter> = chapters.iter().collect();
        sort_chapters(&mut refs, ChapterSort::DateDesc);
        let dated: Vec<_> = refs.iter().filter(|c| c.published_at.is_some()).collect();
        for w in dated.windows(2) {
            assert!(w[0].published_at >= w[1].published_at);
        }
    }

    #[test]
    fn filters_unread_only() {
        let (_c, chapters) = fixture();
        let read: HashSet<ChapterId> = chapters.iter().take(3).map(|c| c.id).collect();
        let filter = ChapterFilter {
            unread_only: true,
            downloaded_only: false,
        };
        let out = filter_chapters(chapters.iter(), &filter, |id| read.contains(id), |_| false);
        assert_eq!(out.len(), chapters.len() - 3);
        assert!(out.iter().all(|c| !read.contains(&c.id)));
    }

    #[test]
    fn filters_downloaded_only() {
        let (_c, chapters) = fixture();
        let downloaded: HashSet<ChapterId> = chapters.iter().take(2).map(|c| c.id).collect();
        let filter = ChapterFilter {
            unread_only: false,
            downloaded_only: true,
        };
        let out = filter_chapters(
            chapters.iter(),
            &filter,
            |_| false,
            |id| downloaded.contains(id),
        );
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn combined_filters_intersect() {
        let (_c, chapters) = fixture();
        let read: HashSet<ChapterId> = chapters.iter().take(4).map(|c| c.id).collect();
        // Chapters 1 and 2 are both read AND downloaded; 3,4 read only.
        let downloaded: HashSet<ChapterId> = chapters
            .iter()
            .take(2)
            .chain(chapters.iter().skip(4).take(2))
            .map(|c| c.id)
            .collect();
        let filter = ChapterFilter {
            unread_only: true,
            downloaded_only: true,
        };
        let out = filter_chapters(
            chapters.iter(),
            &filter,
            |id| read.contains(id),
            |id| downloaded.contains(id),
        );
        assert_eq!(out.len(), 2);
        assert!(
            out.iter()
                .all(|c| !read.contains(&c.id) && downloaded.contains(&c.id))
        );
    }

    #[test]
    fn selection_is_tracked() {
        let (_c, chapters) = fixture();
        let mut state = ChapterListState::new();
        assert!(state.selected.is_none());
        state.select(chapters[0].id);
        assert_eq!(state.selected, Some(chapters[0].id));
    }
}
