use makepad_widgets::*;
use readmesh_app::{ContentRepository, SourceInfo};

use crate::app::AppAction;
use crate::components::RmTapWidgetRefExt;
use crate::state::{state, with_state_mut};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.BrowseScreen = #(BrowseScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Down

        header := SolidView{
            width: Fill height: Fit
            padding: theme.mspace_3{left: theme.space_3 * 2, right: theme.space_3 * 2}
            draw_bg.color: theme.color_bg_app
            flow: Right spacing: theme.space_2
            align: Align{y: 0.5}
            new_batch: true
            search_input := RmTextInput{
                empty_text: "Search Source"
            }
            pin_filter_button := View{width: 20 height: 20}
            sort_button := View{width: 20 height: 20}
            gear_button := View{width: 20 height: 20}
        }

        list := PortalList{
            width: Fill height: Fill
            flow: Down
            spacing: theme.space_1
            padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
            scroll_bar: ScrollBar{}

            SourceRow := CachedView{RmSourceRow{}}
        }
    }
}

fn source_initial(name: &str) -> String {
    name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_else(|| "M".to_string())
}

/// The Browse screen widget.
#[derive(Script, ScriptHook, Widget)]
pub struct BrowseScreen {
    #[deref]
    view: View,
    #[rust]
    sources: Vec<SourceInfo>,
    #[rust]
    query: String,
}

impl Widget for BrowseScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.query = self.view.text_input(cx, ids!(header.search_input)).text();
        let sources: Vec<SourceInfo> = state().catalog.sources();
        self.sources = sources.clone();

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, sources.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(source) = sources.get(item_id) else {
                        continue;
                    };
                    let q = self.query.to_lowercase();
                    if !q.is_empty() && !source.name.to_lowercase().contains(&q) {
                        continue;
                    }
                    let item = list.item(cx, item_id, id!(SourceRow));
                    item.label(cx, ids!(source_logo.source_initial))
                        .set_text(cx, &source_initial(&source.name));
                    item.label(cx, ids!(texts.source_name))
                        .set_text(cx, &source.name);
                    let lang_info = format!("v{}", source.version);
                    item.label(cx, ids!(texts.source_lang))
                        .set_text(cx, &lang_info);
                    item.draw_all_unscoped(cx);
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for BrowseScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let list = self.view.portal_list(cx, ids!(list));
        for (item_id, item) in list.items_with_actions(actions) {
            if item.rm_tap(cx, ids!(browse_tap)).clicked(actions) {
                if let Some(source) = self.sources.get(item_id).cloned() {
                    with_state_mut(|s| {
                        s.nav.select_tab(readmesh_app::PrimaryTab::Search);
                        s.search.query = source.name.clone();
                        s.refresh_search();
                    });
                    cx.action(AppAction::StateChanged);
                }
            }
        }
    }
}
