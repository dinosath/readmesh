//! Downloads screen: active queue, failed downloads with retry, and
//! completed downloads with removal. The state machine lives in
//! `readmesh_app::downloads` (unit tested); the UI only projects it.

use makepad_widgets::*;
use readmesh_app::DownloadStatus;

use crate::app::AppAction;
use crate::components::RmProgressBarWidgetRefExt;
use crate::state::{state, with_state_mut};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.DownloadsScreen = #(DownloadsScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Down

        header := SolidView{
            width: Fill height: Fit
            padding: theme.mspace_3{left: theme.space_3 * 2, right: theme.space_3 * 2}
            draw_bg.color: theme.color_bg_app
            flow: Right
            align: Align{y: 0.5}
            new_batch: true
            Label{
                text: "Downloads"
                draw_text.color: theme.color_label_inner
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_2}
            }
            Filler{}
            clear_completed_button := RmSmallButton{text: "Clear completed"}
        }

        list := PortalList{
            width: Fill height: Fill
            flow: Down
            spacing: theme.space_1
            padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
            scroll_bar: ScrollBar{}

            Section := CachedView{RmSectionHeader{}}
            Item := CachedView{RmDownloadRow{}}
            Empty := CachedView{
                RmEmptyState{
                    empty_title.text: "No downloads"
                    empty_body.text: "Downloaded chapters are available offline"
                }
            }
        }
    }
}

/// Rows of the downloads list: section headers and download entries
/// (identified by their index in `DownloadManager::items()`).
#[derive(Debug, Clone)]
enum DownloadRow {
    Section { title: String, note: String },
    Item(usize),
    Empty,
}

fn build_rows() -> (Vec<DownloadRow>, bool) {
    let s = state();
    let mut rows = Vec::new();

    let active: Vec<usize> = s
        .downloads
        .items()
        .iter()
        .enumerate()
        .filter(|(_, d)| d.status.is_active())
        .map(|(i, _)| i)
        .collect();
    let failed: Vec<usize> = s
        .downloads
        .items()
        .iter()
        .enumerate()
        .filter(|(_, d)| matches!(d.status, DownloadStatus::Failed(_)))
        .map(|(i, _)| i)
        .collect();
    let completed: Vec<usize> = s
        .downloads
        .items()
        .iter()
        .enumerate()
        .filter(|(_, d)| matches!(d.status, DownloadStatus::Completed))
        .map(|(i, _)| i)
        .collect();

    let push_group = |title: &str, group: &[usize], rows: &mut Vec<DownloadRow>| {
        if !group.is_empty() {
            rows.push(DownloadRow::Section {
                title: title.to_string(),
                note: format!("{}", group.len()),
            });
            rows.extend(group.iter().map(|i| DownloadRow::Item(*i)));
        }
    };
    push_group("Active", &active, &mut rows);
    push_group("Failed", &failed, &mut rows);
    push_group("Completed", &completed, &mut rows);

    if rows.is_empty() {
        rows.push(DownloadRow::Empty);
    }
    let has_completed = !completed.is_empty();
    (rows, has_completed)
}

/// The Downloads screen widget.
#[derive(Script, ScriptHook, Widget)]
pub struct DownloadsScreen {
    #[deref]
    view: View,
    #[rust]
    rows: Vec<DownloadRow>,
}

impl DownloadsScreen {
    fn draw_item(cx: &mut Cx2d, item: &WidgetRef, download_index: usize) {
        let s = state();
        let Some(download) = s.downloads.items().get(download_index) else {
            return;
        };

        item.label(cx, ids!(top_row.titles.dl_title))
            .set_text(cx, &download.novel_title);
        item.label(cx, ids!(top_row.titles.dl_chapter))
            .set_text(cx, &download.chapter_title);

        let (status, show_retry, show_cancel, show_remove) = match &download.status {
            DownloadStatus::Queued => ("Queued".to_string(), false, true, false),
            DownloadStatus::Downloading => (
                format!("{}%", (download.progress * 100.0) as u32),
                false,
                true,
                false,
            ),
            DownloadStatus::Completed => ("Completed".to_string(), false, false, true),
            DownloadStatus::Failed(error) => (format!("Failed — {error}"), true, false, true),
        };
        item.label(cx, ids!(top_row.dl_status))
            .set_text(cx, &status);
        item.rm_progress_bar(cx, ids!(dl_progress))
            .set_progress(cx, download.progress);

        item.button(cx, ids!(actions_row.retry_button))
            .set_visible(cx, show_retry);
        item.button(cx, ids!(actions_row.cancel_button))
            .set_visible(cx, show_cancel);
        item.button(cx, ids!(actions_row.remove_button))
            .set_visible(cx, show_remove);
        item.view(cx, ids!(actions_row))
            .set_visible(cx, show_retry || show_cancel || show_remove);
    }
}

impl Widget for DownloadsScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let (rows, has_completed) = build_rows();
        self.rows = rows.clone();
        self.view
            .button(cx, ids!(header.clear_completed_button))
            .set_visible(cx, has_completed);

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    match rows.get(item_id) {
                        Some(DownloadRow::Section { title, note }) => {
                            let item = list.item(cx, item_id, id!(Section));
                            item.label(cx, ids!(section_title)).set_text(cx, title);
                            item.label(cx, ids!(section_count)).set_text(cx, note);
                            item.draw_all_unscoped(cx);
                        }
                        Some(DownloadRow::Item(index)) => {
                            let item = list.item(cx, item_id, id!(Item));
                            Self::draw_item(cx, &item, *index);
                            item.draw_all_unscoped(cx);
                        }
                        Some(DownloadRow::Empty) => {
                            let item = list.item(cx, item_id, id!(Empty));
                            item.draw_all_unscoped(cx);
                        }
                        None => {}
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl WidgetMatchEvent for DownloadsScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Clear all completed downloads.
        if self
            .view
            .button(cx, ids!(header.clear_completed_button))
            .clicked(actions)
        {
            with_state_mut(|s| {
                s.downloads.clear_completed();
            });
            cx.action(AppAction::StateChanged);
            return;
        }

        let list = self.view.portal_list(cx, ids!(list));
        for (item_id, item) in list.items_with_actions(actions) {
            let Some(DownloadRow::Item(index)) = self.rows.get(item_id) else {
                continue;
            };
            let chapter_id = {
                let s = state();
                s.downloads.items().get(*index).map(|d| d.chapter_id)
            };
            let Some(chapter_id) = chapter_id else {
                continue;
            };

            if item
                .button(cx, ids!(actions_row.retry_button))
                .clicked(actions)
            {
                with_state_mut(|s| {
                    s.downloads.retry(&chapter_id);
                });
                cx.action(AppAction::StateChanged);
            }
            if item
                .button(cx, ids!(actions_row.cancel_button))
                .clicked(actions)
            {
                with_state_mut(|s| {
                    s.downloads.cancel(&chapter_id);
                });
                cx.action(AppAction::StateChanged);
            }
            if item
                .button(cx, ids!(actions_row.remove_button))
                .clicked(actions)
            {
                with_state_mut(|s| {
                    s.downloads.remove(&chapter_id);
                });
                cx.action(AppAction::StateChanged);
            }
        }
    }
}
