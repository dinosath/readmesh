use makepad_widgets::*;
use readmesh_app::{AppState, ContentRepository};
use readmesh_core::NovelId;

use crate::app::AppAction;
use crate::components::RmChipWidgetRefExt;
use crate::screens::{
    GridRow, card_novel_at, draw_card_row, grid_cols, push_card_rows,
};
use crate::state::{state, with_state_mut};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.LibraryScreen = #(LibraryScreen::register_widget(vm)){
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
                empty_text: "Search Library"
            }
            filter_button := RmSmallButton{
                text: ""
            }
        }

        list := PortalList{
            width: Fill height: Fill
            flow: Down
            spacing: theme.space_1
            padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
            scroll_bar: ScrollBar{}

            Section := CachedView{RmSectionHeader{}}
            Row2 := CachedView{RmCardRow2{}}
            Row3 := CachedView{RmCardRow3{}}
            Row4 := CachedView{RmCardRow4{}}
            Chips := CachedView{RmChipRow{}}
            Empty := CachedView{
                RmEmptyState{
                    empty_title.text: "Your library is empty"
                    empty_body.text: "Browse the mesh to discover something to read"
                }
            }
        }
    }
}

fn build_rows(s: &AppState, cols: usize, query: &str) -> Vec<GridRow> {
    let mut rows = Vec::new();

    let all = s.library.all_sorted();
    let filtered: Vec<NovelId> = if query.is_empty() {
        all
    } else {
        let q = query.to_lowercase();
        all.into_iter().filter(|id| {
            s.catalog.novel(id).is_some_and(|n| {
                n.title.to_lowercase().contains(&q)
                    || n.authors.iter().any(|a| a.name.to_lowercase().contains(&q))
                    || n.tags.iter().any(|t| t.name.to_lowercase().contains(&q))
            })
        }).collect()
    };

    if filtered.is_empty() && s.library.novel_count() == 0 {
        rows.push(GridRow::Empty);
    } else if filtered.is_empty() {
        rows.push(GridRow::Section {
            title: "No results".to_string(),
            note: String::new(),
        });
    } else {
        rows.push(GridRow::Section {
            title: String::new(),
            note: format!("{}", filtered.len()),
        });
        push_card_rows(&mut rows, &filtered, cols);
    }

    rows
}

/// The Library screen widget.
#[derive(Script, ScriptHook, Widget)]
pub struct LibraryScreen {
    #[deref]
    view: View,
    #[rust]
    rows: Vec<GridRow>,
    #[rust]
    query: String,
}

impl LibraryScreen {
    pub fn draw_chip_row(
        cx: &mut Cx2d,
        item: &WidgetRef,
        chips: &[String],
        selected: Option<&str>,
    ) {
        const CHIP_IDS: [LiveId; 8] = [
            id!(chip0),
            id!(chip1),
            id!(chip2),
            id!(chip3),
            id!(chip4),
            id!(chip5),
            id!(chip6),
            id!(chip7),
        ];
        for (i, chip_id) in CHIP_IDS.iter().enumerate() {
            let chip = item.view(cx, &[*chip_id]);
            match chips.get(i) {
                Some(label) => {
                    chip.set_visible(cx, true);
                    item.label(cx, &[*chip_id, id!(chip_label)])
                        .set_text(cx, label);
                    let is_selected = selected == Some(label.as_str());
                    item.rm_chip(cx, &[*chip_id]).set_selected(cx, is_selected);
                }
                None => chip.set_visible(cx, false),
            }
        }
    }
}

impl Widget for LibraryScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let cols = grid_cols(state().nav.mode());
        self.query = self.view.text_input(cx, ids!(header.search_input)).text();
        self.rows = build_rows(&state(), cols, &self.query);
        let rows = self.rows.clone();

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    let Some(row) = rows.get(item_id) else {
                        continue;
                    };
                    match row {
                        GridRow::Section { title, note } => {
                            let item = list.item(cx, item_id, id!(Section));
                            item.label(cx, ids!(section_title)).set_text(cx, title);
                            item.label(cx, ids!(section_count)).set_text(cx, note);
                            item.draw_all_unscoped(cx);
                        }
                        GridRow::Cards(novels) => {
                            draw_card_row(cx, &mut list, item_id, novels, cols, &state());
                        }
                        GridRow::Chips(chips) => {
                            let item = list.item(cx, item_id, id!(Chips));
                            Self::draw_chip_row(cx, &item, chips, None);
                            item.draw_all_unscoped(cx);
                        }
                        GridRow::Empty => {
                            let item = list.item(cx, item_id, id!(Empty));
                            item.draw_all_unscoped(cx);
                        }
                        GridRow::Loading | GridRow::Error(_) | GridRow::Toggle { .. } => {}
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for LibraryScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let cols = grid_cols(state().nav.mode());
        let list = self.view.portal_list(cx, ids!(list));
        for (item_id, item) in list.items_with_actions(actions) {
            match self.rows.get(item_id).cloned() {
                Some(GridRow::Cards(novels)) => {
                    if let Some(novel) = card_novel_at(actions, cx, &item, &novels, cols) {
                        with_state_mut(|s| s.open_novel(novel));
                        cx.action(AppAction::StateChanged);
                    }
                }
                _ => {}
            }
        }

        if self
            .view
            .text_input(cx, ids!(header.search_input))
            .returned(actions)
            .is_some()
        {
            cx.action(AppAction::StateChanged);
        }
    }
}
