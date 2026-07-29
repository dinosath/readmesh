use dioxus::prelude::*;

use readmesh_app::navigation::NavigationState;
use readmesh_app::repository::ContentRepository;
use readmesh_app::MockCatalog;

use crate::components::*;
use crate::theme::Theme;

#[component]
pub fn SearchScreen() -> Element {
    let theme = use_context::<Signal<Theme>>();
    let mut nav = use_context::<Signal<NavigationState>>();

    let mut query = use_signal(String::new);
    let catalog = MockCatalog::demo();

    let result_ids = catalog.search(&query.read());
    let results: Vec<_> = result_ids.iter().filter_map(|id| catalog.novel(id)).collect();

    let t = theme.read();
    let container_style = format!(
        "display:flex;flex-direction:column;gap:0;padding:{}px;\
         max-width:900px;margin:0 auto;width:100%;\
         box-sizing:border-box;",
        t.space_4
    );
    let search_bar_style = format!(
        "display:flex;align-items:center;gap:{}px;\
         background:{};border:1px solid {};\
         border-radius:{};padding:8px 12px;\
         margin-bottom:{}px;",
        t.space_2, t.bg_surface, t.border, t.radius_md, t.space_4
    );
    let input_style = format!(
        "flex:1;border:none;outline:none;background:transparent;\
         color:{};font-size:{};\
         font-family:inherit;",
        t.text_primary, t.font_size_base
    );
    let category_heading_style = format!(
        "font-size:{};font-weight:600;color:{};margin-bottom:{}px;",
        t.font_size_lg, t.text_primary, t.space_2
    );
    let category_grid_style = format!(
        "display:flex;flex-wrap:wrap;gap:{}px;",
        t.space_2
    );
    let category_tag_style = format!(
        "padding:8px 16px;border-radius:{};\
         background:{};border:1px solid {};\
         color:{};font-size:{};\
         cursor:pointer;",
        t.radius_full, t.bg_surface_2, t.border, t.text_secondary, t.font_size_sm
    );
    let results_header_style = format!(
        "font-size:{};font-weight:500;color:{};margin-bottom:{}px;",
        t.font_size_base, t.text_secondary, t.space_2
    );
    let results_list_style = format!(
        "display:flex;flex-direction:column;gap:{}px;",
        t.space_2
    );
    let result_item_style = format!(
        "display:flex;align-items:center;gap:{}px;\
         padding:{}px;border-radius:{};\
         cursor:pointer;\
         transition:background 0.15s;",
        t.space_3, t.space_2, t.radius_md
    );
    let result_cover_style = format!(
        "width:40px;height:56px;border-radius:{};\
         background:{};flex-shrink:0;\
         display:flex;align-items:center;\
         justify-content:center;font-size:20px;",
        t.radius_sm, t.accent_soft
    );
    let result_flex_style = "flex:1;".to_string();
    let result_title_style = format!(
        "font-weight:600;color:{};font-size:{};",
        t.text_primary, t.font_size_base
    );
    let empty_results_style = format!(
        "text-align:center;padding:{}px;color:{};",
        t.space_5, t.text_dim
    );
    let dim_color = t.text_dim.to_string();

    let query_empty = query.read().is_empty();
    let query_not_empty = !query_empty;
    let num_results = results.len();
    let results_empty = results.is_empty();
    let result_elements: Vec<Element> = results.iter().map(|item| {
        let id = item.id;
        let title = item.title.clone();
        rsx! {
            div {
                style: "{result_item_style}",
                onclick: move |_| nav.write().open_novel(id),
                div { style: "{result_cover_style}",
                    "📖"
                }
                div { style: "{result_flex_style}",
                    div { style: "{result_title_style}",
                        "{title}"
                    }
                }
            }
        }
    }).collect();
    let categories: Vec<Element> = catalog.genres().iter().map(|cat| {
        rsx! {
            div { style: "{category_tag_style}",
                "{cat}"
            }
        }
    }).collect();
    let no_results_msg = format!("No results found for \"{}\"", query.read());

    rsx! {
        div { style: "{container_style}",

            div { style: "{search_bar_style}",
                IconSearch { size: 18, color: Some(dim_color.clone()) }
                input {
                    style: "{input_style}",
                    placeholder: "Search novels...",
                    value: "{query.read()}",
                    oninput: move |e| { query.set(e.value()); },
                }
            }

            if query_empty {
                div { style: "{category_heading_style}",
                    "Browse Categories"
                }
                div { style: "{category_grid_style}",
                    {categories.into_iter()}
                }
            }

            if query_not_empty {
                div { style: "{results_header_style}",
                    "Results ({num_results})"
                }
                div { style: "{results_list_style}",
                    {result_elements.into_iter()}
                }
                if results_empty {
                    div { style: "{empty_results_style}",
                        "{no_results_msg}"
                    }
                }
            }
        }
    }
}
