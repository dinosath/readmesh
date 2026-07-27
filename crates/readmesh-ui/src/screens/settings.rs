//! Settings screen: appearance, reader, downloads, sources, storage,
//! network and about. Structured in sections so new options slot in
//! without the screen becoming monolithic.

use makepad_widgets::*;
use readmesh_app::{ContentRepository, ReaderTheme, ThemeMode};

use crate::app::AppAction;
use crate::state::{state, with_state_mut};
use crate::theme::apply_theme;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.SettingsScreen = #(SettingsScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Down

        header := SolidView{
            width: Fill height: Fit
            padding: theme.mspace_3{left: theme.space_3 * 2, right: theme.space_3 * 2}
            draw_bg.color: theme.color_bg_app
            flow: Right
            align: Align{y: 0.5}
            new_batch: true
            Label{
                text: "Settings"
                draw_text.color: theme.color_label_inner
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_2}
            }
        }

        ScrollYView{
            width: Fill height: Fill
            flow: Down spacing: theme.space_1
            padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
            new_batch: true

            // ---- Appearance ----
            RmSettingsHeader{settings_header.text: "Appearance"}
            dark_mode_row := RmSettingsToggleRow{}
            reader_theme_row := RmSettingsActionRow{}

            // ---- Reader ----
            RmSettingsHeader{settings_header.text: "Reader"}
            font_size_row := RmSettingsActionRow{}
            line_spacing_row := RmSettingsActionRow{}
            immersive_row := RmSettingsToggleRow{}

            // ---- Downloads ----
            RmSettingsHeader{settings_header.text: "Downloads"}
            max_concurrent_row := RmSettingsActionRow{}
            wifi_only_row := RmSettingsToggleRow{}
            auto_download_row := RmSettingsActionRow{}
            delete_after_read_row := RmSettingsToggleRow{}

            // ---- Sources ----
            RmSettingsHeader{settings_header.text: "Sources"}
            sources_host := View{
                width: Fill height: Fit
                flow: Down spacing: theme.space_1
                source0 := RmSettingsToggleRow{}
                source1 := RmSettingsToggleRow{}
                source2 := RmSettingsToggleRow{}
            }

            // ---- Storage ----
            RmSettingsHeader{settings_header.text: "Storage"}
            cache_limit_row := RmSettingsActionRow{}
            cache_used_row := RmSettingsActionRow{}

            // ---- Network ----
            RmSettingsHeader{settings_header.text: "Network"}
            data_saver_row := RmSettingsToggleRow{}
            p2p_row := RmSettingsToggleRow{}

            // ---- About ----
            RmSettingsHeader{settings_header.text: "About"}
            version_row := RmSettingsActionRow{}
            license_row := RmSettingsActionRow{}
            about_card := RoundedView{
                width: Fill height: Fit
                flow: Down spacing: theme.space_1
                padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
                draw_bg.color: theme.color_bg_container
                draw_bg.border_radius: 8.0
                new_batch: true
                Label{
                    width: Fill
                    text: "ReadMesh — a mesh-native reader"
                    draw_text.color: theme.color_label_inner
                    draw_text.text_style: theme.font_bold{font_size: theme.font_size_p}
                }
                Label{
                    width: Fill
                    text: "Discover, organize, download and read content from distributed sources. Currently running on local demo data; P2P backends plug in behind the same repository traits."
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style +: {font_size: theme.font_size_p, line_spacing: 1.4}
                }
            }
            View{width: Fill height: 40}
        }
    }
}

const SOURCE_ROW_IDS: [LiveId; 3] = [id!(source0), id!(source1), id!(source2)];

/// The Settings screen widget.
#[derive(Script, ScriptHook, Widget)]
pub struct SettingsScreen {
    #[deref]
    view: View,
}

impl SettingsScreen {
    fn set_row_texts(&self, cx: &mut Cx2d, path: &[LiveId], title: &str, subtitle: &str) {
        let row = self.view.view(cx, path);
        row.label(cx, ids!(texts.set_title)).set_text(cx, title);
        row.label(cx, ids!(texts.set_subtitle))
            .set_text(cx, subtitle);
    }

    fn set_row_value(&self, cx: &mut Cx2d, path: &[LiveId], value: &str, button: &str) {
        let row = self.view.view(cx, path);
        row.label(cx, ids!(set_value)).set_text(cx, value);
        row.button(cx, ids!(set_button)).set_text(cx, button);
    }

    fn set_row_toggle(&self, cx: &mut Cx2d, path: &[LiveId], active: bool) {
        self.view
            .view(cx, path)
            .check_box(cx, ids!(set_toggle))
            .set_active(cx, active, Animate::No);
    }

    /// Project `AppSettings` onto the static rows (runs every draw).
    fn sync(&self, cx: &mut Cx2d) {
        let s = state();

        // Appearance.
        self.set_row_texts(
            cx,
            ids!(dark_mode_row),
            "Dark mode",
            "Dark, reader-friendly theme",
        );
        self.set_row_toggle(cx, ids!(dark_mode_row), s.settings.theme == ThemeMode::Dark);
        self.set_row_texts(
            cx,
            ids!(reader_theme_row),
            "Reader theme",
            "Colors used by the reader",
        );
        self.set_row_value(
            cx,
            ids!(reader_theme_row),
            s.settings.reader.theme.name(),
            "Change",
        );

        // Reader.
        self.set_row_texts(
            cx,
            ids!(font_size_row),
            "Font size",
            "Reader body text size",
        );
        let font = format!("{:.0}", s.settings.reader.font_size);
        self.set_row_value(cx, ids!(font_size_row), &font, "Change");
        self.set_row_texts(
            cx,
            ids!(line_spacing_row),
            "Line spacing",
            "Reader line height multiplier",
        );
        let spacing = format!("{:.1}", s.settings.reader.line_spacing);
        self.set_row_value(cx, ids!(line_spacing_row), &spacing, "Change");
        self.set_row_texts(
            cx,
            ids!(immersive_row),
            "Immersive mode",
            "Open the reader with controls hidden",
        );
        self.set_row_toggle(cx, ids!(immersive_row), s.settings.reader.immersive);

        // Downloads.
        self.set_row_texts(
            cx,
            ids!(max_concurrent_row),
            "Simultaneous downloads",
            "Maximum active downloads",
        );
        let concurrent = format!("{}", s.settings.downloads.max_concurrent);
        self.set_row_value(cx, ids!(max_concurrent_row), &concurrent, "Change");
        self.set_row_texts(
            cx,
            ids!(wifi_only_row),
            "Wi-Fi only",
            "Only download on unmetered connections",
        );
        self.set_row_toggle(cx, ids!(wifi_only_row), s.settings.downloads.wifi_only);
        self.set_row_texts(
            cx,
            ids!(auto_download_row),
            "Auto-download next",
            "Unread chapters to keep downloaded",
        );
        let auto = format!("{}", s.settings.downloads.auto_download_next);
        self.set_row_value(cx, ids!(auto_download_row), &auto, "Change");
        self.set_row_texts(
            cx,
            ids!(delete_after_read_row),
            "Delete after reading",
            "Remove downloads of finished chapters",
        );
        self.set_row_toggle(
            cx,
            ids!(delete_after_read_row),
            s.settings.downloads.delete_after_read,
        );

        // Sources.
        let sources = s.catalog.sources();
        for (i, row_id) in SOURCE_ROW_IDS.iter().enumerate() {
            let row = self.view.view(cx, &[*row_id]);
            match sources.get(i) {
                Some(source) => {
                    row.set_visible(cx, true);
                    row.label(cx, ids!(texts.set_title))
                        .set_text(cx, &source.name);
                    row.label(cx, ids!(texts.set_subtitle)).set_text(
                        cx,
                        &format!("v{} · {} novels", source.version, source.novel_count),
                    );
                    row.check_box(cx, ids!(set_toggle)).set_active(
                        cx,
                        s.settings.is_source_enabled(&source.id.0),
                        Animate::No,
                    );
                }
                None => row.set_visible(cx, false),
            }
        }

        // Storage.
        self.set_row_texts(
            cx,
            ids!(cache_limit_row),
            "Cache limit",
            "Maximum size of cached content",
        );
        let limit = format!("{} MiB", s.settings.storage.cache_limit_mb);
        self.set_row_value(cx, ids!(cache_limit_row), &limit, "Change");
        self.set_row_texts(
            cx,
            ids!(cache_used_row),
            "Cache used",
            "Space used by downloaded chapters",
        );
        let used = format!(
            "{:.1} MiB",
            s.settings.storage.cache_used_bytes as f64 / 1_048_576.0
        );
        self.set_row_value(cx, ids!(cache_used_row), &used, "Clear");

        // Network.
        self.set_row_texts(
            cx,
            ids!(data_saver_row),
            "Data saver",
            "Reduce bandwidth usage",
        );
        self.set_row_toggle(cx, ids!(data_saver_row), s.settings.network.data_saver);
        self.set_row_texts(
            cx,
            ids!(p2p_row),
            "P2P mirroring",
            "Mirror chapters for followed peers",
        );
        self.set_row_toggle(cx, ids!(p2p_row), s.settings.network.p2p_mirroring);

        // About.
        self.set_row_texts(cx, ids!(version_row), "Version", "ReadMesh");
        self.set_row_value(cx, ids!(version_row), s.settings.version(), "");
        self.view
            .view(cx, ids!(version_row))
            .button(cx, ids!(set_button))
            .set_visible(cx, false);
        self.set_row_texts(cx, ids!(license_row), "License", "Open source");
        self.set_row_value(cx, ids!(license_row), "MIT OR Apache-2.0", "");
        self.view
            .view(cx, ids!(license_row))
            .button(cx, ids!(set_button))
            .set_visible(cx, false);
    }

    fn row_clicked(&self, cx: &mut Cx, actions: &Actions, path: &[LiveId]) -> bool {
        self.view
            .view(cx, path)
            .button(cx, ids!(set_button))
            .clicked(actions)
    }

    fn row_toggled(&self, cx: &mut Cx, actions: &Actions, path: &[LiveId]) -> Option<bool> {
        self.view
            .view(cx, path)
            .check_box(cx, ids!(set_toggle))
            .changed(actions)
    }
}

impl Widget for SettingsScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.sync(cx);
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for SettingsScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // ---- Appearance ----
        if let Some(dark) = self.row_toggled(cx, actions, ids!(dark_mode_row)) {
            let mode = with_state_mut(|s| {
                s.settings.theme = if dark {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
                s.settings.theme
            });
            apply_theme(cx, mode);
            cx.action(AppAction::StateChanged);
        }
        if self.row_clicked(cx, actions, ids!(reader_theme_row)) {
            with_state_mut(|s| {
                let next = match s.settings.reader.theme {
                    ReaderTheme::Dark => ReaderTheme::Light,
                    ReaderTheme::Light => ReaderTheme::Sepia,
                    ReaderTheme::Sepia => ReaderTheme::Dark,
                };
                s.settings.reader.set_theme(next);
                s.reader.settings = s.settings.reader.clone();
            });
            cx.action(AppAction::StateChanged);
        }

        // ---- Reader ----
        if self.row_clicked(cx, actions, ids!(font_size_row)) {
            with_state_mut(|s| {
                // Cycle through a useful range of sizes.
                if s.settings.reader.font_size >= 24.0 {
                    s.settings.reader.font_size = 14.0;
                } else {
                    s.settings.reader.font_size += 2.0;
                }
                s.reader.settings = s.settings.reader.clone();
            });
            cx.action(AppAction::StateChanged);
        }
        if self.row_clicked(cx, actions, ids!(line_spacing_row)) {
            with_state_mut(|s| {
                if s.settings.reader.line_spacing >= 2.0 {
                    s.settings.reader.line_spacing = 1.3;
                } else {
                    s.settings.reader.line_spacing += 0.1;
                }
                s.settings.reader = s.settings.reader.clone();
            });
            cx.action(AppAction::StateChanged);
        }
        if let Some(immersive) = self.row_toggled(cx, actions, ids!(immersive_row)) {
            with_state_mut(|s| {
                s.settings.reader.immersive = immersive;
                s.reader.settings = s.settings.reader.clone();
            });
            cx.action(AppAction::StateChanged);
        }

        // ---- Downloads ----
        if self.row_clicked(cx, actions, ids!(max_concurrent_row)) {
            with_state_mut(|s| {
                s.settings.downloads.max_concurrent = if s.settings.downloads.max_concurrent >= 5 {
                    1
                } else {
                    s.settings.downloads.max_concurrent + 1
                };
                s.downloads
                    .set_max_concurrent(s.settings.downloads.max_concurrent);
            });
            cx.action(AppAction::StateChanged);
        }
        if let Some(wifi_only) = self.row_toggled(cx, actions, ids!(wifi_only_row)) {
            with_state_mut(|s| s.settings.downloads.wifi_only = wifi_only);
            cx.action(AppAction::StateChanged);
        }
        if self.row_clicked(cx, actions, ids!(auto_download_row)) {
            with_state_mut(|s| {
                s.settings.downloads.auto_download_next =
                    match s.settings.downloads.auto_download_next {
                        0 => 1,
                        1 => 2,
                        2 => 3,
                        3 => 5,
                        _ => 0,
                    };
            });
            cx.action(AppAction::StateChanged);
        }
        if let Some(delete) = self.row_toggled(cx, actions, ids!(delete_after_read_row)) {
            with_state_mut(|s| s.settings.downloads.delete_after_read = delete);
            cx.action(AppAction::StateChanged);
        }

        // ---- Sources ----
        for (i, row_id) in SOURCE_ROW_IDS.iter().enumerate() {
            if self.row_toggled(cx, actions, &[*row_id]).is_some() {
                let key =
                    with_state_mut(|s| s.catalog.sources().get(i).map(|src| src.id.0.clone()));
                if let Some(key) = key {
                    with_state_mut(|s| {
                        s.settings.toggle_source(&key);
                    });
                    cx.action(AppAction::StateChanged);
                }
            }
        }

        // ---- Storage ----
        if self.row_clicked(cx, actions, ids!(cache_limit_row)) {
            with_state_mut(|s| {
                s.settings.storage.cache_limit_mb = match s.settings.storage.cache_limit_mb {
                    128 => 256,
                    256 => 512,
                    512 => 1024,
                    _ => 128,
                };
            });
            cx.action(AppAction::StateChanged);
        }
        if self.row_clicked(cx, actions, ids!(cache_used_row)) {
            with_state_mut(|s| {
                s.settings.clear_cache();
            });
            cx.action(AppAction::StateChanged);
        }

        // ---- Network ----
        if let Some(data_saver) = self.row_toggled(cx, actions, ids!(data_saver_row)) {
            with_state_mut(|s| s.settings.network.data_saver = data_saver);
            cx.action(AppAction::StateChanged);
        }
        if let Some(p2p) = self.row_toggled(cx, actions, ids!(p2p_row)) {
            with_state_mut(|s| s.settings.network.p2p_mirroring = p2p);
            cx.action(AppAction::StateChanged);
        }
    }
}
