//! The root application state: navigation + screen states + services.
//!
//! `AppState` is the single source of truth the Makepad UI renders from.
//! All mutating operations are methods here (the "update" half of the
//! Elm-style loop) so the UI layer stays a thin projection.

use readmesh_core::{ChapterId, NovelId};

use crate::chapters::{ChapterListState, filter_chapters, sort_chapters};
use crate::downloads::DownloadManager;
use crate::library::{ContinueReadingItem, LibraryState};
use crate::mock::MockCatalog;
use crate::navigation::{NavigationState, PrimaryTab, Route};
use crate::reader::ReaderState;
use crate::repository::ContentRepository;
use crate::search::{SearchState, run_search};
use crate::settings::AppSettings;

/// Fraction of a download completed per UI timer tick.
pub const DOWNLOAD_TICK_RATE: f32 = 0.04;

/// Root application state.
pub struct AppState {
    pub nav: NavigationState,
    pub search: SearchState,
    pub library: LibraryState,
    pub chapter_list: ChapterListState,
    pub downloads: DownloadManager,
    pub reader: ReaderState,
    pub settings: AppSettings,
    /// Content catalog behind the repository trait. Concrete today
    /// (`MockCatalog`), swappable for a daemon-backed implementation later.
    pub catalog: MockCatalog,
}

impl AppState {
    /// An empty app with the mock catalog (nothing in the library yet).
    pub fn new() -> Self {
        Self {
            nav: NavigationState::new(),
            search: SearchState::new(),
            library: LibraryState::new(),
            chapter_list: ChapterListState::new(),
            downloads: DownloadManager::new(),
            reader: ReaderState::new(),
            settings: AppSettings::new(),
            catalog: MockCatalog::demo(),
        }
    }

    /// A pre-seeded demo state: some library novels, favorites, progress and
    /// downloads so every screen has meaningful content on first launch.
    pub fn demo() -> Self {
        let mut state = Self::new();

        let trending = state.catalog.trending(6);
        let recent = state.catalog.recently_updated(8);
        // Library: a mix of trending + recently updated novels.
        let mut library_ids: Vec<NovelId> = Vec::new();
        for id in trending.into_iter().chain(recent) {
            if !library_ids.contains(&id) {
                library_ids.push(id);
            }
            if library_ids.len() == 7 {
                break;
            }
        }
        for id in &library_ids {
            let novel = state.catalog.novel(id).expect("demo novel");
            let chapters = state.catalog.chapters(id);
            state.library.add_to_library(novel, chapters);
        }

        // Favorites.
        for id in library_ids.iter().take(3) {
            state.library.toggle_favorite(id);
        }

        // Reading progress on a few novels.
        for (i, id) in library_ids.iter().take(4).enumerate() {
            let chapters = state.catalog.chapters(id);
            let at = ((i + 1) * 2).min(chapters.len() - 1);
            let chapter = &chapters[at];
            state.library.record_progress(id, &chapter.id, 0);
        }

        // Downloads: some completed, one active, one that will fail once.
        let first = library_ids[0];
        let second = library_ids[1];
        let first_chapters = state.catalog.chapters(&first);
        let first_novel = state.catalog.novel(&first).expect("novel");
        for chapter in first_chapters.iter().take(3) {
            state.downloads.enqueue(
                chapter.id,
                first,
                first_novel.title.clone(),
                chapter.index,
                chapter.title.clone(),
                state.catalog.chapter_size(&chapter.id),
            );
        }
        // Run them to completion.
        for _ in 0..40 {
            state.downloads.tick(0.25);
        }
        // An active download + a doomed one on the second novel.
        let second_chapters = state.catalog.chapters(&second);
        let second_novel = state.catalog.novel(&second).expect("novel");
        if let Some(chapter) = second_chapters.first() {
            state.downloads.enqueue(
                chapter.id,
                second,
                second_novel.title.clone(),
                chapter.index,
                chapter.title.clone(),
                state.catalog.chapter_size(&chapter.id),
            );
        }
        if let Some(chapter) = second_chapters.get(1) {
            state.downloads.enqueue(
                chapter.id,
                second,
                second_novel.title.clone(),
                chapter.index,
                chapter.title.clone(),
                state.catalog.chapter_size(&chapter.id),
            );
            state.downloads.seed_failure(chapter.id, 0.5);
        }
        for _ in 0..3 {
            state.downloads.tick(0.25);
        }

        // Simulate a plausible cache size.
        state.settings.storage.cache_used_bytes = 148 * 1024 * 1024;

        state
    }

    // ---- navigation commands ----------------------------------------------

    pub fn select_tab(&mut self, tab: PrimaryTab) {
        self.nav.select_tab(tab);
        // Sync settings reader prefs back out when leaving the reader.
        if !matches!(self.nav.current(), Route::Reader { .. }) {
            self.settings.reader = self.reader.settings.clone();
        }
    }

    pub fn go_back(&mut self) -> bool {
        if matches!(self.nav.current(), Route::Reader { .. }) {
            self.reader.close();
        }
        self.nav.back()
    }

    pub fn open_novel(&mut self, novel: NovelId) {
        self.chapter_list.selected = None;
        self.chapter_list.filter = Default::default();
        self.nav.open_novel(novel);
    }

    // ---- reading -------------------------------------------------------------

    /// Start (or continue) reading a novel: opens the reader at the resume
    /// chapter and records progress.
    pub fn start_reading(&mut self, novel: NovelId) {
        let Some(chapter) = self
            .library
            .resume_chapter(&novel)
            .or_else(|| self.catalog.chapters(&novel).first().map(|c| c.id))
        else {
            return;
        };
        self.open_chapter(novel, chapter);
    }

    /// Open a specific chapter in the reader.
    pub fn open_chapter(&mut self, novel: NovelId, chapter: ChapterId) {
        // Ensure chapters are known to the library so progress works even
        // for novels that are not in the library yet.
        if !self.library.contains(&novel)
            && let Some(n) = self.catalog.novel(&novel)
        {
            self.library
                .add_to_library(n, self.catalog.chapters(&novel));
        }
        self.library.mark_read(&chapter);
        self.library.record_progress(&novel, &chapter, 0);
        self.reader.open(novel, chapter);
        self.chapter_list.select(chapter);
        self.nav.open_reader(novel, chapter);
    }

    /// The novel's chapters in reading order (ids).
    pub fn reading_order(&self, novel: &NovelId) -> Vec<ChapterId> {
        self.catalog
            .chapters(novel)
            .into_iter()
            .map(|c| c.id)
            .collect()
    }

    /// Advance to the next chapter from the reader.
    pub fn reader_next_chapter(&mut self) -> bool {
        let Some((novel, current)) = self.reader.current else {
            return false;
        };
        let order = self.reading_order(&novel);
        if let Some(next) = self.reader.next_chapter(&order) {
            let _ = current;
            self.library.mark_read(&next);
            self.library.record_progress(&novel, &next, 0);
            self.chapter_list.select(next);
            self.nav.open_reader(novel, next);
            true
        } else {
            false
        }
    }

    /// Go to the previous chapter from the reader.
    pub fn reader_prev_chapter(&mut self) -> bool {
        let Some((novel, _)) = self.reader.current else {
            return false;
        };
        let order = self.reading_order(&novel);
        if let Some(prev) = self.reader.prev_chapter(&order) {
            self.library.record_progress(&novel, &prev, 0);
            self.chapter_list.select(prev);
            self.nav.open_reader(novel, prev);
            true
        } else {
            false
        }
    }

    // ---- library commands ------------------------------------------------------

    pub fn toggle_favorite(&mut self, novel: &NovelId) -> bool {
        self.library.toggle_favorite(novel)
    }

    pub fn toggle_library_membership(&mut self, novel: &NovelId) -> bool {
        if self.library.contains(novel) {
            self.library.remove_from_library(novel);
            false
        } else if let Some(n) = self.catalog.novel(novel) {
            self.library.add_to_library(n, self.catalog.chapters(novel));
            true
        } else {
            false
        }
    }

    pub fn mark_chapter_read(&mut self, chapter: &ChapterId, read: bool) {
        if read {
            self.library.mark_read(chapter);
        } else {
            self.library.mark_unread(chapter);
        }
    }

    pub fn continue_reading(&self) -> Vec<ContinueReadingItem> {
        self.library.continue_reading()
    }

    // ---- chapter list ----------------------------------------------------------

    /// The chapter list for the detail screen with sort + filter applied.
    pub fn visible_chapters(&self, novel: &NovelId) -> Vec<readmesh_core::Chapter> {
        let chapters = self.catalog.chapters(novel);
        let mut refs: Vec<&readmesh_core::Chapter> = chapters.iter().collect();
        sort_chapters(&mut refs, self.chapter_list.sort);
        let filtered = filter_chapters(
            refs,
            &self.chapter_list.filter,
            |id| self.library.is_read(id),
            |id| self.downloads.is_downloaded(id),
        );
        filtered.into_iter().cloned().collect()
    }

    // ---- search ------------------------------------------------------------------

    /// Begin a search: enters the `Loading` phase. Completion happens in
    /// [`AppState::finish_search`], which the UI calls after a tick — this
    /// mirrors how a future async backend will deliver results.
    pub fn begin_search(&mut self, query: &str) -> bool {
        self.search.submit(query)
    }

    /// Complete a pending search against the repository.
    ///
    /// Queries prefixed with `!` deterministically fail (a development
    /// trigger for exercising the error state without a backend).
    pub fn finish_search(&mut self) {
        if !self.search.is_loading() {
            return;
        }
        if self.search.query.starts_with('!') {
            self.search.fail("source unreachable (simulated)");
            return;
        }
        let results = run_search(&self.catalog, &self.search);
        self.search.complete(results);
    }

    /// Submit a search query and synchronously complete it (convenience
    /// for tests and non-UI callers).
    pub fn submit_search(&mut self, query: &str) {
        if self.begin_search(query) {
            self.finish_search();
        }
    }

    /// Re-run the active query (sort/filter changed, or retry).
    pub fn refresh_search(&mut self) {
        if self.search.refresh() {
            self.finish_search();
        }
    }

    // ---- downloads -----------------------------------------------------------------

    /// Queue a single chapter for download.
    pub fn download_chapter(&mut self, novel: &NovelId, chapter: &ChapterId) -> bool {
        let Some(ch) = self
            .catalog
            .chapters(novel)
            .into_iter()
            .find(|c| &c.id == chapter)
        else {
            return false;
        };
        let title = self
            .catalog
            .novel(novel)
            .map(|n| n.title)
            .unwrap_or_default();
        let size = self.catalog.chapter_size(chapter);
        self.downloads
            .enqueue(*chapter, *novel, title, ch.index, ch.title, size)
    }

    /// Queue multiple chapters (e.g. all unread) of a novel.
    pub fn download_chapters(&mut self, novel: &NovelId, chapters: &[ChapterId]) -> usize {
        chapters
            .iter()
            .filter(|c| self.download_chapter(novel, c))
            .count()
    }

    /// Download all not-yet-downloaded chapters of a novel. Returns the
    /// number of newly queued downloads.
    pub fn download_all(&mut self, novel: &NovelId) -> usize {
        let pending: Vec<ChapterId> = self
            .catalog
            .chapters(novel)
            .iter()
            .filter(|c| !self.downloads.is_downloaded(&c.id))
            .map(|c| c.id)
            .collect();
        self.download_chapters(novel, &pending)
    }

    /// Advance the download simulation by one tick.
    pub fn tick_downloads(&mut self) {
        let rate = DOWNLOAD_TICK_RATE;
        self.downloads.tick(rate);
        let used = self
            .downloads
            .items()
            .iter()
            .filter(|d| matches!(d.status, crate::downloads::DownloadStatus::Completed))
            .map(|d| d.size_bytes)
            .sum();
        self.settings.storage.cache_used_bytes = used;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloads::DownloadStatus;

    #[test]
    fn demo_state_has_content_for_every_screen() {
        let state = AppState::demo();
        assert!(state.library.novel_count() >= 5);
        assert!(!state.library.favorites().is_empty());
        assert!(!state.continue_reading().is_empty());
        let counts = state.downloads.counts();
        assert!(counts.completed > 0);
        assert!(counts.downloading + counts.queued > 0);
    }

    #[test]
    fn start_reading_opens_reader_and_records_progress() {
        let mut state = AppState::demo();
        let novel = state.library.all_sorted()[0];
        state.start_reading(novel);
        assert!(state.reader.is_open());
        assert!(matches!(state.nav.current(), Route::Reader { .. }));
        assert_eq!(
            state.library.resume_chapter(&novel),
            state.reader.current.map(|(_, c)| c)
        );
    }

    #[test]
    fn reader_chapter_navigation_flows_through_state() {
        let mut state = AppState::demo();
        let novel = state.library.all_sorted()[0];
        state.start_reading(novel);
        let first = state.reader.current.unwrap().1;
        assert!(state.reader_next_chapter());
        let second = state.reader.current.unwrap().1;
        assert_ne!(first, second);
        assert!(state.library.is_read(&second));
        assert!(state.reader_prev_chapter());
        assert_eq!(state.reader.current.unwrap().1, first);
    }

    #[test]
    fn leaving_reader_via_back_closes_it() {
        let mut state = AppState::demo();
        let novel = state.library.all_sorted()[0];
        state.start_reading(novel);
        assert!(state.go_back());
        assert!(!state.reader.is_open());
        assert!(matches!(state.nav.current(), Route::Tab(_)));
    }

    #[test]
    fn toggle_library_membership() {
        let mut state = AppState::demo();
        let novel = state.catalog.trending(1)[0];
        let was_in = state.library.contains(&novel);
        let now_in = state.toggle_library_membership(&novel);
        assert_eq!(now_in, !was_in);
        assert_eq!(state.library.contains(&novel), now_in);
    }

    #[test]
    fn download_all_queues_pending_chapters() {
        let mut state = AppState::demo();
        let novel = state.catalog.recently_added(1)[0];
        let total = state.catalog.chapters(&novel).len();
        let queued = state.download_all(&novel);
        assert_eq!(queued, total);
        // Second call queues nothing new.
        assert_eq!(state.download_all(&novel), 0);
    }

    #[test]
    fn download_tick_advances_simulation() {
        let mut state = AppState::demo();
        let before = state.downloads.counts().completed;
        for _ in 0..100 {
            state.tick_downloads();
        }
        let after = state.downloads.counts();
        assert!(after.completed > before || after.failed > 0);
    }

    #[test]
    fn visible_chapters_respects_filters() {
        let mut state = AppState::demo();
        let novel = state.library.all_sorted()[0];
        let all = state.visible_chapters(&novel);
        assert!(!all.is_empty());

        state.chapter_list.filter.unread_only = true;
        let unread = state.visible_chapters(&novel);
        assert!(unread.iter().all(|c| !state.library.is_read(&c.id)));

        state.chapter_list.filter.unread_only = false;
        state.chapter_list.filter.downloaded_only = true;
        let downloaded = state.visible_chapters(&novel);
        assert!(
            downloaded
                .iter()
                .all(|c| state.downloads.is_downloaded(&c.id))
        );
    }

    #[test]
    fn submit_search_end_to_end() {
        let mut state = AppState::demo();
        state.submit_search("moon");
        assert!(matches!(
            state.search.phase,
            crate::search::SearchPhase::Ready
        ));
        assert!(!state.search.results.is_empty());
    }

    #[test]
    fn open_chapter_adds_non_library_novel_to_library() {
        let mut state = AppState::new();
        let novel = state.catalog.all_novels()[0].id;
        assert!(!state.library.contains(&novel));
        let chapter = state.catalog.chapters(&novel)[0].id;
        state.open_chapter(novel, chapter);
        assert!(state.library.contains(&novel));
        assert!(state.library.is_read(&chapter));
    }

    #[test]
    fn failed_download_can_be_retried_through_state() {
        let mut state = AppState::demo();
        // Drive the seeded failure.
        for _ in 0..20 {
            state.tick_downloads();
        }
        let failed: Vec<ChapterId> = state
            .downloads
            .items()
            .iter()
            .filter(|d| matches!(d.status, DownloadStatus::Failed(_)))
            .map(|d| d.chapter_id)
            .collect();
        assert!(!failed.is_empty(), "demo seeds a failing download");
        assert!(state.downloads.retry(&failed[0]));
    }
}
