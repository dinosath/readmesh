//! Centralized, strongly-typed navigation state.
//!
//! The navigation model is intentionally free of any UI framework types so it
//! can be unit tested without a graphical environment. The Makepad shell in
//! `readmesh-ui` is a pure projection of this state.

use readmesh_core::{ChapterId, NovelId};

/// Primary destinations shown in the app shell (navigation rail on desktop,
/// bottom navigation bar on mobile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PrimaryTab {
    Library,
    Browse,
    Search,
    Downloads,
    Settings,
}

impl PrimaryTab {
    /// All tabs in the order they appear in the app shell.
    pub const ALL: [PrimaryTab; 5] = [
        PrimaryTab::Library,
        PrimaryTab::Browse,
        PrimaryTab::Search,
        PrimaryTab::Downloads,
        PrimaryTab::Settings,
    ];

    pub fn title(self) -> &'static str {
        match self {
            PrimaryTab::Library => "Library",
            PrimaryTab::Browse => "Browse",
            PrimaryTab::Search => "Search",
            PrimaryTab::Downloads => "Downloads",
            PrimaryTab::Settings => "Settings",
        }
    }
}

/// A typed route within the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// One of the primary tabs (root-level destination).
    Tab(PrimaryTab),
    /// Detail page for a novel (info + chapter list).
    NovelDetail(NovelId),
    /// Full-screen reader for a specific chapter.
    Reader { novel: NovelId, chapter: ChapterId },
}

impl Route {
    /// The primary tab this route is visually "under" in the shell.
    pub fn tab(&self) -> Option<PrimaryTab> {
        match self {
            Route::Tab(tab) => Some(*tab),
            Route::NovelDetail(_) | Route::Reader { .. } => None,
        }
    }

    /// The reader is a distraction-free experience that hides the app chrome
    /// (navigation rail / bottom bar).
    pub fn hides_chrome(&self) -> bool {
        matches!(self, Route::Reader { .. })
    }
}

/// Layout mode of the app shell, derived from the available window width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavMode {
    /// Narrow layout: bottom navigation bar, single column content.
    Mobile,
    /// Medium layout: compact navigation rail, single column content.
    Tablet,
    /// Wide layout: full navigation rail with labels, roomy content.
    Desktop,
}

impl NavMode {
    /// Breakpoints (logical pixels) between layout modes.
    pub const MOBILE_MAX: f64 = 640.0;
    pub const TABLET_MAX: f64 = 1024.0;

    pub fn from_width(width: f64) -> Self {
        if width < Self::MOBILE_MAX {
            NavMode::Mobile
        } else if width < Self::TABLET_MAX {
            NavMode::Tablet
        } else {
            NavMode::Desktop
        }
    }

    /// Mobile uses a bottom navigation bar; tablet/desktop use a side rail.
    pub fn uses_bottom_nav(self) -> bool {
        matches!(self, NavMode::Mobile)
    }
}

/// Central navigation state: the active primary tab plus a back stack of
/// detail routes pushed on top of it.
#[derive(Debug, Clone)]
pub struct NavigationState {
    tab: PrimaryTab,
    stack: Vec<Route>,
    mode: NavMode,
}

impl Default for NavigationState {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            tab: PrimaryTab::Library,
            stack: Vec::new(),
            mode: NavMode::Desktop,
        }
    }

    /// The route that should currently be rendered.
    pub fn current(&self) -> Route {
        self.stack.last().copied().unwrap_or(Route::Tab(self.tab))
    }

    /// The currently selected primary tab, regardless of pushed routes.
    pub fn current_tab(&self) -> PrimaryTab {
        self.tab
    }

    /// Select a primary tab. This clears the detail back stack, matching the
    /// behavior of bottom-navigation apps (LNReader included).
    pub fn select_tab(&mut self, tab: PrimaryTab) {
        self.tab = tab;
        self.stack.clear();
    }

    /// Push a detail route onto the stack.
    pub fn push(&mut self, route: Route) {
        if let Route::Tab(tab) = route {
            self.select_tab(tab);
        } else if self.current() != route {
            self.stack.push(route);
        }
    }

    /// Open the detail page for a novel.
    pub fn open_novel(&mut self, novel: NovelId) {
        self.push(Route::NovelDetail(novel));
    }

    /// Open the full-screen reader at a specific chapter.
    pub fn open_reader(&mut self, novel: NovelId, chapter: ChapterId) {
        self.push(Route::Reader { novel, chapter });
    }

    /// Go back one step. Returns `true` if a route was popped, `false` when
    /// already at the root of the current tab.
    pub fn back(&mut self) -> bool {
        self.stack.pop().is_some()
    }

    pub fn can_go_back(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Depth of the detail stack (0 = at a primary tab root).
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    pub fn mode(&self) -> NavMode {
        self.mode
    }

    /// Update the layout mode (called by the shell when the window resizes).
    /// Returns `true` if the mode changed.
    pub fn set_mode(&mut self, mode: NavMode) -> bool {
        if self.mode != mode {
            self.mode = mode;
            true
        } else {
            false
        }
    }

    /// Whether the current route hides the app chrome (reader mode).
    pub fn chrome_hidden(&self) -> bool {
        self.current().hides_chrome()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use readmesh_core::{ChapterId, NovelId};

    fn novel_id(n: u8) -> NovelId {
        NovelId(blake3_hash(n))
    }

    fn chapter_id(n: u8) -> ChapterId {
        ChapterId(blake3_hash(n))
    }

    fn blake3_hash(n: u8) -> blake3::Hash {
        blake3::hash(&[n; 32])
    }

    #[test]
    fn initial_route_is_library_tab() {
        let nav = NavigationState::new();
        assert_eq!(nav.current(), Route::Tab(PrimaryTab::Library));
        assert_eq!(nav.current_tab(), PrimaryTab::Library);
        assert!(!nav.can_go_back());
        assert_eq!(nav.depth(), 0);
    }

    #[test]
    fn tab_selection_switches_and_clears_stack() {
        let mut nav = NavigationState::new();
        let id = novel_id(1);
        nav.open_novel(id);
        assert_eq!(nav.current(), Route::NovelDetail(id));
        assert!(nav.can_go_back());

        nav.select_tab(PrimaryTab::Browse);
        assert_eq!(nav.current(), Route::Tab(PrimaryTab::Browse));
        assert!(
            !nav.can_go_back(),
            "selecting a tab clears the detail stack"
        );
    }

    #[test]
    fn push_and_back_navigation() {
        let mut nav = NavigationState::new();
        let novel = novel_id(7);
        let chapter = chapter_id(3);

        nav.open_novel(novel);
        nav.open_reader(novel, chapter);
        assert_eq!(nav.depth(), 2);
        assert_eq!(nav.current(), Route::Reader { novel, chapter });

        assert!(nav.back());
        assert_eq!(nav.current(), Route::NovelDetail(novel));

        assert!(nav.back());
        assert_eq!(nav.current(), Route::Tab(PrimaryTab::Library));

        assert!(!nav.back(), "back at root returns false");
        assert_eq!(nav.current(), Route::Tab(PrimaryTab::Library));
    }

    #[test]
    fn pushing_same_route_twice_is_a_noop() {
        let mut nav = NavigationState::new();
        let id = novel_id(1);
        nav.open_novel(id);
        nav.open_novel(id);
        assert_eq!(nav.depth(), 1);
    }

    #[test]
    fn mobile_navigation_state() {
        let mut nav = NavigationState::new();
        nav.set_mode(NavMode::Mobile);
        assert_eq!(nav.mode(), NavMode::Mobile);
        assert!(nav.mode().uses_bottom_nav());

        nav.select_tab(PrimaryTab::Downloads);
        assert_eq!(nav.current(), Route::Tab(PrimaryTab::Downloads));
        nav.open_novel(novel_id(9));
        assert!(nav.can_go_back());
        assert!(nav.back());
        assert_eq!(nav.current(), Route::Tab(PrimaryTab::Downloads));
    }

    #[test]
    fn desktop_navigation_state() {
        let mut nav = NavigationState::new();
        nav.set_mode(NavMode::Desktop);
        assert!(!nav.mode().uses_bottom_nav());
    }

    #[test]
    fn nav_mode_breakpoints() {
        assert_eq!(NavMode::from_width(320.0), NavMode::Mobile);
        assert_eq!(NavMode::from_width(639.9), NavMode::Mobile);
        assert_eq!(NavMode::from_width(640.0), NavMode::Tablet);
        assert_eq!(NavMode::from_width(1023.9), NavMode::Tablet);
        assert_eq!(NavMode::from_width(1024.0), NavMode::Desktop);
        assert_eq!(NavMode::from_width(1920.0), NavMode::Desktop);
    }

    #[test]
    fn mode_change_is_reported_once() {
        let mut nav = NavigationState::new();
        assert!(nav.set_mode(NavMode::Mobile));
        assert!(!nav.set_mode(NavMode::Mobile));
        assert!(nav.set_mode(NavMode::Desktop));
    }

    #[test]
    fn reader_hides_chrome() {
        let mut nav = NavigationState::new();
        assert!(!nav.chrome_hidden());
        nav.open_reader(novel_id(1), chapter_id(1));
        assert!(nav.chrome_hidden());
        nav.back();
        assert!(!nav.chrome_hidden());
    }

    #[test]
    fn invalid_back_at_root_keeps_state() {
        let mut nav = NavigationState::new();
        nav.select_tab(PrimaryTab::Settings);
        assert!(!nav.back());
        assert_eq!(nav.current(), Route::Tab(PrimaryTab::Settings));
    }
}
