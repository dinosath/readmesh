use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.CollaborativeScreen = #(CollaborativeScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Down
        draw_bg.color: theme.color_bg_app
        body := Label{width: Fill height: Fill align: Center text: "Collaborative Workspace" draw_text.color: theme.color_label_inner_inactive}
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct CollaborativeScreen { #[deref] view: View }
impl Widget for CollaborativeScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) { self.view.handle_event(cx, event, scope); self.widget_match_event(cx, event, scope); }
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep { self.view.draw_walk(cx, scope, walk) }
}
impl WidgetMatchEvent for CollaborativeScreen {}
