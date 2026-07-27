//! Reader screen: a distraction-free, full-screen reading experience.
//!
//! Chapter text with configurable typography and reader themes (dark /
//! light / sepia), overlay controls that toggle on tap, previous/next
//! chapter navigation and a chapter position indicator. All behavior is
//! driven by `readmesh_app::reader` (unit tested).

use makepad_widgets::*;
use readmesh_app::{ContentRepository, ReaderTheme, Route};
use readmesh_core::ChapterId;

use crate::app::AppAction;
use crate::components::RmTapWidgetExt;
use crate::state::{state, with_state_mut};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.ReaderScreen = #(ReaderScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Overlay

        // Base layer: the scrollable chapter text. The background color is
        // drawn by the ReaderScreen widget itself (reader theme).
        reader_bg := View{
            width: Fill height: Fill

            ScrollYView{
                width: Fill height: Fill
                flow: Down
                new_batch: true

                content_tap := RmTap{
                    width: Fill height: Fit
                    flow: Down spacing: theme.space_3
                    padding: Inset{top: 84 bottom: 120 left: 28 right: 28}
                    cursor: MouseCursor.Default
                    show_bg: false

                    chapter_title := Label{
                        width: Fill
                        text: "Chapter"
                        draw_text.color: mod.rm.reader_dark_text
                        draw_text.text_style: theme.font_bold{font_size: theme.font_size_3}
                    }
                    body_text := Label{
                        width: Fill
                        text: ""
                        draw_text.color: mod.rm.reader_dark_text
                        draw_text.text_style +: {
                            font_size: 18.0
                            line_spacing: 1.6
                        }
                    }
                    end_nav := View{
                        width: Fill height: Fit
                        flow: Right spacing: theme.space_2
                        align: Center
                        prev_button2 := RmSecondaryButton{text: "← Previous"}
                        next_button2 := RmPrimaryButton{text: "Next →"}
                    }
                }
            }
        }

        // Top bar (overlay).
        top_bar := SolidView{
            width: Fill height: Fit
            flow: Right spacing: theme.space_2
            align: Align{y: 0.5}
            padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
            draw_bg.color: #x000000b3
            new_batch: true
            close_button := RmSmallButton{text: "← Back"}
            novel_title := Label{
                width: Fill
                text: ""
                draw_text.color: #xfff
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
            }
            settings_button := RmSmallButton{text: "Aa"}
        }

        // Bottom bar (overlay).
        bottom_bar := SolidView{
            width: Fill height: Fill
            flow: Overlay
            draw_bg.color: #0000
            align: Align{x: 0.0 y: 1.0}
            bar_inner := SolidView{
                width: Fill height: Fit
                flow: Right spacing: theme.space_2
                align: Align{y: 0.5}
                padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
                draw_bg.color: #x000000b3
                new_batch: true
                prev_button := RmSmallButton{text: "← Prev"}
                progress_label := Label{
                    width: Fill
                    align: Center
                    text: ""
                    draw_text.color: #xfffd
                    draw_text.text_style.font_size: theme.font_size_p
                }
                next_button := RmSmallButton{text: "Next →"}
            }
        }

        // Reader settings panel (overlay, bottom sheet style).
        settings_panel := RoundedView{
            width: Fill height: Fit
            flow: Down spacing: theme.space_2
            padding: theme.mspace_3
            margin: Inset{left: 16 right: 16 bottom: 76}
            draw_bg.color: theme.color_fg_app
            draw_bg.border_radius: 12.0
            new_batch: true
            visible: false

            Label{
                text: "Reader settings"
                draw_text.color: theme.color_label_inner
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
            }

            View{
                width: Fill height: Fit
                flow: Right spacing: theme.space_2
                align: Align{y: 0.5}
                Label{
                    width: Fill
                    text: "Font size"
                    draw_text.color: theme.color_label_inner
                    draw_text.text_style.font_size: theme.font_size_p
                }
                font_dec := RmSmallButton{text: "A−"}
                font_size_label := Label{
                    text: "18"
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_p
                }
                font_inc := RmSmallButton{text: "A+"}
            }

            View{
                width: Fill height: Fit
                flow: Right spacing: theme.space_2
                align: Align{y: 0.5}
                Label{
                    width: Fill
                    text: "Line spacing"
                    draw_text.color: theme.color_label_inner
                    draw_text.text_style.font_size: theme.font_size_p
                }
                spacing_dec := RmSmallButton{text: "−"}
                spacing_label := Label{
                    text: "1.6"
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_p
                }
                spacing_inc := RmSmallButton{text: "+"}
            }

            View{
                width: Fill height: Fit
                flow: Right spacing: theme.space_2
                align: Align{y: 0.5}
                Label{
                    width: Fill
                    text: "Theme"
                    draw_text.color: theme.color_label_inner
                    draw_text.text_style.font_size: theme.font_size_p
                }
                theme_dark := RmSmallButton{text: "Dark"}
                theme_light := RmSmallButton{text: "Light"}
                theme_sepia := RmSmallButton{text: "Sepia"}
            }

            immersive_check := CheckBox{
                text: "Immersive mode (hide controls)"
            }
        }
    }
}

/// Reader theme palettes (deliberately independent of the app theme, so the
/// reader always offers dark/light/sepia regardless of global settings).
const READER_DARK_BG: Vec4f = vec4(0.070, 0.086, 0.109, 1.0);
const READER_DARK_TEXT: Vec4f = vec4(0.847, 0.870, 0.902, 1.0);
const READER_LIGHT_BG: Vec4f = vec4(0.980, 0.973, 0.953, 1.0);
const READER_LIGHT_TEXT: Vec4f = vec4(0.169, 0.192, 0.220, 1.0);
const READER_SEPIA_BG: Vec4f = vec4(0.953, 0.914, 0.843, 1.0);
const READER_SEPIA_TEXT: Vec4f = vec4(0.290, 0.247, 0.184, 1.0);

/// The full-screen Reader widget.
#[derive(Script, ScriptHook, Widget)]
pub struct ReaderScreen {
    #[deref]
    view: View,
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[rust]
    loaded_chapter: Option<ChapterId>,
    #[rust]
    panel_open: bool,
}

impl ReaderScreen {
    /// Sync all visual properties from reader state (theme, typography,
    /// controls visibility, labels).
    fn sync(&mut self, cx: &mut Cx2d) {
        let (theme, font_size, line_spacing, controls_visible, current) = {
            let s = state();
            (
                s.reader.settings.theme,
                s.reader.settings.font_size,
                s.reader.settings.line_spacing,
                s.reader.controls_visible,
                s.reader.current,
            )
        };

        // Reader theme colors (direct field mutation, no script eval).
        let (bg_color, text_color) = match theme {
            ReaderTheme::Dark => (READER_DARK_BG, READER_DARK_TEXT),
            ReaderTheme::Light => (READER_LIGHT_BG, READER_LIGHT_TEXT),
            ReaderTheme::Sepia => (READER_SEPIA_BG, READER_SEPIA_TEXT),
        };
        self.draw_bg.color = bg_color;
        if let Some(mut title) = self
            .view
            .label(cx, ids!(reader_bg.content_tap.chapter_title))
            .borrow_mut()
        {
            title.draw_text.color = text_color;
        }
        if let Some(mut body) = self
            .view
            .label(cx, ids!(reader_bg.content_tap.body_text))
            .borrow_mut()
        {
            body.draw_text.color = text_color;
            body.draw_text.text_style.font_size = font_size;
            body.draw_text.text_style.line_spacing = line_spacing;
        }

        // Controls visibility.
        self.view
            .view(cx, ids!(top_bar))
            .set_visible(cx, controls_visible);
        self.view
            .view(cx, ids!(bottom_bar.bar_inner))
            .set_visible(cx, controls_visible);
        self.view
            .view(cx, ids!(settings_panel))
            .set_visible(cx, controls_visible && self.panel_open);

        // Settings panel labels.
        let size_text = format!("{font_size:.0}");
        self.view
            .label(cx, ids!(settings_panel.font_size_label))
            .set_text(cx, &size_text);
        let spacing_text = format!("{line_spacing:.1}");
        self.view
            .label(cx, ids!(settings_panel.spacing_label))
            .set_text(cx, &spacing_text);

        // Chapter content + position indicator.
        if let Some((novel_id, chapter_id)) = current {
            if self.loaded_chapter != Some(chapter_id) {
                self.loaded_chapter = Some(chapter_id);
                let s = state();
                if let Some(content) = s.catalog.chapter_content(&chapter_id) {
                    self.view
                        .label(cx, ids!(reader_bg.content_tap.body_text))
                        .set_text(cx, &content);
                }
                let chapter = s
                    .catalog
                    .chapters(&novel_id)
                    .into_iter()
                    .find(|c| c.id == chapter_id);
                if let Some(chapter) = chapter {
                    self.view
                        .label(cx, ids!(reader_bg.content_tap.chapter_title))
                        .set_text(cx, &chapter.title);
                }
                let novel_title = s
                    .catalog
                    .novel(&novel_id)
                    .map(|n| n.title)
                    .unwrap_or_default();
                self.view
                    .label(cx, ids!(top_bar.novel_title))
                    .set_text(cx, &novel_title);
            }

            let s = state();
            let order = s.reading_order(&novel_id);
            let position = s.reader.position(&order).map(|p| p + 1).unwrap_or(0);
            let total = order.len();
            let progress_text = format!("Chapter {position} of {total}");
            self.view
                .label(cx, ids!(bottom_bar.bar_inner.progress_label))
                .set_text(cx, &progress_text);

            let has_prev = s.reader.has_prev(&order);
            let has_next = s.reader.has_next(&order);
            self.view
                .button(cx, ids!(bottom_bar.bar_inner.prev_button))
                .set_visible(cx, has_prev);
            self.view
                .button(cx, ids!(bottom_bar.bar_inner.next_button))
                .set_visible(cx, has_next);
            self.view
                .button(cx, ids!(reader_bg.content_tap.end_nav.prev_button2))
                .set_visible(cx, has_prev);
            self.view
                .button(cx, ids!(reader_bg.content_tap.end_nav.next_button2))
                .set_visible(cx, has_next);
        }
    }
}

impl Widget for ReaderScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Reset when the reader route is not active (e.g. reopened later).
        if !matches!(state().nav.current(), Route::Reader { .. }) {
            self.loaded_chapter = None;
            self.panel_open = false;
        }
        self.sync(cx);
        // Paint the reader theme background behind the content.
        let rect = cx.peek_walk_turtle(walk);
        self.draw_bg.draw_abs(cx, rect);
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ReaderScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Close the reader.
        if self
            .view
            .button(cx, ids!(top_bar.close_button))
            .clicked(actions)
        {
            with_state_mut(|s| {
                s.go_back();
            });
            cx.action(AppAction::StateChanged);
            return;
        }

        // Chapter navigation (bottom bar + end-of-chapter buttons).
        let next = self
            .view
            .button(cx, ids!(bottom_bar.bar_inner.next_button))
            .clicked(actions)
            || self
                .view
                .button(cx, ids!(reader_bg.content_tap.end_nav.next_button2))
                .clicked(actions);
        if next {
            with_state_mut(|s| {
                s.reader_next_chapter();
            });
            cx.action(AppAction::StateChanged);
            return;
        }
        let prev = self
            .view
            .button(cx, ids!(bottom_bar.bar_inner.prev_button))
            .clicked(actions)
            || self
                .view
                .button(cx, ids!(reader_bg.content_tap.end_nav.prev_button2))
                .clicked(actions);
        if prev {
            with_state_mut(|s| {
                s.reader_prev_chapter();
            });
            cx.action(AppAction::StateChanged);
            return;
        }

        // Tap the text area to toggle the controls overlay.
        if self
            .view
            .rm_tap(cx, ids!(reader_bg.content_tap))
            .clicked(actions)
        {
            with_state_mut(|s| s.reader.toggle_controls());
            cx.action(AppAction::StateChanged);
        }

        // Settings panel toggle.
        if self
            .view
            .button(cx, ids!(top_bar.settings_button))
            .clicked(actions)
        {
            self.panel_open = !self.panel_open;
            cx.action(AppAction::StateChanged);
        }

        // Typography controls (also persisted into AppSettings).
        if self
            .view
            .button(cx, ids!(settings_panel.font_inc))
            .clicked(actions)
        {
            with_state_mut(|s| {
                s.reader.settings.increase_font_size();
                s.settings.reader = s.reader.settings.clone();
            });
            cx.action(AppAction::StateChanged);
        }
        if self
            .view
            .button(cx, ids!(settings_panel.font_dec))
            .clicked(actions)
        {
            with_state_mut(|s| {
                s.reader.settings.decrease_font_size();
                s.settings.reader = s.reader.settings.clone();
            });
            cx.action(AppAction::StateChanged);
        }
        if self
            .view
            .button(cx, ids!(settings_panel.spacing_inc))
            .clicked(actions)
        {
            with_state_mut(|s| {
                s.reader.settings.increase_line_spacing();
                s.settings.reader = s.reader.settings.clone();
            });
            cx.action(AppAction::StateChanged);
        }
        if self
            .view
            .button(cx, ids!(settings_panel.spacing_dec))
            .clicked(actions)
        {
            with_state_mut(|s| {
                s.reader.settings.decrease_line_spacing();
                s.settings.reader = s.reader.settings.clone();
            });
            cx.action(AppAction::StateChanged);
        }

        // Reader themes.
        for (path, theme) in [
            (ids!(settings_panel.theme_dark), ReaderTheme::Dark),
            (ids!(settings_panel.theme_light), ReaderTheme::Light),
            (ids!(settings_panel.theme_sepia), ReaderTheme::Sepia),
        ] {
            if self.view.button(cx, path).clicked(actions) {
                with_state_mut(|s| {
                    s.reader.settings.set_theme(theme);
                    s.settings.reader = s.reader.settings.clone();
                });
                cx.action(AppAction::StateChanged);
            }
        }

        // Immersive mode.
        if let Some(immersive) = self
            .view
            .check_box(cx, ids!(settings_panel.immersive_check))
            .changed(actions)
        {
            with_state_mut(|s| {
                s.reader.settings.immersive = immersive;
                s.settings.reader = s.reader.settings.clone();
                s.reader.controls_visible = !immersive;
            });
            cx.action(AppAction::StateChanged);
        }
    }
}
