use dioxus::prelude::*;

use readmesh_app::navigation::NavigationState;
use readmesh_app::repository::ContentRepository;
use readmesh_app::MockCatalog;

use crate::components::*;
use crate::theme::Theme;

#[component]
pub fn BrowseScreen() -> Element {
    let theme = use_context::<Signal<Theme>>();
    let mut nav = use_context::<Signal<NavigationState>>();

    let catalog = MockCatalog::demo();
    let trending_ids = catalog.trending(10);
    let trending: Vec<_> = trending_ids.iter().filter_map(|id| catalog.novel(id)).collect();

    let t = theme.read();
    let container_style = format!(
        "display:flex;flex-direction:column;gap:0;padding:{}px;\
         max-width:900px;margin:0 auto;width:100%;\
         box-sizing:border-box;",
        t.space_4
    );
    let heading_style = format!(
        "font-size:{};font-weight:600;color:{};margin-bottom:{}px;",
        t.font_size_2xl, t.text_primary, t.space_3
    );
    let subheading_style = format!(
        "font-size:{};color:{};margin-bottom:{}px;",
        t.font_size_base, t.text_secondary, t.space_4
    );
    let trending_label_style = format!(
        "font-size:{};font-weight:600;color:{};margin-bottom:{}px;",
        t.font_size_lg, t.text_primary, t.space_2
    );
    let grid_style = format!(
        "display:grid;grid-template-columns:repeat(auto-fill,minmax(140px,1fr));gap:{}px;",
        t.space_3
    );
    let cursor_pointer_style = "cursor:pointer;".to_string();
    let item_card_style = format!(
        "width:100%;aspect-ratio:3/4;\
         border-radius:{};\
         background:{};\
         margin-bottom:6px;\
         display:flex;align-items:center;\
         justify-content:center;\
         font-size:28px;",
        t.radius_lg, t.accent_soft
    );
    let item_title_style = format!(
        "font-size:{};font-weight:500;color:{};\
         overflow:hidden;text-overflow:ellipsis;\
         white-space:nowrap;",
        t.font_size_sm, t.text_primary
    );

    let trending_elements: Vec<Element> = trending.iter().map(|item| {
        let id = item.id;
        let title = item.title.clone();
        rsx! {
            div {
                style: "{cursor_pointer_style}",
                onclick: move |_| nav.write().open_novel(id),
                div { style: "{item_card_style}",
                    "📖"
                }
                div { style: "{item_title_style}",
                    "{title}"
                }
            }
        }
    }).collect();

    rsx! {
        div { style: "{container_style}",

            div { style: "{heading_style}",
                "Discover"
            }
            div { style: "{subheading_style}",
                "Explore trending and recommended novels."
            }

            div { style: "{trending_label_style}",
                "Trending Now"
            }
            div { style: "{grid_style}",
                {trending_elements.into_iter()}
            }

            RmSecondaryButton {
                full_width: true,
                on_click: move |_| {},
                "Load More"
            }
        }
    }
}
