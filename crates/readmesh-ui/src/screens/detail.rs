//! Content Detail screen: novel metadata, actions (read / bookmark /
//! library / download / share), reading progress, and the full chapter
//! list with read/unread + download state, sorting, filtering and refresh.

use makepad_widgets::*;
use readmesh_app::{ChapterSort, ContentRepository, DownloadStatus, Route};
use readmesh_core::{ChapterId, NovelId};

use crate::app::AppAction;
use crate::components::{RmCoverWidgetRefExt, RmProgressBarWidgetRefExt, tap_clicked};
use crate::screens::cover_color;
use crate::state::{state, with_state_mut};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // The detail header card: cover, metadata, summary, tags, progress,
    // and the action row.
    mod.widgets.RmDetailHeader = RoundedView{
        width: Fill height: Fit
        flow: Down spacing: theme.space_2
        padding: theme.mspace_3
        margin: theme.mspace_1{bottom: theme.space_2}
        draw_bg.color: theme.color_bg_container
        draw_bg.border_radius: 12.0
        new_batch: true

        top_row := View{
            width: Fill height: Fit
            flow: Right spacing: theme.space_3

            cover_column := View{
                width: 120 height: Fit
                flow: Down spacing: theme.space_1
                cover_frame := View{
                    width: 120 height: 170
                    detail_cover := RmCover{}
                }
                detail_status := Label{
                    width: Fill
                    text: "Ongoing"
                    draw_text.color: theme.color_highlight
                    draw_text.text_style.font_size: theme.font_size_code
                }
            }

            info := View{
                width: Fill height: Fit
                flow: Down spacing: theme.space_1
                detail_title := Label{
                    width: Fill
                    text: "Title"
                    draw_text.color: theme.color_label_inner
                    draw_text.text_style: theme.font_bold{font_size: theme.font_size_3}
                }
                detail_alt := Label{
                    width: Fill
                    text: ""
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_code
                }
                detail_author := Label{
                    width: Fill
                    text: ""
                    draw_text.color: theme.color_label_inner
                    draw_text.text_style.font_size: theme.font_size_p
                }
                detail_artist := Label{
                    width: Fill
                    text: ""
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_p
                }
                detail_source := Label{
                    width: Fill
                    text: ""
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_code
                }
            }
        }

        detail_summary := Label{
            width: Fill
            text: ""
            draw_text.color: theme.color_label_inner_inactive
            draw_text.text_style +: {font_size: theme.font_size_p, line_spacing: 1.4}
        }

        tags_row := View{
            width: Fill height: Fit
            flow: Flow.Right{wrap: true}
            spacing: theme.space_1
            tag0 := RmTagChip{}
            tag1 := RmTagChip{}
            tag2 := RmTagChip{}
            tag3 := RmTagChip{}
            tag4 := RmTagChip{}
        }

        progress_block := View{
            width: Fill height: Fit
            flow: Down spacing: theme.space_1
            detail_progress_label := Label{
                width: Fill
                text: ""
                draw_text.color: theme.color_label_inner_inactive
                draw_text.text_style.font_size: theme.font_size_code
            }
            detail_progress := RmProgressBar{}
        }

        actions_row := View{
            width: Fill height: Fit
            flow: Flow.Right{wrap: true}
            spacing: theme.space_2
            read_button := RmPrimaryButton{text: "Start Reading"}
            fav_button := RmSecondaryButton{text: "Bookmark"}
            library_button := RmSecondaryButton{text: "Add to Library"}
            download_all_button := RmSecondaryButton{text: "Download All"}
            share_button := RmSecondaryButton{text: "Share"}
        }
    }

    // Chapter list controls: sort toggle, filters, batch download, refresh.
    mod.widgets.RmChapterControls = View{
        width: Fill height: Fit
        flow: Right spacing: theme.space_2
        align: Align{y: 0.5}
        padding: theme.mspace_v_1
        sort_button := RmSmallButton{text: "Oldest first"}
        unread_check := CheckBox{text: "Unread"}
        downloaded_check := CheckBox{text: "Downloaded"}
        Filler{}
        download_unread_button := RmSmallButton{text: "Download unread"}
        refresh_button := RmSmallButton{text: "Refresh"}
    }

    mod.widgets.DetailScreen = #(DetailScreen::register_widget(vm)){
        width: Fill height: Fill
        flow: Down

        header := SolidView{
            width: Fill height: Fit
            padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
            draw_bg.color: theme.color_bg_app
            flow: Right spacing: theme.space_2
            align: Align{y: 0.5}
            new_batch: true
            back_button := RmSmallButton{text: "← Back"}
            nav_title := Label{
                width: Fill
                text: "Details"
                draw_text.color: theme.color_label_inner
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
            }
        }

        list := PortalList{
            width: Fill height: Fill
            flow: Down
            spacing: theme.space_1
            padding: theme.mspace_1{left: theme.space_3, right: theme.space_3}
            scroll_bar: ScrollBar{}

            Detail := CachedView{RmDetailHeader{}}
            Section := CachedView{RmSectionHeader{}}
            Controls := CachedView{RmChapterControls{}}
            Row := CachedView{RmChapterRow{}}
        }
    }
}

/// Rows of the detail screen list.
#[derive(Debug, Clone)]
enum DetailRow {
    Header,
    ChaptersSection { count: usize },
    Controls,
    Chapter(ChapterId),
}

/// The Content Detail screen widget (includes the chapter list).
#[derive(Script, ScriptHook, Widget)]
pub struct DetailScreen {
    #[deref]
    view: View,
    #[rust]
    rows: Vec<DetailRow>,
    #[rust]
    share_feedback: bool,
    #[rust]
    last_novel: Option<NovelId>,
}

/// The novel currently shown, from the navigation route.
fn current_novel() -> Option<NovelId> {
    match state().nav.current() {
        Route::NovelDetail(id) => Some(id),
        _ => None,
    }
}

impl DetailScreen {
    fn build_rows(novel: &NovelId) -> Vec<DetailRow> {
        let s = state();
        let chapters = s.visible_chapters(novel);
        let mut rows = vec![
            DetailRow::Header,
            DetailRow::ChaptersSection {
                count: chapters.len(),
            },
            DetailRow::Controls,
        ];
        rows.extend(chapters.iter().map(|c| DetailRow::Chapter(c.id)));
        rows
    }

    fn draw_header(cx: &mut Cx2d, item: &WidgetRef, novel_id: &NovelId, share_feedback: bool) {
        let s = state();
        let Some(novel) = s.catalog.novel(novel_id) else {
            return;
        };
        let meta = s.catalog.novel_meta(novel_id).unwrap_or_default();

        item.label(cx, ids!(top_row.info.detail_title))
            .set_text(cx, &novel.title);
        item.label(cx, ids!(top_row.info.detail_alt))
            .set_text(cx, &meta.alt_titles.join(" · "));
        let author = novel
            .authors
            .first()
            .map(|a| format!("by {}", a.name))
            .unwrap_or_default();
        item.label(cx, ids!(top_row.info.detail_author))
            .set_text(cx, &author);
        let artist = meta
            .artist
            .map(|a| format!("Art by {a}"))
            .unwrap_or_default();
        item.label(cx, ids!(top_row.info.detail_artist))
            .set_text(cx, &artist);
        item.label(cx, ids!(cover_column.detail_status))
            .set_text(cx, &format!("{:?}", novel.status));
        let source = novel
            .source_refs
            .first()
            .map(|r| r.plugin_id.0.clone())
            .unwrap_or_default();
        let chapters_total = s.catalog.chapters(novel_id).len();
        item.label(cx, ids!(top_row.info.detail_source))
            .set_text(cx, &format!("{source} · {chapters_total} chapters"));
        item.label(cx, ids!(detail_summary))
            .set_text(cx, novel.summary.as_deref().unwrap_or_default());

        // Cover.
        item.label(
            cx,
            ids!(cover_column.cover_frame.detail_cover.cover_initial),
        )
        .set_text(
            cx,
            &novel
                .title
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>())
                .unwrap_or_else(|| "R".into()),
        );
        item.rm_cover(cx, ids!(cover_column.cover_frame.detail_cover))
            .set_color(cover_color(novel_id));

        // Tags.
        const TAG_IDS: [LiveId; 5] = [id!(tag0), id!(tag1), id!(tag2), id!(tag3), id!(tag4)];
        for (i, tag_id) in TAG_IDS.iter().enumerate() {
            let tag = item.view(cx, ids!(tags_row)).view(cx, &[*tag_id]);
            match novel.tags.get(i) {
                Some(t) => {
                    tag.set_visible(cx, true);
                    item.label(cx, &[id!(tags_row), *tag_id, id!(chip_label)])
                        .set_text(cx, &t.name);
                }
                None => tag.set_visible(cx, false),
            }
        }

        // Reading progress.
        let progress = s.library.progress_for(novel_id);
        let read = s.library.read_count_for(novel_id);
        item.label(cx, ids!(progress_block.detail_progress_label))
            .set_text(cx, &format!("{read} of {chapters_total} chapters read"));
        item.rm_progress_bar(cx, ids!(progress_block.detail_progress))
            .set_progress(cx, progress);

        // Action button labels from state.
        let has_progress = s.library.has_saved_progress(novel_id);
        let read_label = if has_progress {
            "Continue Reading"
        } else {
            "Start Reading"
        };
        item.button(cx, ids!(actions_row.read_button))
            .set_text(cx, read_label);

        let fav_label = if s.library.is_favorite(novel_id) {
            "Bookmarked ✓"
        } else {
            "Bookmark"
        };
        item.button(cx, ids!(actions_row.fav_button))
            .set_text(cx, fav_label);

        let in_library = s.library.contains(novel_id);
        let library_label = if in_library {
            "In Library ✓"
        } else {
            "Add to Library"
        };
        item.button(cx, ids!(actions_row.library_button))
            .set_text(cx, library_label);

        let share_label = if share_feedback {
            "Link copied ✓"
        } else {
            "Share"
        };
        item.button(cx, ids!(actions_row.share_button))
            .set_text(cx, share_label);
    }

    fn draw_controls(cx: &mut Cx2d, item: &WidgetRef) {
        let s = state();
        let sort_label = match s.chapter_list.sort {
            ChapterSort::IndexAsc => "Oldest first",
            _ => "Newest first",
        };
        item.button(cx, ids!(sort_button)).set_text(cx, sort_label);
        item.check_box(cx, ids!(unread_check)).set_active(
            cx,
            s.chapter_list.filter.unread_only,
            Animate::No,
        );
        item.check_box(cx, ids!(downloaded_check)).set_active(
            cx,
            s.chapter_list.filter.downloaded_only,
            Animate::No,
        );
    }

    fn draw_chapter(cx: &mut Cx2d, item: &WidgetRef, novel_id: &NovelId, chapter_id: &ChapterId) {
        let s = state();
        let Some(chapter) = s
            .catalog
            .chapters(novel_id)
            .into_iter()
            .find(|c| &c.id == chapter_id)
        else {
            return;
        };

        item.label(cx, ids!(middle.row_title))
            .set_text(cx, &chapter.title);
        let date = chapter
            .published_at
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        let is_read = s.library.is_read(chapter_id);
        let read_note = if is_read { " · Read" } else { "" };
        item.label(cx, ids!(middle.row_meta))
            .set_text(cx, &format!("#{} · {date}{read_note}", chapter.index + 1));

        item.check_box(cx, ids!(read_check))
            .set_active(cx, is_read, Animate::No);

        // Download state presentation.
        let (state_text, show_button) = match s.downloads.status_of(chapter_id) {
            Some(DownloadStatus::Completed) => ("Downloaded ✓".to_string(), false),
            Some(DownloadStatus::Downloading) => {
                let progress = s
                    .downloads
                    .get(chapter_id)
                    .map(|d| (d.progress * 100.0) as u32)
                    .unwrap_or(0);
                (format!("{progress}%"), false)
            }
            Some(DownloadStatus::Queued) => ("Queued".to_string(), false),
            Some(DownloadStatus::Failed(_)) => ("Failed".to_string(), true),
            None => (String::new(), true),
        };
        item.label(cx, ids!(download_state))
            .set_text(cx, &state_text);
        item.button(cx, ids!(download_button))
            .set_visible(cx, show_button);
    }
}

impl Widget for DetailScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let Some(novel_id) = current_novel() else {
            return self.view.draw_walk(cx, scope, walk);
        };
        if self.last_novel != Some(novel_id) {
            self.last_novel = Some(novel_id);
            self.share_feedback = false;
        }
        self.rows = Self::build_rows(&novel_id);
        let rows = self.rows.clone();
        let share_feedback = self.share_feedback;

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, rows.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    match rows.get(item_id) {
                        Some(DetailRow::Header) => {
                            let item = list.item(cx, item_id, id!(Detail));
                            Self::draw_header(cx, &item, &novel_id, share_feedback);
                            item.draw_all_unscoped(cx);
                        }
                        Some(DetailRow::ChaptersSection { count }) => {
                            let item = list.item(cx, item_id, id!(Section));
                            item.label(cx, ids!(section_title)).set_text(cx, "Chapters");
                            let note = format!("{count}");
                            item.label(cx, ids!(section_count)).set_text(cx, &note);
                            item.draw_all_unscoped(cx);
                        }
                        Some(DetailRow::Controls) => {
                            let item = list.item(cx, item_id, id!(Controls));
                            Self::draw_controls(cx, &item);
                            item.draw_all_unscoped(cx);
                        }
                        Some(DetailRow::Chapter(chapter_id)) => {
                            let item = list.item(cx, item_id, id!(Row));
                            Self::draw_chapter(cx, &item, &novel_id, chapter_id);
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

impl WidgetMatchEvent for DetailScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let Some(novel_id) = current_novel() else {
            return;
        };

        // Back navigation.
        if self
            .view
            .button(cx, ids!(header.back_button))
            .clicked(actions)
        {
            with_state_mut(|s| {
                s.go_back();
            });
            cx.action(AppAction::StateChanged);
            return;
        }

        let list = self.view.portal_list(cx, ids!(list));
        for (item_id, item) in list.items_with_actions(actions) {
            match self.rows.get(item_id).cloned() {
                Some(DetailRow::Header) => {
                    if item
                        .button(cx, ids!(actions_row.read_button))
                        .clicked(actions)
                    {
                        with_state_mut(|s| s.start_reading(novel_id));
                        cx.action(AppAction::StateChanged);
                    }
                    if item
                        .button(cx, ids!(actions_row.fav_button))
                        .clicked(actions)
                    {
                        with_state_mut(|s| {
                            s.toggle_favorite(&novel_id);
                        });
                        cx.action(AppAction::StateChanged);
                    }
                    if item
                        .button(cx, ids!(actions_row.library_button))
                        .clicked(actions)
                    {
                        with_state_mut(|s| {
                            s.toggle_library_membership(&novel_id);
                        });
                        cx.action(AppAction::StateChanged);
                    }
                    if item
                        .button(cx, ids!(actions_row.download_all_button))
                        .clicked(actions)
                    {
                        with_state_mut(|s| {
                            s.download_all(&novel_id);
                        });
                        cx.action(AppAction::StateChanged);
                    }
                    if item
                        .button(cx, ids!(actions_row.share_button))
                        .clicked(actions)
                    {
                        let url = with_state_mut(|s| {
                            s.catalog
                                .novel(&novel_id)
                                .and_then(|n| n.source_refs.first().map(|r| r.remote_url.clone()))
                                .unwrap_or_default()
                        });
                        if !url.is_empty() {
                            cx.copy_to_clipboard(&url);
                        }
                        self.share_feedback = true;
                        cx.action(AppAction::StateChanged);
                    }
                }
                Some(DetailRow::Controls) => {
                    if item.button(cx, ids!(sort_button)).clicked(actions) {
                        with_state_mut(|s| s.chapter_list.toggle_sort_order());
                        cx.action(AppAction::StateChanged);
                    }
                    if let Some(unread) = item.check_box(cx, ids!(unread_check)).changed(actions) {
                        with_state_mut(|s| s.chapter_list.filter.unread_only = unread);
                        cx.action(AppAction::StateChanged);
                    }
                    if let Some(downloaded) =
                        item.check_box(cx, ids!(downloaded_check)).changed(actions)
                    {
                        with_state_mut(|s| {
                            s.chapter_list.filter.downloaded_only = downloaded;
                        });
                        cx.action(AppAction::StateChanged);
                    }
                    if item
                        .button(cx, ids!(download_unread_button))
                        .clicked(actions)
                    {
                        with_state_mut(|s| {
                            let unread: Vec<ChapterId> = s
                                .catalog
                                .chapters(&novel_id)
                                .iter()
                                .filter(|c| {
                                    !s.library.is_read(&c.id) && !s.downloads.is_downloaded(&c.id)
                                })
                                .map(|c| c.id)
                                .collect();
                            s.download_chapters(&novel_id, &unread);
                        });
                        cx.action(AppAction::StateChanged);
                    }
                    if item.button(cx, ids!(refresh_button)).clicked(actions) {
                        // The mock catalog is static; a real backend would
                        // re-fetch the chapter list here.
                        cx.action(AppAction::StateChanged);
                    }
                }
                Some(DetailRow::Chapter(chapter_id)) => {
                    // Tap the row -> start reading from this chapter.
                    if tap_clicked(actions, item.widget_uid()) {
                        with_state_mut(|s| s.open_chapter(novel_id, chapter_id));
                        cx.action(AppAction::StateChanged);
                        return;
                    }
                    // Toggle read/unread.
                    if let Some(read) = item.check_box(cx, ids!(read_check)).changed(actions) {
                        with_state_mut(|s| s.mark_chapter_read(&chapter_id, read));
                        cx.action(AppAction::StateChanged);
                    }
                    // Download this chapter.
                    if item.button(cx, ids!(download_button)).clicked(actions) {
                        with_state_mut(|s| {
                            s.download_chapter(&novel_id, &chapter_id);
                        });
                        cx.action(AppAction::StateChanged);
                    }
                }
                Some(DetailRow::ChaptersSection { .. }) | None => {}
            }
        }
    }
}
