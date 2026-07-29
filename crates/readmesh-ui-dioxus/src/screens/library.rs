use dioxus::prelude::*;

use readmesh_app::navigation::{NavigationState, Route};
use readmesh_app::repository::ContentRepository;
use readmesh_app::MockCatalog;

use crate::components::*;
use crate::theme::Theme;

#[component]
pub fn LibraryScreen() -> Element {
    let theme = use_context::<Signal<Theme>>();
    let mut nav = use_context::<Signal<NavigationState>>();

    let catalog = MockCatalog::demo();
    let novels = catalog.all_novels();

    let t = theme.read();
    let container_style = format!(
        "display:flex;flex-direction:column;gap:0;padding:{}px;\
         max-width:900px;margin:0 auto;width:100%;box-sizing:border-box;",
        t.space_4
    );
    let continue_header_style = format!(
        "display:flex;align-items:center;justify-content:space-between;margin-bottom:{}px;",
        t.space_2
    );
    let section_title_style = format!(
        "font-size:{};font-weight:600;color:{};",
        t.font_size_xl, t.text_primary
    );
    let continue_card_style = format!(
        "display:flex;align-items:center;gap:{}px;\
         background:{};border:1px solid {};\
         border-radius:{};padding:{}px;\
         margin-bottom:{}px;cursor:pointer;\
         transition:border-color 0.15s;",
        t.space_3, t.bg_surface, t.border, t.radius_lg, t.space_3, t.space_2
    );
    let card_cover_style = format!(
        "width:48px;height:48px;border-radius:{};\
         background:{};flex-shrink:0;",
        t.radius_md, t.accent_soft
    );
    let card_flex_style = "flex:1;".to_string();
    let card_title_style = format!(
        "font-size:{};font-weight:600;color:{};",
        t.font_size_base, t.text_primary
    );
    let card_subtitle_style = format!(
        "font-size:{};color:{};",
        t.font_size_sm, t.text_dim
    );
    let progress_bg_style = format!(
        "height:4px;background:{};\
         border-radius:2px;margin-top:6px;\
         position:relative;overflow:hidden;",
        t.border
    );
    let progress_fill_style = format!(
        "height:100%;width:35%;background:{};\
         border-radius:2px;",
        t.accent
    );
    let progress_pct_style = format!(
        "font-size:{};color:{};",
        t.font_size_sm, t.text_dim
    );
    let library_header_style = format!(
        "display:flex;align-items:center;justify-content:space-between;margin-bottom:{}px;",
        t.space_2
    );
    let empty_state_style = format!(
        "display:flex;flex-direction:column;align-items:center;\
         justify-content:center;padding:{}px 0;color:{};gap:8px;",
        t.space_5, t.text_dim
    );
    let empty_subtitle_style = format!("font-size:{};", t.font_size_sm);
    let grid_style = format!(
        "display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:{}px;",
        t.space_3
    );
    let grid_card_style = format!(
        "width:100%;aspect-ratio:3/4;\
         border-radius:{};\
         background:{};\
         margin-bottom:8px;\
         display:flex;align-items:center;\
         justify-content:center;\
         font-size:36px;color:{};",
        t.radius_lg, t.accent_soft, t.accent_dim
    );
    let grid_item_title_style = format!(
        "font-size:{};font-weight:500;color:{};\
         overflow:hidden;text-overflow:ellipsis;\
         white-space:nowrap;",
        t.font_size_base, t.text_primary
    );
    let dim_color = t.text_dim.to_string();
    let cursor_pointer_style = "cursor:pointer;".to_string();

    let first_novel = novels.first();
    let novels_empty = novels.is_empty();
    let grid_items: Vec<Element> = novels.iter().map(|item| {
        let item_id = item.id;
        let title = item.title.clone();
        rsx! {
            div {
                style: "{cursor_pointer_style}",
                onclick: move |_| nav.write().open_novel(item_id),
                div { style: "{grid_card_style}",
                    "📖"
                }
                div { style: "{grid_item_title_style}",
                    "{title}"
                }
            }
        }
    }).collect();

    rsx! {
        div { style: "{container_style}",

            if let Some(item) = first_novel {
                div { style: "{continue_header_style}",
                    div { style: "{section_title_style}",
                        "Continue Reading"
                    }
                }
                div { style: "{continue_card_style}",
                    div { style: "{card_cover_style}",
                    }
                    div { style: "{card_flex_style}",
                        div { style: "{card_title_style}",
                            "{item.title}"
                        }
                        div { style: "{card_subtitle_style}",
                            "Continue where you left off"
                        }
                        div { style: "{progress_bg_style}",
                            div { style: "{progress_fill_style}",
                            }
                        }
                    }
                    div { style: "{progress_pct_style}",
                        "35%"
                    }
                }
            }

            div { style: "{library_header_style}",
                div { style: "{section_title_style}",
                    "Library"
                }
                RmSmallButton {
                    on_click: move |_| nav.write().push(Route::CreateNovel),
                    IconBook { size: 14 }
                    "Create"
                }
            }

            if novels_empty {
                div { style: "{empty_state_style}",
                    IconBook { size: 40, color: Some(dim_color.clone()) }
                    div { "Your library is empty" }
                    div { style: "{empty_subtitle_style}",
                        "Use Discover to find new novels"
                    }
                }
            } else {
                div { style: "{grid_style}",
                    {grid_items.into_iter()}
                }
            }
        }
    }
}
