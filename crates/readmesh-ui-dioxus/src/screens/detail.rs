use dioxus::prelude::*;

use readmesh_app::navigation::NavigationState;
use readmesh_app::repository::ContentRepository;
use readmesh_app::MockCatalog;
use readmesh_core::NovelId;

use crate::components::*;
use crate::theme::Theme;

#[derive(Props, Clone, PartialEq)]
pub struct NovelDetailScreenProps {
    novel_id: NovelId,
}

#[component]
pub fn NovelDetailScreen(props: NovelDetailScreenProps) -> Element {
    let theme = use_context::<Signal<Theme>>();
    let mut nav = use_context::<Signal<NavigationState>>();

    let catalog = MockCatalog::demo();
    let novel = catalog.novel(&props.novel_id);

    if let Some(novel) = novel {
        let chapters = catalog.chapters(&props.novel_id);
        let mut tab = use_signal(|| 0);
        let author = novel.authors.first().map(|a| a.name.clone()).unwrap_or_default();

        let t = theme.read();
        let container_style = format!(
            "display:flex;flex-direction:column;padding:{}px;\
             max-width:900px;margin:0 auto;width:100%;\
             box-sizing:border-box;",
            t.space_4
        );
        let header_style = format!(
            "display:flex;gap:{}px;margin-bottom:{}px;",
            t.space_4, t.space_3
        );
        let cover_style = format!(
            "width:120px;height:180px;border-radius:{};\
             background:{};flex-shrink:0;\
             display:flex;align-items:center;\
             justify-content:center;font-size:48px;",
            t.radius_lg, t.accent_soft
        );
        let info_style = "flex:1;display:flex;flex-direction:column;gap:8px;".to_string();
        let novel_title_style = format!(
            "font-size:{};font-weight:700;color:{};",
            t.font_size_2xl, t.text_primary
        );
        let meta_row_style = "display:flex;align-items:center;gap:8px;flex-wrap:wrap;".to_string();
        let author_style = format!("font-size:{};color:{};", t.font_size_base, t.text_secondary);
        let summary_style = format!(
            "font-size:{};color:{};\
             line-height:1.6;display:-webkit-box;\
             -webkit-line-clamp:3;-webkit-box-orient:vertical;\
             overflow:hidden;",
            t.font_size_base, t.text_secondary
        );
        let action_row_style = format!("display:flex;gap:{}px;margin-top:auto;", t.space_2);
        let tab_bar_style = format!(
            "display:flex;gap:0;margin-bottom:{}px;\
             border-bottom:2px solid {};",
            t.space_3, t.border
        );
        let tab_content_style = "display:flex;flex-direction:column;".to_string();
        let chapter_row_style = format!(
            "display:flex;align-items:center;\
             padding:{}px 0;border-bottom:1px solid {};\
             cursor:pointer;",
            t.space_2, t.border
        );
        let chapter_title_style = format!(
            "flex:1;font-size:{};color:{};",
            t.font_size_base, t.text_primary
        );
        let details_text_style = format!(
            "font-size:{};color:{};line-height:1.8;",
            t.font_size_base, t.text_secondary
        );
        let related_grid_style = format!(
            "display:grid;grid-template-columns:repeat(auto-fill,minmax(120px,1fr));gap:{}px;",
            t.space_2
        );
        let cursor_pointer_style = "cursor:pointer;".to_string();
        let related_card_style = format!(
            "width:100%;aspect-ratio:3/4;\
             border-radius:{};background:{};\
             margin-bottom:6px;\
             display:flex;align-items:center;\
             justify-content:center;font-size:20px;",
            t.radius_md, t.accent_soft
        );
        let related_title_style = format!(
            "font-size:{};color:{};\
             overflow:hidden;text-overflow:ellipsis;\
             white-space:nowrap;",
            t.font_size_sm, t.text_primary
        );
        let dim_color = t.text_dim.to_string();

        let novel_summary = novel.summary.as_ref();
        let novel_title_display = novel.title.clone();
        let tab_0_active = *tab.read() == 0;
        let tab_1_active = *tab.read() == 1;
        let tab_2_active = *tab.read() == 2;

        let chapter_elements: Vec<Element> = chapters.iter().map(|ch| {
            let ch_id = ch.id;
            let ch_title = ch.title.clone();
            rsx! {
                div {
                    style: "{chapter_row_style}",
                    onclick: move |_| nav.write().open_reader(props.novel_id, ch_id),
                    div { style: "{chapter_title_style}",
                        "{ch_title}"
                    }
                    IconNext { size: 16, color: Some(dim_color.clone()) }
                }
            }
        }).collect();

        let related_items: Vec<Element> = catalog.all_novels().iter().take(4).map(|item| {
            let id = item.id;
            let title = item.title.clone();
            rsx! {
                div {
                    style: "{cursor_pointer_style}",
                    onclick: move |_| nav.write().open_novel(id),
                    div { style: "{related_card_style}",
                        "📖"
                    }
                    div { style: "{related_title_style}",
                        "{title}"
                    }
                }
            }
        }).collect();

        rsx! {
            div { style: "{container_style}",

                div { style: "{header_style}",
                    div { style: "{cover_style}",
                        "📖"
                    }
                    div { style: "{info_style}",
                        div { style: "{novel_title_style}",
                            "{novel_title_display}"
                        }
                        div { style: "{meta_row_style}",
                            div { style: "{author_style}",
                                "{author}"
                            }
                        }
                        if let Some(desc) = novel_summary {
                            div { style: "{summary_style}",
                                "{desc}"
                            }
                        }
                        div { style: "{action_row_style}",
                            RmPrimaryButton {
                                full_width: false,
                                on_click: move |_| {
                                    if let Some(ch) = chapters.first() {
                                        nav.write().open_reader(props.novel_id, ch.id);
                                    }
                                },
                                "Start Reading"
                            }
                            RmSecondaryButton {
                                full_width: false,
                                on_click: move |_| {},
                                "Add to Library"
                            }
                        }
                    }
                }

                div { style: "{tab_bar_style}",
                    DetailTab { label: "Chapters", active: tab_0_active, on_click: move |_| tab.set(0) }
                    DetailTab { label: "Details", active: tab_1_active, on_click: move |_| tab.set(1) }
                    DetailTab { label: "Related", active: tab_2_active, on_click: move |_| tab.set(2) }
                }

                if tab_0_active {
                    div { style: "{tab_content_style}",
                        {chapter_elements.into_iter()}
                    }
                }

                if tab_1_active {
                    div { style: "{details_text_style}",
                        if let Some(desc) = novel_summary {
                            "{desc}"
                        } else {
                            "No description available."
                        }
                    }
                }

                if tab_2_active {
                    div { style: "{related_grid_style}",
                        {related_items.into_iter()}
                    }
                }
            }
        }
    } else {
        let t = theme.read();
        let not_found_style = format!(
            "display:flex;align-items:center;justify-content:center;\
             height:100%;color:{};",
            t.text_dim
        );

        rsx! {
            div { style: "{not_found_style}",
                "Novel not found"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DetailTabProps {
    label: String,
    active: bool,
    on_click: EventHandler<MouseEvent>,
}

#[component]
fn DetailTab(props: DetailTabProps) -> Element {
    let theme = use_context::<Signal<Theme>>();

    let t = theme.read();
    let fg = if props.active { t.accent } else { t.text_dim };
    let border_css = if props.active {
        format!("2px solid {}", t.accent)
    } else {
        "2px solid transparent".into()
    };
    let tab_style = format!(
        "padding:12px 20px;font-size:{};font-weight:500;\
         color:{};border-bottom:{};\
         cursor:pointer;user-select:none;\
         transition:all 0.15s;",
        t.font_size_base, fg, border_css
    );

    rsx! {
        div {
            style: "{tab_style}",
            onclick: move |e| props.on_click.call(e),
            "{props.label}"
        }
    }
}
