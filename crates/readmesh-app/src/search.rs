//! Search state and query logic, separated from the visual UI.
//!
//! The search flow is an explicit state machine:
//! `Idle -> Loading -> Ready | Empty | Error`, plus history, filtering and
//! sorting. Everything here is unit tested without a UI.

use std::collections::VecDeque;

use readmesh_core::{Novel, NovelId, NovelStatus};

use crate::repository::ContentRepository;

/// Maximum number of remembered searches.
pub const SEARCH_HISTORY_LIMIT: usize = 10;

/// How search results are ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchSort {
    /// Best textual match first (default).
    #[default]
    Relevance,
    TitleAsc,
    TitleDesc,
    RecentlyUpdated,
}

/// Optional filters applied on top of the text query.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchFilter {
    pub genre: Option<String>,
    pub status: Option<NovelStatus>,
    /// Restrict results to a single source plugin id (as string).
    pub source: Option<String>,
}

impl SearchFilter {
    pub fn is_active(&self) -> bool {
        self.genre.is_some() || self.status.is_some() || self.source.is_some()
    }
}

/// The phase of the search state machine.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SearchPhase {
    /// No search performed yet.
    #[default]
    Idle,
    /// A query has been submitted and results are being produced.
    Loading,
    /// Results are available (possibly empty — inspect `results`).
    Ready,
    /// The search failed.
    Error(String),
}

/// Search UI state: query, phase machine, results, history, sort & filter.
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub phase: SearchPhase,
    pub results: Vec<NovelId>,
    /// Most-recent-first list of past queries.
    pub history: VecDeque<String>,
    pub sort: SearchSort,
    pub filter: SearchFilter,
}

impl SearchState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit a query. Empty/whitespace queries reset to `Idle` and are not
    /// recorded in history. Returns `true` if a search was started.
    pub fn submit(&mut self, query: &str) -> bool {
        let query = query.trim();
        if query.is_empty() {
            self.query.clear();
            self.phase = SearchPhase::Idle;
            self.results.clear();
            return false;
        }
        self.query = query.to_string();
        self.phase = SearchPhase::Loading;
        self.push_history(query);
        true
    }

    /// Complete a pending search with ranked results. Transitions to `Ready`
    /// regardless of whether the result list is empty (the UI renders the
    /// empty state when `results.is_empty()`).
    pub fn complete(&mut self, results: Vec<NovelId>) {
        if matches!(self.phase, SearchPhase::Loading) {
            self.results = results;
            self.phase = SearchPhase::Ready;
        }
    }

    /// Fail a pending search with an error message.
    pub fn fail(&mut self, error: impl Into<String>) {
        if matches!(self.phase, SearchPhase::Loading) {
            self.results.clear();
            self.phase = SearchPhase::Error(error.into());
        }
    }

    /// Re-run the current query (e.g. after changing sort/filter or on
    /// retry from the error state). No-op without an active query.
    pub fn refresh(&mut self) -> bool {
        if self.query.is_empty() {
            return false;
        }
        self.phase = SearchPhase::Loading;
        true
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.phase, SearchPhase::Loading)
    }

    /// `true` when a completed search produced no results.
    pub fn is_empty_result(&self) -> bool {
        matches!(self.phase, SearchPhase::Ready) && self.results.is_empty()
    }

    fn push_history(&mut self, query: &str) {
        self.history.retain(|q| q != query);
        self.history.push_front(query.to_string());
        while self.history.len() > SEARCH_HISTORY_LIMIT {
            self.history.pop_back();
        }
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn set_sort(&mut self, sort: SearchSort) {
        self.sort = sort;
    }
}

/// Run a search against a repository: text query + filters + sorting.
///
/// This is a pure function so both the UI and tests drive identical logic.
pub fn run_search<R: ContentRepository + ?Sized>(repo: &R, state: &SearchState) -> Vec<NovelId> {
    let mut ids = repo.search(&state.query);
    apply_filter(repo, &mut ids, &state.filter);
    apply_sort(repo, &mut ids, state.sort);
    ids
}

fn apply_filter<R: ContentRepository + ?Sized>(
    repo: &R,
    ids: &mut Vec<NovelId>,
    filter: &SearchFilter,
) {
    if !filter.is_active() {
        return;
    }
    ids.retain(|id| {
        let Some(novel) = repo.novel(id) else {
            return false;
        };
        if let Some(genre) = &filter.genre
            && !novel
                .tags
                .iter()
                .any(|t| t.name.eq_ignore_ascii_case(genre))
        {
            return false;
        }
        if let Some(status) = &filter.status
            && novel.status != *status
        {
            return false;
        }
        if let Some(source) = &filter.source
            && !novel
                .source_refs
                .iter()
                .any(|r| r.plugin_id.0.eq_ignore_ascii_case(source))
        {
            return false;
        }
        true
    });
}

fn apply_sort<R: ContentRepository + ?Sized>(repo: &R, ids: &mut [NovelId], sort: SearchSort) {
    match sort {
        // The repository already returns relevance-ranked ids.
        SearchSort::Relevance => {}
        SearchSort::TitleAsc => ids.sort_by_key(|id| title_key(repo.novel(id))),
        SearchSort::TitleDesc => {
            ids.sort_by_key(|id| std::cmp::Reverse(title_key(repo.novel(id))));
        }
        SearchSort::RecentlyUpdated => {
            ids.sort_by_key(|id| std::cmp::Reverse(repo.novel(id).map(|n| n.updated_at)));
        }
    }
}

fn title_key(novel: Option<Novel>) -> String {
    novel.map(|n| n.title.to_lowercase()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockCatalog;

    #[test]
    fn empty_query_resets_to_idle_and_skips_history() {
        let mut s = SearchState::new();
        assert!(!s.submit(""));
        assert!(!s.submit("   "));
        assert_eq!(s.phase, SearchPhase::Idle);
        assert!(s.history.is_empty());
        assert!(s.results.is_empty());
    }

    #[test]
    fn submit_enters_loading_and_records_history() {
        let mut s = SearchState::new();
        assert!(s.submit("shadow"));
        assert_eq!(s.phase, SearchPhase::Loading);
        assert_eq!(s.query, "shadow");
        assert_eq!(s.history.front().map(String::as_str), Some("shadow"));
    }

    #[test]
    fn complete_transitions_to_ready_with_results() {
        let catalog = MockCatalog::demo();
        let mut s = SearchState::new();
        assert!(s.submit("the"));
        let results = run_search(&catalog, &s);
        assert!(!results.is_empty());
        s.complete(results.clone());
        assert_eq!(s.phase, SearchPhase::Ready);
        assert_eq!(s.results, results);
        assert!(!s.is_empty_result());
    }

    #[test]
    fn search_with_no_hits_yields_empty_state() {
        let catalog = MockCatalog::demo();
        let mut s = SearchState::new();
        assert!(s.submit("zzz-no-such-novel-zzz"));
        let results = run_search(&catalog, &s);
        s.complete(results);
        assert_eq!(s.phase, SearchPhase::Ready);
        assert!(s.is_empty_result());
    }

    #[test]
    fn error_state_is_reachable_and_retryable() {
        let mut s = SearchState::new();
        s.submit("shadow");
        s.fail("network unreachable");
        assert_eq!(s.phase, SearchPhase::Error("network unreachable".into()));
        assert!(s.results.is_empty());

        assert!(s.refresh());
        assert!(s.is_loading());
    }

    #[test]
    fn history_dedupes_and_caps_at_limit() {
        let mut s = SearchState::new();
        for i in 0..15 {
            s.submit(&format!("query-{i}"));
        }
        assert_eq!(s.history.len(), SEARCH_HISTORY_LIMIT);
        assert_eq!(s.history.front().map(String::as_str), Some("query-14"));

        s.submit("query-14");
        assert_eq!(
            s.history
                .iter()
                .filter(|q| q.as_str() == "query-14")
                .count(),
            1
        );
        assert_eq!(s.history.front().map(String::as_str), Some("query-14"));
    }

    #[test]
    fn filtering_by_genre() {
        let catalog = MockCatalog::demo();
        let mut s = SearchState::new();
        s.submit("");
        s.query.clear();
        // Use an empty text query with a genre filter: all novels of that genre.
        s.filter.genre = Some("Fantasy".into());
        let results = run_search(&catalog, &s);
        assert!(!results.is_empty());
        for id in &results {
            let novel = catalog.novel(id).expect("novel exists");
            assert!(novel.tags.iter().any(|t| t.name == "Fantasy"));
        }
    }

    #[test]
    fn filtering_by_status() {
        let catalog = MockCatalog::demo();
        let mut s = SearchState::new();
        s.filter.status = Some(NovelStatus::Completed);
        let results = run_search(&catalog, &s);
        assert!(!results.is_empty());
        for id in &results {
            assert_eq!(catalog.novel(id).unwrap().status, NovelStatus::Completed);
        }
    }

    #[test]
    fn sorting_by_title() {
        let catalog = MockCatalog::demo();
        let mut s = SearchState::new();
        s.set_sort(SearchSort::TitleAsc);
        let results = run_search(&catalog, &s);
        let titles: Vec<String> = results
            .iter()
            .map(|id| catalog.novel(id).unwrap().title.to_lowercase())
            .collect();
        let mut sorted = titles.clone();
        sorted.sort();
        assert_eq!(titles, sorted);
    }

    #[test]
    fn sorting_by_recently_updated() {
        let catalog = MockCatalog::demo();
        let mut s = SearchState::new();
        s.set_sort(SearchSort::RecentlyUpdated);
        let results = run_search(&catalog, &s);
        let stamps: Vec<_> = results
            .iter()
            .map(|id| catalog.novel(id).unwrap().updated_at)
            .collect();
        let mut sorted = stamps.clone();
        sorted.sort_by_key(|t| std::cmp::Reverse(*t));
        assert_eq!(stamps, sorted);
    }

    #[test]
    fn relevance_ranks_title_matches_first() {
        let catalog = MockCatalog::demo();
        let mut s = SearchState::new();
        s.submit("moon");
        let results = run_search(&catalog, &s);
        assert!(!results.is_empty());
        let first = catalog.novel(&results[0]).unwrap();
        assert!(
            first.title.to_lowercase().contains("moon"),
            "top relevance hit should match the title, got {}",
            first.title
        );
    }
}
