//! Adaptive application shell.
//!
//! The shell owns the navigation chrome (side rail on tablet/desktop,
//! bottom bar on mobile) and the routed content area. It is a pure
//! projection of [`readmesh_app::NavigationState`]: every `draw_walk` re-syncs
//! chrome visibility, screen visibility and nav selection from state.
//!
//! Contains two custom widgets:
//! - [`RmNavButton`] — a selectable navigation button (View + Animator,
//!   modeled on Robrix's `NavigationBarButton`)
//! - [`AppShell`] — the shell itself

use makepad_widgets::*;
use readmesh_app::{NavMode, PrimaryTab, Route};

use crate::app::AppAction;
use crate::state::{state, with_state_mut};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // A navigation button with hover + selected states, driven by an
    // Animator (pattern from Robrix's NavigationBarButton).
    mod.widgets.RmNavButton = #(RmNavButton::register_widget(vm)){
        width: Fill height: 40
        flow: Right spacing: theme.space_2
        align: Align{x: 0.0 y: 0.5}
        padding: theme.mspace_h_2{left: theme.space_3, right: theme.space_3}
        cursor: MouseCursor.Hand
        new_batch: true
        show_bg: true

        draw_bg +: {
            hover: instance(0.0)
            active: instance(0.0)
            color_hover: instance(theme.color_bg_highlight)
            color_active: instance(theme.color_bg_highlight * 1.6)
            border_radius: uniform(8.0)

            get_color: fn() -> vec4 {
                let hover_color = vec4(self.color_hover.xyz, self.color_hover.w * self.hover)
                return mix(hover_color, self.color_active, self.active)
            }
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, max(1.0, self.border_radius))
                sdf.fill_keep(self.get_color())
                return sdf.result
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward{duration: 0.12}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {hover: 1.0}}
                }
                down: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
            active: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {active: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {active: 1.0}}
                }
            }
        }

        nav_icon := View{width: 20 height: 20}
        nav_label := Label{
            width: Fill
            text: "Item"
            draw_text.color: theme.color_label_inner
            draw_text.text_style.font_size: theme.font_size_4
        }
    }

    // Tab variant for the mobile bottom bar (icon above label, centered).
    mod.widgets.RmNavTabButton = mod.widgets.RmNavButton{
        height: 56
        flow: Down
        align: Center
        spacing: 2
        padding: theme.mspace_h_1
        draw_bg +: {
            border_radius: uniform(8.0)
        }
        nav_icon +: {
            width: 22 height: 22
        }
        nav_label +: {
            align: Center
            draw_text +: {
                text_style +: {font_size: theme.font_size_code}
            }
        }
    }

    mod.widgets.AppShell = #(AppShell::register_widget(vm)){
        width: Fill height: Fill
        flow: Down

        main_row := View{
            width: Fill height: Fill
            flow: Right

            nav_rail := SolidView{
                width: 200 height: Fill
                flow: Down spacing: theme.space_1
                padding: theme.mspace_2
                draw_bg.color: theme.color_fg_app
                new_batch: true

                rail_brand := View{
                    width: Fill height: Fit
                    flow: Down spacing: 2
                    padding: theme.mspace_2{left: theme.space_2, bottom: theme.space_3}
                    Label{
                        text: "ReadMesh"
                        draw_text.color: theme.color_highlight
                        draw_text.text_style: theme.font_bold{font_size: theme.font_size_3}
                    }
                    Label{
                        text: "mesh-native reader"
                        draw_text.color: theme.color_label_inner_inactive
                        draw_text.text_style.font_size: theme.font_size_code
                    }
                }

                rail_library := RmNavButton{
                    nav_icon := IconBook{}
                    nav_label.text: "Library"
                }
                rail_browse := RmNavButton{
                    nav_icon := IconCompass{}
                    nav_label.text: "Browse"
                }
                rail_search := RmNavButton{
                    nav_icon := IconSearch{}
                    nav_label.text: "Search"
                }
                rail_downloads := RmNavButton{
                    nav_icon := IconDownload{}
                    nav_label.text: "Downloads"
                }
                rail_settings := RmNavButton{
                    nav_icon := IconGear{}
                    nav_label.text: "Settings"
                }

                Filler{}

                rail_hint := Label{
                    width: Fill
                    text: "Local demo data"
                    padding: theme.mspace_h_2{left: theme.space_2}
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_code
                }
            }

            content_host := View{
                width: Fill height: Fill
                flow: Overlay

                library_screen := LibraryScreen{}
                browse_screen := BrowseScreen{visible: false}
                search_screen := SearchScreen{visible: false}
                detail_screen := DetailScreen{visible: false}
                downloads_screen := DownloadsScreen{visible: false}
                settings_screen := SettingsScreen{visible: false}
                reader_screen := ReaderScreen{visible: false}
                onboarding_screen := OnboardingScreen{visible: false}
                create_novel_screen := CreateNovelScreen{visible: false}
                metadata_editor_screen := MetadataEditorScreen{visible: false}
                cover_studio_screen := CoverStudioScreen{visible: false}
                import_screen := ImportScreen{visible: false}
                chapter_editor_screen := ChapterEditorScreen{visible: false}
                collaborative_screen := CollaborativeScreen{visible: false}
                peer_connections_screen := PeerConnectionsScreen{visible: false}
                sync_dashboard_screen := SyncDashboardScreen{visible: false}
            }
        }

        bottom_nav := SolidView{
            width: Fill height: Fit
            flow: Right
            padding: theme.mspace_1
            draw_bg.color: theme.color_fg_app
            new_batch: true
            visible: false

            tab_library := RmNavTabButton{
                nav_icon := IconBook{}
                nav_label.text: "Library"
            }
            tab_browse := RmNavTabButton{
                nav_icon := IconCompass{}
                nav_label.text: "Browse"
            }
            tab_search := RmNavTabButton{
                nav_icon := IconSearch{}
                nav_label.text: "Search"
            }
            tab_downloads := RmNavTabButton{
                nav_icon := IconDownload{}
                nav_label.text: "Downloads"
            }
            tab_settings := RmNavTabButton{
                nav_icon := IconGear{}
                nav_label.text: "Settings"
            }
        }
    }
}

/// Actions emitted by [`RmNavButton`].
#[derive(Clone, Debug, Default)]
pub enum RmNavButtonAction {
    #[default]
    None,
    Clicked,
}

/// A navigation button: a `View` with hover/selected animator states that
/// emits [`RmNavButtonAction::Clicked`]. Selection is managed by the parent
/// (radio-group semantics), per the selection model documented in Robrix.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct RmNavButton {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
}

impl Widget for RmNavButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        // Run our own hit test BEFORE forwarding to children, so children
        // cannot consume the click on our area (see Robrix's rationale).
        let area = self.view.area();
        let widget_uid = self.widget_uid();
        match event.hits(cx, area) {
            Hit::FingerHoverIn(_) => self.animator_play(cx, ids!(hover.on)),
            Hit::FingerHoverOut(_) => self.animator_play(cx, ids!(hover.off)),
            Hit::FingerDown(_) => self.animator_play(cx, ids!(hover.down)),
            Hit::FingerMove(fe) if !fe.is_over => self.animator_play(cx, ids!(hover.off)),
            Hit::FingerUp(fe) => {
                if fe.is_over && fe.is_primary_hit() && fe.was_tap() {
                    cx.widget_action(widget_uid, RmNavButtonAction::Clicked);
                }
                if fe.device.has_hovers() && fe.is_over {
                    self.animator_play(cx, ids!(hover.on));
                } else {
                    self.animator_play(cx, ids!(hover.off));
                }
            }
            _ => {}
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl RmNavButton {
    pub fn set_selected(&mut self, cx: &mut Cx, selected: bool) {
        self.animator_toggle(cx, selected, Animate::No, ids!(active.on), ids!(active.off));
    }
}

impl RmNavButtonRef {
    pub fn set_selected(&self, cx: &mut Cx, selected: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_selected(cx, selected);
        }
    }

    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            return matches!(item.cast(), RmNavButtonAction::Clicked);
        }
        false
    }
}

/// The adaptive app shell. Projects `NavigationState` onto the widget tree.
#[derive(Script, ScriptHook, Widget)]
pub struct AppShell {
    #[deref]
    view: View,
    #[rust]
    applied_route: Option<Route>,
    #[rust]
    applied_mode: Option<NavMode>,
    #[rust]
    applied_badge: usize,
}

/// The screens hosted in the content area, with the route that activates
/// each one.
const SCREENS: &[(PrimaryTab, &[LiveId])] = &[
    (
        PrimaryTab::Library,
        ids!(main_row.content_host.library_screen),
    ),
    (
        PrimaryTab::Browse,
        ids!(main_row.content_host.browse_screen),
    ),
    (
        PrimaryTab::Search,
        ids!(main_row.content_host.search_screen),
    ),
    (
        PrimaryTab::Downloads,
        ids!(main_row.content_host.downloads_screen),
    ),
    (
        PrimaryTab::Settings,
        ids!(main_row.content_host.settings_screen),
    ),
];

const NAV_ITEMS: &[(PrimaryTab, &[LiveId], &[LiveId])] = &[
    (
        PrimaryTab::Library,
        ids!(main_row.nav_rail.rail_library),
        ids!(bottom_nav.tab_library),
    ),
    (
        PrimaryTab::Browse,
        ids!(main_row.nav_rail.rail_browse),
        ids!(bottom_nav.tab_browse),
    ),
    (
        PrimaryTab::Search,
        ids!(main_row.nav_rail.rail_search),
        ids!(bottom_nav.tab_search),
    ),
    (
        PrimaryTab::Downloads,
        ids!(main_row.nav_rail.rail_downloads),
        ids!(bottom_nav.tab_downloads),
    ),
    (
        PrimaryTab::Settings,
        ids!(main_row.nav_rail.rail_settings),
        ids!(bottom_nav.tab_settings),
    ),
];

impl AppShell {
    fn sync_chrome(&mut self, cx: &mut Cx2d) {
        let (route, tab, mode, active_downloads) = {
            let s = state();
            (
                s.nav.current(),
                s.nav.current_tab(),
                s.nav.mode(),
                s.downloads.active_count(),
            )
        };

        let route_changed = self.applied_route != Some(route);
        let mode_changed = self.applied_mode != Some(mode);

        if mode_changed {
            self.applied_mode = Some(mode);
        }
        if route_changed {
            self.applied_route = Some(route);
        }

        // Chrome visibility: rail on tablet/desktop, bottom bar on mobile,
        // neither while the reader is open.
        if mode_changed || route_changed {
            let chrome_hidden = route.hides_chrome();
            self.view
                .view(cx, ids!(main_row.nav_rail))
                .set_visible(cx, !mode.uses_bottom_nav() && !chrome_hidden);
            self.view
                .view(cx, ids!(bottom_nav))
                .set_visible(cx, mode.uses_bottom_nav() && !chrome_hidden);
        }

        // Screen visibility.
        if route_changed {
            // Hide all screens first
            for (_, path) in SCREENS {
                self.view.view(cx, path).set_visible(cx, false);
            }
            // Show the matching tab screen
            if let Route::Tab(pt) = route {
                for (screen_tab, path) in SCREENS {
                    self.view
                        .view(cx, path)
                        .set_visible(cx, *screen_tab == pt);
                }
            }
            self.view
                .view(cx, ids!(main_row.content_host.detail_screen))
                .set_visible(cx, matches!(route, Route::NovelDetail(_)));
            self.view
                .view(cx, ids!(main_row.content_host.reader_screen))
                .set_visible(cx, matches!(route, Route::Reader { .. }));
            self.view
                .view(cx, ids!(main_row.content_host.onboarding_screen))
                .set_visible(cx, matches!(route, Route::Onboarding));
            self.view
                .view(cx, ids!(main_row.content_host.create_novel_screen))
                .set_visible(cx, matches!(route, Route::CreateNovel));
            self.view
                .view(cx, ids!(main_row.content_host.metadata_editor_screen))
                .set_visible(cx, matches!(route, Route::MetadataEditor(_)));
            self.view
                .view(cx, ids!(main_row.content_host.cover_studio_screen))
                .set_visible(cx, matches!(route, Route::CoverStudio(_)));
            self.view
                .view(cx, ids!(main_row.content_host.import_screen))
                .set_visible(cx, matches!(route, Route::ImportFromWebsite));
            self.view
                .view(cx, ids!(main_row.content_host.chapter_editor_screen))
                .set_visible(cx, matches!(route, Route::ChapterEditor { .. }));
            self.view
                .view(cx, ids!(main_row.content_host.collaborative_screen))
                .set_visible(cx, matches!(route, Route::CollaborativeWorkspace(_)));
            self.view
                .view(cx, ids!(main_row.content_host.peer_connections_screen))
                .set_visible(cx, matches!(route, Route::PeerConnections));
            self.view
                .view(cx, ids!(main_row.content_host.sync_dashboard_screen))
                .set_visible(cx, matches!(route, Route::SyncDashboard));

            // Nav selection follows the active tab.
            if let Route::Tab(_) = route {
                for (item_tab, rail_path, tab_path) in NAV_ITEMS {
                    let selected = *item_tab == tab;
                    self.view
                        .rm_nav_button(cx, rail_path)
                        .set_selected(cx, selected);
                    self.view
                        .rm_nav_button(cx, tab_path)
                        .set_selected(cx, selected);
                }
            }
        }

        // Downloads badge on the rail item.
        if self.applied_badge != active_downloads {
            self.applied_badge = active_downloads;
            let text = if active_downloads > 0 {
                format!("Downloads · {active_downloads}")
            } else {
                "Downloads".to_string()
            };
            self.view
                .label(cx, ids!(main_row.nav_rail.rail_downloads.nav_label))
                .set_text(cx, &text);
        }
    }
}

impl Widget for AppShell {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Adaptive layout: derive the nav mode from the available width.
        let width = cx.peek_walk_turtle(walk).size.x;
        let mode = NavMode::from_width(width);
        with_state_mut(|s| {
            s.nav.set_mode(mode);
        });
        self.sync_chrome(cx);
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for AppShell {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        for (tab, rail_path, tab_path) in NAV_ITEMS {
            let clicked = self.view.rm_nav_button(cx, rail_path).clicked(actions)
                || self.view.rm_nav_button(cx, tab_path).clicked(actions);
            if clicked {
                with_state_mut(|s| s.select_tab(*tab));
                cx.action(AppAction::StateChanged);
            }
        }
    }
}
