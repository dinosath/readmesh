use dioxus::prelude::*;

use readmesh_core::NovelId;

use crate::components::*;
use crate::theme::Theme;

struct MockDownload {
    novel_id: NovelId,
    novel_title: String,
    progress: f32,
    status: MockDlStatus,
    size_bytes: u64,
}

enum MockDlStatus {
    Completed,
    Downloading,
    Queued,
}

fn mock_downloads() -> Vec<MockDownload> {
    vec![
        MockDownload {
            novel_id: NovelId(blake3::hash(b"1")),
            novel_title: "The Wandering Inn".into(),
            progress: 1.0,
            status: MockDlStatus::Completed,
            size_bytes: 2_340_000,
        },
        MockDownload {
            novel_id: NovelId(blake3::hash(b"2")),
            novel_title: "Lord of the Mysteries".into(),
            progress: 1.0,
            status: MockDlStatus::Completed,
            size_bytes: 4_120_000,
        },
        MockDownload {
            novel_id: NovelId(blake3::hash(b"3")),
            novel_title: "Reverend Insanity".into(),
            progress: 0.64,
            status: MockDlStatus::Downloading,
            size_bytes: 1_800_000,
        },
        MockDownload {
            novel_id: NovelId(blake3::hash(b"4")),
            novel_title: "Mother of Learning".into(),
            progress: 0.0,
            status: MockDlStatus::Queued,
            size_bytes: 0,
        },
    ]
}

#[component]
pub fn DownloadsScreen() -> Element {
    let theme = use_context::<Signal<Theme>>();
    let downloads = mock_downloads();
    let active: Vec<_> = downloads.iter().filter(|d| matches!(d.status, MockDlStatus::Downloading | MockDlStatus::Queued)).collect();
    let completed: Vec<_> = downloads.iter().filter(|d| matches!(d.status, MockDlStatus::Completed)).collect();

    let t = theme.read();
    let container_style = format!(
        "display:flex;flex-direction:column;gap:0;padding:{}px;\
         max-width:900px;margin:0 auto;width:100%;\
         box-sizing:border-box;",
        t.space_4
    );
    let page_title_style = format!(
        "font-size:{};font-weight:600;color:{};margin-bottom:{}px;",
        t.font_size_2xl, t.text_primary, t.space_3
    );
    let section_title_style = format!(
        "font-size:{};font-weight:600;color:{};margin-bottom:{}px;",
        t.font_size_lg, t.text_primary, t.space_2
    );
    let section_title_margin_style = format!(
        "font-size:{};font-weight:600;color:{};margin-bottom:{}px;margin-top:{}px;",
        t.font_size_lg, t.text_primary, t.space_2, t.space_2
    );
    let active_card_style = format!(
        "display:flex;align-items:center;gap:{}px;\
         background:{};border:1px solid {};\
         border-radius:{};padding:{}px;\
         margin-bottom:{}px;",
        t.space_3, t.bg_surface, t.border, t.radius_lg, t.space_3, t.space_2
    );
    let cover_style = format!(
        "width:40px;height:56px;border-radius:{};\
         background:{};flex-shrink:0;\
         display:flex;align-items:center;\
         justify-content:center;",
        t.radius_sm, t.accent_soft
    );
    let flex_1_style = "flex:1;".to_string();
    let novel_title_style = format!(
        "font-weight:600;color:{};font-size:{};",
        t.text_primary, t.font_size_base
    );
    let progress_bg_style = format!(
        "height:4px;background:{};\
         border-radius:2px;margin-top:6px;\
         overflow:hidden;",
        t.border
    );
    let download_info_style = format!(
        "font-size:{};color:{};margin-top:4px;",
        t.font_size_sm, t.text_dim
    );
    let completed_row_style = format!(
        "display:flex;align-items:center;gap:{}px;\
         padding:{}px 0;border-bottom:1px solid {};",
        t.space_3, t.space_2, t.border
    );
    let completed_cover_style = format!(
        "width:40px;height:56px;border-radius:{};\
         background:{};flex-shrink:0;\
         display:flex;align-items:center;\
         justify-content:center;",
        t.radius_sm, t.ok
    );
    let completed_info_style = format!(
        "font-size:{};color:{};",
        t.font_size_sm, t.text_dim
    );
    let empty_style = format!(
        "display:flex;flex-direction:column;align-items:center;\
         padding:{}px 0;color:{};gap:8px;",
        t.space_5, t.text_dim
    );
    let dim_color = t.text_dim.to_string();
    let ok_color = t.ok.to_string();

    let has_active = !active.is_empty();
    let has_completed = !completed.is_empty();
    let is_empty = downloads.is_empty();

    let active_elements: Vec<Element> = active.iter().map(|dl| {
        let pct = (dl.progress * 100.0) as i32;
        let bar_color = if matches!(dl.status, MockDlStatus::Downloading) { t.accent } else { t.text_dim };
        let fill_style = format!(
            "height:100%;width:{}%;background:{};border-radius:2px;",
            pct, bar_color
        );
        let dl_title = dl.novel_title.clone();
        let status_text = if matches!(dl.status, MockDlStatus::Downloading) {
            format!("Downloading... {:.0}%", dl.progress * 100.0)
        } else {
            "Waiting...".into()
        };
        rsx! {
            div { style: "{active_card_style}",
                div { style: "{cover_style}",
                    "📖"
                }
                div { style: "{flex_1_style}",
                    div { style: "{novel_title_style}",
                        "{dl_title}"
                    }
                    div { style: "{progress_bg_style}",
                        div { style: "{fill_style}",
                        }
                    }
                    div { style: "{download_info_style}",
                        "{status_text}"
                    }
                }
            }
        }
    }).collect();

    let completed_count = completed.len();
    let completed_elements: Vec<Element> = completed.iter().map(|dl| {
        let dl_title = dl.novel_title.clone();
        let size_str = format!("{:.1} MB", dl.size_bytes as f64 / 1_000_000.0);
        rsx! {
            div { style: "{completed_row_style}",
                div { style: "{completed_cover_style}",
                    IconCheck { size: 18, color: Some(ok_color.clone()) }
                }
                div { style: "{flex_1_style}",
                    div { style: "{novel_title_style}",
                        "{dl_title}"
                    }
                    div { style: "{completed_info_style}",
                        "{size_str}"
                    }
                }
                IconTrash { size: 16, color: Some(dim_color.clone()) }
            }
        }
    }).collect();

    rsx! {
        div { style: "{container_style}",

            div { style: "{page_title_style}",
                "Downloads"
            }

            if has_active {
                div { style: "{section_title_style}",
                    "Active"
                }
                {active_elements.into_iter()}
            }

            if has_completed {
                div { style: "{section_title_margin_style}",
                    "Downloaded ({completed_count})"
                }
                {completed_elements.into_iter()}
            }

            if is_empty {
                div { style: "{empty_style}",
                    IconDownload { size: 40, color: Some(dim_color.clone()) }
                    div { "No downloads yet" }
                }
            }
        }
    }
}
