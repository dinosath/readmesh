use makepad_widgets::*;

use crate::app::AppAction;
use crate::state::with_state_mut;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.OnboardingScreen = #(OnboardingScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Down spacing: 20
        padding: 40
        align: Center
        draw_bg.color: theme.color_bg_app

        title_label := Label{
            width: Fit height: Fit
            text: "ReadMesh"
            draw_text.color: theme.color_highlight
            draw_text.text_style: theme.font_bold{font_size: 32.0}
        }
        done_button := RmPrimaryButton{
            width: Fill height: 48
            text: "Get Started"
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct OnboardingScreen {
    #[deref]
    view: View,
}

impl Widget for OnboardingScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for OnboardingScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        if self.view.button(cx, ids!(done_button)).clicked(actions) {
            with_state_mut(|s| s.nav.select_tab(readmesh_app::PrimaryTab::Library));
            cx.action(AppAction::StateChanged);
        }
    }
}
