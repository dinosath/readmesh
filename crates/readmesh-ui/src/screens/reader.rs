use makepad_widgets::*;
use readmesh_app::{ContentRepository, ReaderTheme, Route};

use crate::app::AppAction;
use crate::components::{RmTapWidgetExt, RmVerticalSliderWidgetExt};
use crate::state::{state, with_state_mut};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.ReaderScreen = #(ReaderScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Overlay

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

        // Top overlay bar.
        top_bar := SolidView{
            width: Fill height: Fit
            flow: Right spacing: theme.space_2
            align: Align{y: 0.5}
            padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
            draw_bg.color: #x000000b3
            new_batch: true
            back_btn := RmTap{
                width: 32 height: 32
                flow: Overlay
                align: Center
                show_bg: false
                cursor: MouseCursor.Hand
                IconBack{}
            }
            title_block := View{
                width: Fill height: Fit
                flow: Down spacing: 1
                novel_title := Label{
                    width: Fill
                    text: ""
                    draw_text.color: #xfff
                    draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
                }
                chapter_subtitle := Label{
                    width: Fill
                    text: ""
                    draw_text.color: #xfffd
                    draw_text.text_style.font_size: theme.font_size_code
                }
            }
            bookmark_btn := RmTap{
                width: 28 height: 28
                flow: Overlay
                align: Center
                show_bg: false
                cursor: MouseCursor.Hand
                IconBookmark{}
            }
        }

        // Bottom overlay bar (icon row).
        bottom_bar := SolidView{
            width: Fill height: 48
            flow: Overlay
            draw_bg.color: #0000
            align: Align{x: 0.0 y: 1.0}
            bar_inner := SolidView{
                width: Fill height: 48
                flow: Right
                align: Center
                draw_bg.color: #x000000b3
                new_batch: true
                prev_btn := RmTap{
                    width: 36 height: 36
                    flow: Overlay
                    align: Center
                    show_bg: false
                    cursor: MouseCursor.Hand
                    IconPrev{}
                }
                Filler{}
                translate_btn := RmTap{
                    width: 36 height: 36
                    flow: Overlay
                    align: Center
                    show_bg: false
                    cursor: MouseCursor.Hand
                    IconTranslate{}
                }
                Filler{}
                textsize_btn := RmTap{
                    width: 36 height: 36
                    flow: Overlay
                    align: Center
                    show_bg: false
                    cursor: MouseCursor.Hand
                    IconAa{}
                }
                Filler{}
                gear_btn := RmTap{
                    width: 36 height: 36
                    flow: Overlay
                    align: Center
                    show_bg: false
                    cursor: MouseCursor.Hand
                    IconGear{}
                }
                Filler{}
                next_btn := RmTap{
                    width: 36 height: 36
                    flow: Overlay
                    align: Center
                    show_bg: false
                    cursor: MouseCursor.Hand
                    IconNext{}
                }
            }
        }

        // Vertical chapter scrubber (right edge).
        scrubber := SolidView{
            width: Fit height: Fill
            flow: Overlay
            align: Align{x: 1.0 y: 0.5}
            draw_bg.color: #x00000000
            padding: Inset{left: 0 right: 0 top: 80 bottom: 80}
            scrub_inner := View{
                width: Fit height: Fill
                flow: Overlay
                align: Center
                scrub_slider := RmVerticalSlider{
                    width: 6 height: Fill
                    draw_track +: {
                        color: #xffffff22
                        border_radius: 3.0
                    }
                    draw_thumb +: {
                        color: theme.color_highlight
                        border_radius: 3.0
                    }
                }
                scrub_labels := View{
                    width: Fit height: Fill
                    flow: Down
                    align: Center
                    spacing: 4
                    scrub_current := Label{
                        text: "0"
                        draw_text.color: theme.color_highlight
                        draw_text.text_style: theme.font_bold{font_size: theme.font_size_code}
                    }
                    scrub_total := Label{
                        text: "0"
                        draw_text.color: #xfffa
                        draw_text.text_style.font_size: theme.font_size_code
                    }
                }
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

const READER_DARK_BG: Vec4f = vec4(0.070, 0.086, 0.109, 1.0);
const READER_DARK_TEXT: Vec4f = vec4(0.847, 0.870, 0.902, 1.0);
const READER_LIGHT_BG: Vec4f = vec4(0.980, 0.973, 0.953, 1.0);
const READER_LIGHT_TEXT: Vec4f = vec4(0.169, 0.192, 0.220, 1.0);
const READER_SEPIA_BG: Vec4f = vec4(0.953, 0.914, 0.843, 1.0);
const READER_SEPIA_TEXT: Vec4f = vec4(0.290, 0.247, 0.184, 1.0);

#[derive(Script, ScriptHook, Widget)]
pub struct ReaderScreen {
    #[deref]
    view: View,
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[rust]
    loaded_chapter: Option<readmesh_core::ChapterId>,
    #[rust]
    panel_open: bool,
    #[rust]
    last_novel_title: String,
    #[rust]
    last_chapter_index: u32,
    #[rust]
    last_total: usize,
}

impl ReaderScreen {
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

        self.view
            .view(cx, ids!(top_bar))
            .set_visible(cx, controls_visible);
        self.view
            .view(cx, ids!(bottom_bar.bar_inner))
            .set_visible(cx, controls_visible);
        self.view
            .view(cx, ids!(settings_panel))
            .set_visible(cx, controls_visible && self.panel_open);
        self.view
            .view(cx, ids!(scrubber))
            .set_visible(cx, controls_visible);

        let size_text = format!("{font_size:.0}");
        self.view
            .label(cx, ids!(settings_panel.font_size_label))
            .set_text(cx, &size_text);
        let spacing_text = format!("{line_spacing:.1}");
        self.view
            .label(cx, ids!(settings_panel.spacing_label))
            .set_text(cx, &spacing_text);

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
                    self.last_chapter_index = chapter.index;
                }
                let novel_title = s
                    .catalog
                    .novel(&novel_id)
                    .map(|n| n.title)
                    .unwrap_or_default();
                self.last_novel_title = novel_title.clone();
                self.view
                    .label(cx, ids!(top_bar.title_block.novel_title))
                    .set_text(cx, &novel_title);
            }

            let s = state();
            let order = s.reading_order(&novel_id);
            let position = s.reader.position(&order).map(|p| p + 1).unwrap_or(0);
            let total = order.len();
            self.last_total = total;
            // Chapter subtitle in top bar.
            let sub = format!("CH {}", self.last_chapter_index + 1);
            self.view
                .label(cx, ids!(top_bar.title_block.chapter_subtitle))
                .set_text(cx, &sub);

            // Scrubber labels.
            self.view
                .label(cx, ids!(scrubber.scrub_inner.scrub_labels.scrub_current))
                .set_text(cx, &position.to_string());
            self.view
                .label(cx, ids!(scrubber.scrub_inner.scrub_labels.scrub_total))
                .set_text(cx, &total.to_string());

            let _fraction = if total > 1 {
                (position as f32 - 1.0) / (total as f32 - 1.0)
            } else {
                0.0
            };
            self.view
                .rm_vertical_slider(
                    cx,
                    ids!(scrubber.scrub_inner.scrub_slider),
                )
                .set_range(cx, 0.0, (total - 1).max(0) as f32);
            self.view
                .rm_vertical_slider(
                    cx,
                    ids!(scrubber.scrub_inner.scrub_slider),
                )
                .set_value(cx, (position - 1) as f32);

            let has_prev = s.reader.has_prev(&order);
            let has_next = s.reader.has_next(&order);
            self.view
                .view(cx, ids!(bottom_bar.bar_inner.prev_btn))
                .set_visible(cx, has_prev);
            self.view
                .view(cx, ids!(bottom_bar.bar_inner.next_btn))
                .set_visible(cx, has_next);
            self.view
                .button(cx, ids!(reader_bg.content_tap.end_nav.prev_button2))
                .set_visible(cx, has_prev);
            self.view
                .button(cx, ids!(reader_bg.content_tap.end_nav.next_button2))
                .set_visible(cx, has_next);
        }
    }

    fn next_chapter(&mut self, cx: &mut Cx) {
        with_state_mut(|s| {
            s.reader_next_chapter();
        });
        cx.action(AppAction::StateChanged);
    }

    fn prev_chapter(&mut self, cx: &mut Cx) {
        with_state_mut(|s| {
            s.reader_prev_chapter();
        });
        cx.action(AppAction::StateChanged);
    }
}

impl Widget for ReaderScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if !matches!(state().nav.current(), Route::Reader { .. }) {
            self.loaded_chapter = None;
            self.panel_open = false;
        }
        self.sync(cx);
        let rect = cx.peek_walk_turtle(walk);
        self.draw_bg.draw_abs(cx, rect);
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for ReaderScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Close reader — back icon.
        if self
            .view
            .rm_tap(cx, ids!(top_bar.back_btn))
            .clicked(actions)
        {
            with_state_mut(|s| {
                s.go_back();
            });
            cx.action(AppAction::StateChanged);
            return;
        }

        // Chapter navigation via prev/next buttons.
        let next = self
            .view
            .rm_tap(cx, ids!(bottom_bar.bar_inner.next_btn))
            .clicked(actions)
            || self
                .view
                .button(cx, ids!(reader_bg.content_tap.end_nav.next_button2))
                .clicked(actions);
        if next {
            self.next_chapter(cx);
            return;
        }
        let prev = self
            .view
            .rm_tap(cx, ids!(bottom_bar.bar_inner.prev_btn))
            .clicked(actions)
            || self
                .view
                .button(cx, ids!(reader_bg.content_tap.end_nav.prev_button2))
                .clicked(actions);
        if prev {
            self.prev_chapter(cx);
            return;
        }

        // Tap text area to toggle controls.
        if self
            .view
            .rm_tap(cx, ids!(reader_bg.content_tap))
            .clicked(actions)
        {
            with_state_mut(|s| s.reader.toggle_controls());
            cx.action(AppAction::StateChanged);
        }

        // Scrubber — jump to chapter.
        if self
            .view
            .rm_vertical_slider(cx, ids!(scrubber.scrub_inner.scrub_slider))
            .changed(actions)
        {
            let val = self
                .view
                .rm_vertical_slider(cx, ids!(scrubber.scrub_inner.scrub_slider))
                .get_value();
            let target_idx = val.round() as usize;
            let s = state();
            if let Some((novel_id, _)) = s.reader.current {
                let order = s.reading_order(&novel_id);
                if let Some(chapter_id) = order.get(target_idx) {
                    with_state_mut(|s| {
                        s.open_chapter(novel_id, *chapter_id);
                    });
                    cx.action(AppAction::StateChanged);
                }
            }
        }

        // Settings panel toggle via textsize button.
        if self
            .view
            .rm_tap(cx, ids!(bottom_bar.bar_inner.textsize_btn))
            .clicked(actions)
        {
            self.panel_open = !self.panel_open;
            cx.action(AppAction::StateChanged);
        }

        // Gear button (dismiss settings or future action).
        if self
            .view
            .rm_tap(cx, ids!(bottom_bar.bar_inner.gear_btn))
            .clicked(actions)
        {
            if self.panel_open {
                self.panel_open = false;
                cx.action(AppAction::StateChanged);
            }
        }

        // Typography controls.
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
