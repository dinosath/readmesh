use makepad_widgets::*;

script_mod! {
    mod.widgets.SyncDashboardScreen = #(SyncDashboardScreen::register_widget(vm)){
        width: Fill height: Fill
        show_bg: true draw_bg: {color: theme.color_bg_app}
        flow: Down
        body := Label{width: Fill height: Fill align: Center text: "Sync Dashboard" draw_text.color: theme.color_label_inner_inactive}
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct SyncDashboardScreen { #[deref] view: View }
impl Widget for SyncDashboardScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) { self.view.handle_event(cx, event, scope); self.widget_match_event(cx, event, scope); }
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep { self.view.draw_walk(cx, scope, walk) }
}
impl WidgetMatchEvent for SyncDashboardScreen {}
