//! Search screen: query input, history, sort/filter controls, and results
//! with loading / empty / error states. All query logic lives in
//! `readmesh_app::search` (unit tested); this widget is a projection.

use makepad_widgets::*;
use readmesh_app::{AppState, ContentRepository, SearchPhase, SearchSort};

use crate::app::AppAction;
use crate::components::RmChipWidgetRefExt;
use crate::screens::library::LibraryScreen;
use crate::screens::{GridRow, card_novel_at, draw_card_row, grid_cols, push_card_rows};
use crate::state::{state, with_state_mut};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.SearchScreen = #(SearchScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Down

        header := SolidView{
            width: Fill height: Fit
            padding: theme.mspace_3{left: theme.space_3 * 2, right: theme.space_3 * 2}
            draw_bg.color: theme.color_bg_app
            flow: Down spacing: theme.space_2
            new_batch: true

            Label{
                text: "Search"
                draw_text.color: theme.color_label_inner
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_2}
            }

            View{
                width: Fill height: Fit
                flow: Right spacing: theme.space_2
                align: Align{y: 0.5}
                search_input := RmTextInput{
                    empty_text: "Search titles, authors, tags…"
                }
                search_button := RmPrimaryButton{
                    text: "Search"
                }
            }

            View{
                width: Fill height: Fit
                flow: Right spacing: theme.space_2
                align: Align{y: 0.5}
                Label{
                    text: "Sort"
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_code
                }
                sort_dropdown := DropDown{
                    width: Fit
                    labels: ["Relevance", "Title A–Z", "Title Z–A", "Recently Updated"]
                }
                Label{
                    text: "Genre"
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_code
                }
                genre_dropdown := DropDown{
                    width: Fit
                    labels: ["All genres"]
                }
                Filler{}
                clear_history_button := RmSmallButton{
                    text: "Clear history"
                }
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
                    empty_title.text: "No results"
                    empty_body.text: "Try a different title, author, or tag"
                }
            }
            Hint := CachedView{
                RmEmptyState{
                    empty_title.text: "Search the mesh"
                    empty_body.text: "Find novels across every connected source"
                }
            }
            Loading := CachedView{RmLoadingState{}}
            Error := CachedView{RmErrorState{}}
        }
    }
}

const SORTS: [SearchSort; 4] = [
    SearchSort::Relevance,
    SearchSort::TitleAsc,
    SearchSort::TitleDesc,
    SearchSort::RecentlyUpdated,
];

fn build_rows(s: &AppState, cols: usize) -> Vec<GridRow> {
    let mut rows = Vec::new();
    match &s.search.phase {
        SearchPhase::Idle => {
            if !s.search.history.is_empty() {
                rows.push(GridRow::Section {
                    title: "Recent searches".to_string(),
                    note: String::new(),
                });
                rows.push(GridRow::Chips(
                    s.search.history.iter().take(8).cloned().collect(),
                ));
            }
            // Idle hint doubles as the "nothing searched yet" state.
            rows.push(GridRow::Empty);
        }
        SearchPhase::Loading => rows.push(GridRow::Loading),
        SearchPhase::Error(message) => rows.push(GridRow::Error(message.clone())),
        SearchPhase::Ready => {
            if s.search.results.is_empty() {
                rows.push(GridRow::Error(String::new()));
            } else {
                rows.push(GridRow::Section {
                    title: format!("Results for “{}”", s.search.query),
                    note: format!("{}", s.search.results.len()),
                });
                push_card_rows(&mut rows, &s.search.results, cols);
            }
        }
    }
    rows
}

/// The Search screen widget.
#[derive(Script, ScriptHook, Widget)]
pub struct SearchScreen {
    #[deref]
    view: View,
    #[rust]
    rows: Vec<GridRow>,
    #[rust]
    genres_initialized: bool,
}

impl SearchScreen {
    fn submit(&mut self, cx: &mut Cx) {
        let query = self.view.text_input(cx, ids!(header.search_input)).text();
        let started = with_state_mut(|s| s.begin_search(&query));
        if started {
            // App completes the search on the next tick (async-ready flow).
            cx.action(AppAction::SearchRequested);
        }
        cx.action(AppAction::StateChanged);
    }
}

impl Widget for SearchScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Populate the genre filter once from the catalog.
        if !self.genres_initialized {
            self.genres_initialized = true;
            let mut labels = vec!["All genres".to_string()];
            labels.extend(state().catalog.genres());
            self.view
                .drop_down(cx, ids!(header.genre_dropdown))
                .set_labels(cx, labels);
        }

        let cols = grid_cols(state().nav.mode());
        self.rows = build_rows(&state(), cols);
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
                            LibraryScreen::draw_chip_row(cx, &item, chips, None);
                            item.draw_all_unscoped(cx);
                        }
                        GridRow::Empty => {
                            // Idle hint state.
                            let item = list.item(cx, item_id, id!(Hint));
                            item.draw_all_unscoped(cx);
                        }
                        GridRow::Loading => {
                            let item = list.item(cx, item_id, id!(Loading));
                            item.draw_all_unscoped(cx);
                        }
                        GridRow::Error(message) => {
                            if message.is_empty() {
                                // Ready-with-no-results empty state.
                                let item = list.item(cx, item_id, id!(Empty));
                                item.draw_all_unscoped(cx);
                            } else {
                                let item = list.item(cx, item_id, id!(Error));
                                item.label(cx, ids!(error_label)).set_text(cx, message);
                                item.draw_all_unscoped(cx);
                            }
                        }
                        GridRow::Toggle { .. } => {}
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for SearchScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Submit via return key or the search button.
        if self
            .view
            .text_input(cx, ids!(header.search_input))
            .returned(actions)
            .is_some()
            || self
                .view
                .button(cx, ids!(header.search_button))
                .clicked(actions)
        {
            self.submit(cx);
        }

        // Sort selection.
        if let Some(index) = self
            .view
            .drop_down(cx, ids!(header.sort_dropdown))
            .selected(actions)
            && let Some(sort) = SORTS.get(index)
        {
            with_state_mut(|s| {
                s.search.set_sort(*sort);
                s.refresh_search();
            });
            cx.action(AppAction::StateChanged);
        }

        // Genre filter (0 = all genres).
        if let Some(index) = self
            .view
            .drop_down(cx, ids!(header.genre_dropdown))
            .selected(actions)
        {
            with_state_mut(|s| {
                if index == 0 {
                    s.search.filter.genre = None;
                } else if let Some(genre) = s.catalog.genres().get(index - 1) {
                    s.search.filter.genre = Some(genre.clone());
                }
                s.refresh_search();
            });
            cx.action(AppAction::StateChanged);
        }

        // Clear history.
        if self
            .view
            .button(cx, ids!(header.clear_history_button))
            .clicked(actions)
        {
            with_state_mut(|s| s.search.clear_history());
            cx.action(AppAction::StateChanged);
        }

        // List interactions.
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
                Some(GridRow::Chips(chips)) => {
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
                        if item.rm_chip(cx, &[*chip_id]).clicked(actions)
                            && let Some(query) = chips.get(i).cloned()
                        {
                            self.view
                                .text_input(cx, ids!(header.search_input))
                                .set_text(cx, &query);
                            self.submit(cx);
                        }
                    }
                }
                Some(GridRow::Error(message))
                    if !message.is_empty()
                        && item.button(cx, ids!(retry_button)).clicked(actions) =>
                {
                    with_state_mut(|s| s.refresh_search());
                    cx.action(AppAction::StateChanged);
                }
                _ => {}
            }
        }
    }
}
