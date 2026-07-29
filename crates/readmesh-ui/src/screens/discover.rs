use makepad_widgets::*;

script_mod! {
    mod.widgets.DiscoverScreen = #(DiscoverScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Down spacing: theme.space_3
        padding: theme.mspace_2
        show_bg: true
        draw_bg: {color: theme.color_bg_app}

        header := Label{
            width: Fill height: Fit
            text: "Discover"
            draw_text.color: theme.color_label_inner
            draw_text.text_style: theme.font_bold{font_size: theme.font_size_2}
        }
        placeholder := Label{
            width: Fill height: Fill
            align: Center
            text: "Trending & recommended novels\n(curated feeds coming soon)"
            draw_text.color: theme.color_label_inner_inactive
            draw_text.text_style.font_size: theme.font_size_4
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct DiscoverScreen {
    #[deref]
    view: View,
}

impl Widget for DiscoverScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for DiscoverScreen {}
