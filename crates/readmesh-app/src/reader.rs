//! Reader state: current chapter, in-chapter progress, chapter navigation
//! and reader preferences. UI-independent and fully unit tested.

use readmesh_core::{ChapterId, NovelId};

/// Reader color themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReaderTheme {
    /// Dark, reader-friendly (default).
    #[default]
    Dark,
    Light,
    Sepia,
}

impl ReaderTheme {
    pub const ALL: [ReaderTheme; 3] = [ReaderTheme::Dark, ReaderTheme::Light, ReaderTheme::Sepia];

    pub fn name(self) -> &'static str {
        match self {
            ReaderTheme::Dark => "Dark",
            ReaderTheme::Light => "Light",
            ReaderTheme::Sepia => "Sepia",
        }
    }
}

/// Reader typography and display preferences.
#[derive(Debug, Clone, PartialEq)]
pub struct ReaderSettings {
    /// Body text size in logical pixels.
    pub font_size: f32,
    /// Line height multiplier.
    pub line_spacing: f32,
    pub theme: ReaderTheme,
    /// Immersive mode hides all chrome until tapped.
    pub immersive: bool,
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            font_size: 18.0,
            line_spacing: 1.6,
            theme: ReaderTheme::Dark,
            immersive: false,
        }
    }
}

impl ReaderSettings {
    pub const MIN_FONT_SIZE: f32 = 12.0;
    pub const MAX_FONT_SIZE: f32 = 32.0;
    pub const MIN_LINE_SPACING: f32 = 1.2;
    pub const MAX_LINE_SPACING: f32 = 2.4;

    pub fn increase_font_size(&mut self) {
        self.font_size = (self.font_size + 1.0).min(Self::MAX_FONT_SIZE);
    }

    pub fn decrease_font_size(&mut self) {
        self.font_size = (self.font_size - 1.0).max(Self::MIN_FONT_SIZE);
    }

    pub fn increase_line_spacing(&mut self) {
        self.line_spacing = (self.line_spacing + 0.1).min(Self::MAX_LINE_SPACING);
    }

    pub fn decrease_line_spacing(&mut self) {
        self.line_spacing = (self.line_spacing - 0.1).max(Self::MIN_LINE_SPACING);
    }

    pub fn set_theme(&mut self, theme: ReaderTheme) {
        self.theme = theme;
    }
}

/// State of the full-screen reader.
#[derive(Debug, Clone, Default)]
pub struct ReaderState {
    pub settings: ReaderSettings,
    /// The chapter currently being read.
    pub current: Option<(NovelId, ChapterId)>,
    /// Scroll progress within the current chapter, `0.0..=1.0`.
    pub chapter_progress: f32,
    /// Whether the reader controls overlay is visible.
    pub controls_visible: bool,
}

impl ReaderState {
    pub fn new() -> Self {
        Self {
            controls_visible: true,
            ..Self::default()
        }
    }

    pub fn open(&mut self, novel: NovelId, chapter: ChapterId) {
        self.current = Some((novel, chapter));
        self.chapter_progress = 0.0;
        self.controls_visible = !self.settings.immersive;
    }

    pub fn close(&mut self) {
        self.current = None;
        self.chapter_progress = 0.0;
    }

    pub fn is_open(&self) -> bool {
        self.current.is_some()
    }

    /// Advance to the next chapter in `ordered` (the novel's chapters in
    /// reading order). Returns the new chapter id, or `None` at the end.
    pub fn next_chapter(&mut self, ordered: &[ChapterId]) -> Option<ChapterId> {
        let (_, current) = self.current?;
        let pos = ordered.iter().position(|c| *c == current)?;
        let next = *ordered.get(pos + 1)?;
        self.current = Some((self.current?.0, next));
        self.chapter_progress = 0.0;
        Some(next)
    }

    /// Go back to the previous chapter. Returns the new chapter id, or
    /// `None` at the beginning.
    pub fn prev_chapter(&mut self, ordered: &[ChapterId]) -> Option<ChapterId> {
        let (_, current) = self.current?;
        let pos = ordered.iter().position(|c| *c == current)?;
        let prev = *ordered.get(pos.checked_sub(1)?)?;
        self.current = Some((self.current?.0, prev));
        self.chapter_progress = 0.0;
        Some(prev)
    }

    /// `true` when a next chapter exists in `ordered`.
    pub fn has_next(&self, ordered: &[ChapterId]) -> bool {
        let Some((_, current)) = self.current else {
            return false;
        };
        ordered
            .iter()
            .position(|c| *c == current)
            .is_some_and(|pos| pos + 1 < ordered.len())
    }

    /// `true` when a previous chapter exists in `ordered`.
    pub fn has_prev(&self, ordered: &[ChapterId]) -> bool {
        let Some((_, current)) = self.current else {
            return false;
        };
        ordered
            .iter()
            .position(|c| *c == current)
            .is_some_and(|pos| pos > 0)
    }

    /// Position of the current chapter within `ordered` (0-based).
    pub fn position(&self, ordered: &[ChapterId]) -> Option<usize> {
        let (_, current) = self.current?;
        ordered.iter().position(|c| *c == current)
    }

    /// Update in-chapter scroll progress (clamped to `0.0..=1.0`).
    pub fn set_progress(&mut self, progress: f32) {
        self.chapter_progress = progress.clamp(0.0, 1.0);
    }

    pub fn toggle_controls(&mut self) {
        self.controls_visible = !self.controls_visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chapter(n: u8) -> ChapterId {
        ChapterId(blake3::hash(&[n; 4]))
    }

    fn novel() -> NovelId {
        NovelId(blake3::hash(&[1u8; 4]))
    }

    fn ordered() -> Vec<ChapterId> {
        (0..5).map(chapter).collect()
    }

    #[test]
    fn open_sets_current_and_resets_progress() {
        let mut r = ReaderState::new();
        r.set_progress(0.7);
        r.open(novel(), chapter(1));
        assert_eq!(r.current, Some((novel(), chapter(1))));
        assert!((r.chapter_progress - 0.0).abs() < f32::EPSILON);
        assert!(r.is_open());
    }

    #[test]
    fn next_chapter_advances_and_resets_progress() {
        let mut r = ReaderState::new();
        let chapters = ordered();
        r.open(novel(), chapters[1]);
        r.set_progress(0.5);
        assert_eq!(r.next_chapter(&chapters), Some(chapters[2]));
        assert_eq!(r.current, Some((novel(), chapters[2])));
        assert!((r.chapter_progress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn next_chapter_at_end_returns_none() {
        let mut r = ReaderState::new();
        let chapters = ordered();
        r.open(novel(), chapters[4]);
        assert_eq!(r.next_chapter(&chapters), None);
        assert_eq!(r.current, Some((novel(), chapters[4])), "unchanged at end");
    }

    #[test]
    fn prev_chapter_goes_back() {
        let mut r = ReaderState::new();
        let chapters = ordered();
        r.open(novel(), chapters[2]);
        assert_eq!(r.prev_chapter(&chapters), Some(chapters[1]));
    }

    #[test]
    fn prev_chapter_at_start_returns_none() {
        let mut r = ReaderState::new();
        let chapters = ordered();
        r.open(novel(), chapters[0]);
        assert_eq!(r.prev_chapter(&chapters), None);
        assert!(!r.has_prev(&chapters));
        assert!(r.has_next(&chapters));
    }

    #[test]
    fn navigation_without_open_reader_is_safe() {
        let mut r = ReaderState::new();
        let chapters = ordered();
        assert_eq!(r.next_chapter(&chapters), None);
        assert_eq!(r.prev_chapter(&chapters), None);
        assert!(!r.has_next(&chapters));
        assert!(!r.has_prev(&chapters));
    }

    #[test]
    fn progress_is_clamped() {
        let mut r = ReaderState::new();
        r.set_progress(1.5);
        assert!((r.chapter_progress - 1.0).abs() < f32::EPSILON);
        r.set_progress(-0.2);
        assert!((r.chapter_progress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn chapter_position_is_reported() {
        let mut r = ReaderState::new();
        let chapters = ordered();
        r.open(novel(), chapters[3]);
        assert_eq!(r.position(&chapters), Some(3));
    }

    #[test]
    fn font_size_controls_are_bounded() {
        let mut s = ReaderSettings::default();
        for _ in 0..100 {
            s.increase_font_size();
        }
        assert!((s.font_size - ReaderSettings::MAX_FONT_SIZE).abs() < f32::EPSILON);
        for _ in 0..100 {
            s.decrease_font_size();
        }
        assert!((s.font_size - ReaderSettings::MIN_FONT_SIZE).abs() < f32::EPSILON);
    }

    #[test]
    fn line_spacing_controls_are_bounded() {
        let mut s = ReaderSettings::default();
        for _ in 0..100 {
            s.increase_line_spacing();
        }
        assert!(s.line_spacing <= ReaderSettings::MAX_LINE_SPACING);
        for _ in 0..100 {
            s.decrease_line_spacing();
        }
        assert!(s.line_spacing >= ReaderSettings::MIN_LINE_SPACING);
    }

    #[test]
    fn controls_toggle_and_immersive_open() {
        let mut r = ReaderState::new();
        assert!(r.controls_visible);
        r.toggle_controls();
        assert!(!r.controls_visible);

        r.settings.immersive = true;
        r.open(novel(), chapter(0));
        assert!(
            !r.controls_visible,
            "immersive mode opens with hidden controls"
        );
    }

    #[test]
    fn theme_switching() {
        let mut s = ReaderSettings::default();
        assert_eq!(s.theme, ReaderTheme::Dark);
        s.set_theme(ReaderTheme::Sepia);
        assert_eq!(s.theme, ReaderTheme::Sepia);
        s.set_theme(ReaderTheme::Light);
        assert_eq!(s.theme, ReaderTheme::Light);
    }
}
