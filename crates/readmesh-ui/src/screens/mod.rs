//! The eight ReadMesh screens. Each screen is a custom widget that projects
//! `readmesh_app::AppState` — screens own no business logic.
//!
//! Shared infrastructure lives here: the grid row model used by
//! card-grid screens, card population, and the screen widget registration
//! order.

pub mod browse;
pub mod chapter_editor;
pub mod collaborative;
pub mod cover_studio;
pub mod create;
pub mod detail;
pub mod discover;
pub mod downloads;
pub mod import;
pub mod library;
pub mod metadata_editor;
pub mod onboarding;
pub mod peer_connections;
pub mod reader;
pub mod search;
pub mod settings;
pub mod sync_dashboard;

use makepad_widgets::*;
use readmesh_app::{AppState, ContentRepository, NavMode};
use readmesh_core::{Novel, NovelId};

use crate::components::{RmCoverWidgetRefExt, RmProgressBarWidgetRefExt, RmTapWidgetRefExt};

/// Register all screen widgets (order: leaves first, reader last).
pub fn script_mod(vm: &mut ScriptVm) {
    crate::screens::library::script_mod(vm);
    crate::screens::browse::script_mod(vm);
    crate::screens::discover::script_mod(vm);
    crate::screens::search::script_mod(vm);
    crate::screens::detail::script_mod(vm);
    crate::screens::downloads::script_mod(vm);
    crate::screens::reader::script_mod(vm);
    crate::screens::settings::script_mod(vm);
    crate::screens::onboarding::script_mod(vm);
    crate::screens::create::script_mod(vm);
    crate::screens::metadata_editor::script_mod(vm);
    crate::screens::cover_studio::script_mod(vm);
    crate::screens::import::script_mod(vm);
    crate::screens::chapter_editor::script_mod(vm);
    crate::screens::collaborative::script_mod(vm);
    crate::screens::peer_connections::script_mod(vm);
    crate::screens::sync_dashboard::script_mod(vm);
}

/// One row in a sectioned card-grid screen.
#[derive(Debug, Clone)]
pub enum GridRow {
    /// A section header with a title and a count/annotation.
    Section { title: String, note: String },
    /// A row of content cards (up to `cols` items).
    Cards(Vec<NovelId>),
    /// A row of tappable chips (categories, search history).
    Chips(Vec<String>),
    /// A source/settings-style toggle row.
    Toggle {
        key: String,
        title: String,
        subtitle: String,
        checked: bool,
    },
    /// Empty state placeholder.
    Empty,
    /// Loading state placeholder.
    Loading,
    /// Error state with a message.
    Error(String),
}

/// Grid column count per layout mode (responsive grid).
pub fn grid_cols(mode: NavMode) -> usize {
    match mode {
        NavMode::Mobile => 2,
        NavMode::Tablet => 3,
        NavMode::Desktop => 4,
    }
}

/// The PortalList template used for a card row with `cols` columns.
pub fn row_template(cols: usize) -> LiveId {
    match cols {
        2 => id!(Row2),
        3 => id!(Row3),
        _ => id!(Row4),
    }
}

/// Push card rows for `ids` chunked by `cols`.
pub fn push_card_rows(rows: &mut Vec<GridRow>, ids: &[NovelId], cols: usize) {
    for chunk in ids.chunks(cols.max(1)) {
        rows.push(GridRow::Cards(chunk.to_vec()));
    }
}

/// Push a section header followed by card rows (skipped when `ids` is empty).
pub fn push_section(rows: &mut Vec<GridRow>, title: &str, ids: &[NovelId], cols: usize) {
    if ids.is_empty() {
        return;
    }
    rows.push(GridRow::Section {
        title: title.to_string(),
        note: format!("{}", ids.len()),
    });
    push_card_rows(rows, ids, cols);
}

/// Deterministic cover colors derived from the novel id (placeholder covers
/// until real cover fetching lands).
const COVER_COLORS: [Vec4f; 8] = [
    vec4(0.23, 0.29, 0.40, 1.0),
    vec4(0.29, 0.27, 0.44, 1.0),
    vec4(0.44, 0.27, 0.27, 1.0),
    vec4(0.27, 0.44, 0.33, 1.0),
    vec4(0.44, 0.39, 0.24, 1.0),
    vec4(0.24, 0.36, 0.44, 1.0),
    vec4(0.37, 0.26, 0.44, 1.0),
    vec4(0.44, 0.26, 0.35, 1.0),
];

pub fn cover_color(id: &NovelId) -> Vec4f {
    COVER_COLORS[id.as_bytes()[0] as usize % COVER_COLORS.len()]
}

fn cover_initial(title: &str) -> String {
    title
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "R".to_string())
}

/// Populate one card (`card0..card3`) inside a grid row item.
pub fn populate_card(
    cx: &mut Cx2d,
    item: &WidgetRef,
    card_id: LiveId,
    novel: &Novel,
    progress: f32,
    meta: &str,
    total_chapters: usize,
    read_count: usize,
) {
    item.label(cx, &[card_id, id!(card_title)])
        .set_text(cx, &novel.title);
    let subtitle = novel
        .authors
        .first()
        .map(|a| a.name.as_str())
        .unwrap_or_default();
    item.label(cx, &[card_id, id!(card_subtitle)])
        .set_text(cx, subtitle);
    item.label(cx, &[card_id, id!(card_meta)])
        .set_text(cx, meta);

    // Placeholder cover: deterministic color + title initial.
    item.label(
        cx,
        &[card_id, id!(cover_wrap), id!(cover), id!(cover_initial)],
    )
    .set_text(cx, &cover_initial(&novel.title));
    item.rm_cover(cx, &[card_id, id!(cover_wrap), id!(cover)])
        .set_color(cover_color(&novel.id));

    // Unread count badge (top-left corner of cover).
    let unread = total_chapters.saturating_sub(read_count);
    let badge = item.view(cx, &[card_id, id!(cover_wrap), id!(unread_badge)]);
    let show_badge = unread > 0;
    badge.set_visible(cx, show_badge);
    if show_badge {
        item.label(
            cx,
            &[card_id, id!(cover_wrap), id!(unread_badge), id!(badge_label)],
        )
        .set_text(cx, &unread.to_string());
    }

    // Reading progress bar along the cover bottom (hidden at 0).
    let wrap = item.view(cx, &[card_id, id!(cover_wrap), id!(progress_wrap)]);
    wrap.set_visible(cx, progress > 0.001);
    item.rm_progress_bar(
        cx,
        &[
            card_id,
            id!(cover_wrap),
            id!(progress_wrap),
            id!(card_progress),
        ],
    )
    .set_progress(cx, progress);
}

/// Hide a card slot (fewer novels than columns in the last row).
pub fn hide_card(cx: &mut Cx2d, item: &WidgetRef, card_id: LiveId) {
    item.view(cx, &[card_id]).set_visible(cx, false);
}

const CARD_IDS: [LiveId; 4] = [id!(card0), id!(card1), id!(card2), id!(card3)];

/// Draw one grid row item. Returns false if the template didn't match.
pub fn draw_card_row(
    cx: &mut Cx2d,
    list: &mut PortalList,
    item_id: usize,
    novels: &[NovelId],
    cols: usize,
    state: &AppState,
) {
    let item = list.item(cx, item_id, row_template(cols));
    for (slot, card_id) in CARD_IDS.iter().take(cols).enumerate() {
        match novels.get(slot) {
            Some(novel_id) => {
                if let Some(novel) = state.catalog.novel(novel_id) {
                    let progress = state.library.progress_for(novel_id);
                    let chapters = state.catalog.chapters(novel_id).len();
                    let status = format!("{:?}", novel.status);
                    let meta = format!("{chapters} chapters · {status}");
                    let read_count = state.library.read_count_for(novel_id);
                    populate_card(cx, &item, *card_id, &novel, progress, &meta, chapters, read_count);
                } else {
                    hide_card(cx, &item, *card_id);
                }
            }
            None => hide_card(cx, &item, *card_id),
        }
    }
    item.draw_all_unscoped(cx);
}

/// Map a tap on a card in a grid row back to the novel id.
pub fn card_novel_at(
    actions: &Actions,
    cx: &mut Cx,
    item: &WidgetRef,
    novels: &[NovelId],
    cols: usize,
) -> Option<NovelId> {
    for (slot, card_id) in CARD_IDS.iter().take(cols).enumerate() {
        if item.rm_tap(cx, &[*card_id]).clicked(actions) {
            return novels.get(slot).copied();
        }
    }
    None
}

/// A short "N chapters · Status" meta line for cards.
pub fn card_meta(state: &AppState, novel: &Novel) -> String {
    let chapters = state.catalog.chapters(&novel.id).len();
    format!("{chapters} chapters · {:?}", novel.status)
}
