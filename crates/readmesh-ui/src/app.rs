//! ReadMesh application entry point.
//!
//! Registration order (per the Makepad 2.0 app-structure skill):
//! theme base -> ReadMesh themes -> base widgets -> components -> screens ->
//! shell -> app UI.
//!
//! Event flow (Elm-style): widget interactions mutate the global
//! [`readmesh_app::AppState`] and post [`AppAction::StateChanged`]; the App redraws
//! and every widget re-projects from state.

use makepad_widgets::*;

use crate::state::with_state_mut;

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1280, 800)
                window.title: "ReadMesh"
                pass.clear_color: theme.color_bg_app
                body +: {
                    app_shell := AppShell{}
                }
            }
        }
    }
}

/// Application-level actions (NOT widget actions), posted via `cx.action`.
#[derive(Clone, Debug)]
pub enum AppAction {
    /// Application state changed somewhere; redraw the UI.
    StateChanged,
    /// A search was submitted; complete it after a short tick so the
    /// loading state is observable (mirrors a future async backend).
    SearchRequested,
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    download_timer: Timer,
    #[rust]
    search_timer: Timer,
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        // Drives the download simulation (and later, download housekeeping).
        self.download_timer = cx.start_interval(0.4);
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for action in actions {
            match action.downcast_ref::<AppAction>() {
                Some(AppAction::StateChanged) => self.ui.redraw(cx),
                Some(AppAction::SearchRequested) => {
                    self.search_timer = cx.start_timeout(0.25);
                }
                None => {}
            }
        }
    }

    fn handle_timer(&mut self, cx: &mut Cx, event: &TimerEvent) {
        if self.search_timer.is_timer(event).is_some() {
            with_state_mut(|s| s.finish_search());
            self.ui.redraw(cx);
        }
        if self.download_timer.is_timer(event).is_some() {
            let progressed = with_state_mut(|s| {
                if s.downloads.active_count() > 0 {
                    s.tick_downloads();
                    true
                } else {
                    false
                }
            });
            if progressed {
                self.ui.redraw(cx);
            }
        }
    }

    fn handle_key_down(&mut self, cx: &mut Cx, event: &KeyEvent) {
        // Escape navigates back unless a text input is focused.
        if event.key_code == KeyCode::Escape {
            let search_area = self
                .ui
                .text_input(
                    cx,
                    ids!(
                        app_shell
                            .main_row
                            .content_host
                            .search_screen
                            .header
                            .search_input
                    ),
                )
                .area();
            let search_focused = cx.has_key_focus(search_area);
            if !search_focused {
                let went_back = with_state_mut(|s| s.go_back());
                if went_back {
                    self.ui.redraw(cx);
                }
            }
        }
    }

    fn handle_back_pressed(&mut self, cx: &mut Cx) -> bool {
        // Android / mobile back button.
        let went_back = with_state_mut(|s| s.go_back());
        if went_back {
            self.ui.redraw(cx);
        }
        went_back
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        // 1. Base themes, 2. ReadMesh theme selection, 3. base widgets.
        crate::makepad_widgets::theme_mod(vm);
        crate::theme::script_mod(vm);
        crate::makepad_widgets::widgets_mod(vm);
        // 4. Shared components, 5. screens, 6. shell.
        crate::components::script_mod(vm);
        crate::screens::script_mod(vm);
        crate::shell::script_mod(vm);
        // 7. App UI.
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
