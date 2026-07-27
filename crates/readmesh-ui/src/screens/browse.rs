//! Browse / Discover screen: sources, featured, trending, recently
//! added/updated, and category entry points into search.

use makepad_widgets::*;
use readmesh_app::{AppState, ContentRepository, PrimaryTab};
use readmesh_core::NovelId;

use crate::app::AppAction;
use crate::components::RmChipWidgetRefExt;
use crate::screens::library::LibraryScreen;
use crate::screens::{GridRow, card_novel_at, draw_card_row, grid_cols, push_section};
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
            flow: Down spacing: 2
            new_batch: true
            Label{
                text: "Browse"
                draw_text.color: theme.color_label_inner
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_2}
            }
            Label{
                text: "Discover content across the mesh"
                draw_text.color: theme.color_label_inner_inactive
                draw_text.text_style.font_size: theme.font_size_p
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
            ToggleRow := CachedView{RmSettingsToggleRow{}}
        }
    }
}

fn build_rows(s: &AppState, cols: usize) -> Vec<GridRow> {
    let mut rows = Vec::new();

    // Available sources (designed to later list real P2P/distributed sources).
    let sources = s.catalog.sources();
    if !sources.is_empty() {
        rows.push(GridRow::Section {
            title: "Sources".to_string(),
            note: String::new(),
        });
        for source in sources {
            let enabled = s.settings.is_source_enabled(&source.id.0);
            rows.push(GridRow::Toggle {
                key: source.id.0.clone(),
                title: source.name.clone(),
                subtitle: format!("v{} · {} novels", source.version, source.novel_count),
                checked: enabled,
            });
        }
    }

    let featured: Vec<NovelId> = s.catalog.featured(4);
    push_section(&mut rows, "Featured", &featured, cols);

    let trending: Vec<NovelId> = s.catalog.trending(8);
    push_section(&mut rows, "Trending", &trending, cols);

    let added: Vec<NovelId> = s.catalog.recently_added(8);
    push_section(&mut rows, "Recently Added", &added, cols);

    let updated: Vec<NovelId> = s.catalog.recently_updated(8);
    push_section(&mut rows, "Recently Updated", &updated, cols);

    // Categories jump into a filtered search.
    let genres = s.catalog.genres();
    if !genres.is_empty() {
        rows.push(GridRow::Section {
            title: "Categories".to_string(),
            note: String::new(),
        });
        rows.push(GridRow::Chips(genres));
    }

    rows
}

/// The Browse screen widget.
#[derive(Script, ScriptHook, Widget)]
pub struct BrowseScreen {
    #[deref]
    view: View,
    #[rust]
    rows: Vec<GridRow>,
}

impl Widget for BrowseScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
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
                        GridRow::Toggle {
                            title,
                            subtitle,
                            checked,
                            ..
                        } => {
                            let item = list.item(cx, item_id, id!(ToggleRow));
                            item.label(cx, ids!(texts.set_title)).set_text(cx, title);
                            item.label(cx, ids!(texts.set_subtitle))
                                .set_text(cx, subtitle);
                            item.check_box(cx, ids!(set_toggle)).set_active(
                                cx,
                                *checked,
                                Animate::No,
                            );
                            item.draw_all_unscoped(cx);
                        }
                        GridRow::Empty | GridRow::Loading | GridRow::Error(_) => {}
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for BrowseScreen {
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
                            && let Some(genre) = chips.get(i).cloned()
                        {
                            // Jump to a genre-filtered search.
                            with_state_mut(|s| {
                                s.select_tab(PrimaryTab::Search);
                                s.search.filter.genre = Some(genre);
                                s.search.query.clear();
                                s.refresh_search();
                            });
                            cx.action(AppAction::StateChanged);
                        }
                    }
                }
                Some(GridRow::Toggle { key, .. }) => {
                    if let Some(enabled) = item.check_box(cx, ids!(set_toggle)).changed(actions) {
                        let _ = enabled;
                        with_state_mut(|s| {
                            s.settings.toggle_source(&key);
                        });
                        cx.action(AppAction::StateChanged);
                    }
                }
                _ => {}
            }
        }
    }
}
