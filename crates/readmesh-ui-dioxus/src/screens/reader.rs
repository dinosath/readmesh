use dioxus::prelude::*;

use readmesh_app::navigation::NavigationState;
use readmesh_core::{ChapterId, NovelId};

use crate::components::*;
use crate::theme::Theme;

#[derive(Props, Clone, PartialEq)]
pub struct ReaderScreenProps {
    novel_id: NovelId,
    chapter_id: ChapterId,
}

#[component]
pub fn ReaderScreen(props: ReaderScreenProps) -> Element {
    let theme = use_context::<Signal<Theme>>();
    let mut nav = use_context::<Signal<NavigationState>>();

    let mut show_menu = use_signal(|| false);
    let mut show_fonts = use_signal(|| false);
    let mut show_themes = use_signal(|| false);

    let t = theme.read();
    let reader_bg = t.reader_bg;
    let reader_fg = t.reader_text;
    let body_style = format!(
        "display:flex;flex-direction:column;height:100vh;\
         background:{};color:{};",
        reader_bg, reader_fg
    );
    let top_bar_style = format!(
        "display:flex;align-items:center;justify-content:space-between;\
         padding:12px {}px;background:{};\
         border-bottom:1px solid {};\
         position:absolute;top:0;left:0;right:0;z-index:10;",
        t.space_4, reader_bg, t.border
    );
    let back_btn_style = "cursor:pointer;display:flex;align-items:center;gap:4px;".to_string();
    let chapter_label_style = format!("font-size:{};font-weight:600;", t.font_size_base);
    let icon_row_style = format!("display:flex;gap:{}px;", t.space_2);
    let icon_btn_style = "cursor:pointer;".to_string();
    let font_panel_style = format!(
        "display:flex;gap:{}px;padding:{}px {}px;\
         border-bottom:1px solid {};\
         position:absolute;top:48px;left:0;right:0;z-index:10;\
         background:{};flex-wrap:wrap;",
        t.space_2, t.space_3, t.space_4, t.border, reader_bg
    );
    let font_swatch_style = format!(
        "padding:6px 16px;border-radius:{};\
         background:{};\
         border:1px solid {};\
         color:{};font-size:{};\
         cursor:pointer;",
        t.radius_full, t.bg_surface_2, t.border, t.text_secondary, t.font_size_sm
    );
    let theme_panel_style = format!(
        "display:flex;gap:{}px;padding:{}px {}px;\
         border-bottom:1px solid {};\
         position:absolute;top:48px;left:0;right:0;z-index:10;\
         background:{};",
        t.space_2, t.space_3, t.space_4, t.border, reader_bg
    );
    let content_style = format!(
        "flex:1;overflow-y:auto;padding:0 {}px;\
         line-height:1.8;font-size:{};\
         max-width:720px;margin:0 auto;width:100%;\
         box-sizing:border-box;",
        t.space_4, t.font_size_lg
    );
    let chapter_title_style = format!(
        "font-size:{};font-weight:700;margin-bottom:{}px;text-align:center;",
        t.font_size_2xl, t.space_4
    );
    let text_style = "text-align:justify;".to_string();
    let bottom_bar_style = format!(
        "display:flex;align-items:center;justify-content:space-between;\
         padding:12px {}px;background:{};\
         border-top:1px solid {};\
         position:absolute;bottom:0;left:0;right:0;z-index:10;",
        t.space_4, reader_bg, t.border
    );
    let chapter_info_style = format!(
        "font-size:{};color:{};",
        t.font_size_sm, t.text_dim
    );

    let show_menu_val = show_menu.read();
    let show_menu_bool = *show_menu_val;
    let show_fonts_bool = *show_fonts.read();
    let show_themes_bool = *show_themes.read();

    let font_swatches: Vec<Element> = ["System", "Serif", "Sans-serif", "Dyslexic"].iter().map(|f| {
        rsx! {
            div {
                style: "{font_swatch_style}",
                onclick: move |e| e.stop_propagation(),
                "{f}"
            }
        }
    }).collect();

    rsx! {
        div { style: "{body_style}",
            onclick: move |_| {
                let cur = *show_menu.read();
                show_menu.set(!cur);
            },

            if show_menu_bool {
                div { style: "{top_bar_style}",
                    div {
                        style: "{back_btn_style}",
                        onclick: move |e| { e.stop_propagation(); nav.write().back(); },
                        IconBack { size: 20 }
                        "Back"
                    }
                    div { style: "{chapter_label_style}",
                        "Chapter Title"
                    }
                    div { style: "{icon_row_style}",
                        div {
                            style: "{icon_btn_style}",
                            onclick: move |e| { e.stop_propagation(); let v = *show_fonts.read(); show_fonts.set(!v); },
                            IconAa { size: 20 }
                        }
                        div {
                            style: "{icon_btn_style}",
                            onclick: move |e| { e.stop_propagation(); let v = *show_themes.read(); show_themes.set(!v); },
                            IconBookmark { size: 20 }
                        }
                    }
                }
            }

            if show_fonts_bool {
                div { style: "{font_panel_style}",
                    {font_swatches.into_iter()}
                }
            }

            if show_themes_bool {
                div { style: "{theme_panel_style}",
                    ReaderThemeSwatch { bg: "#12161c", fg: "#d8dee6", label: "Dark" }
                    ReaderThemeSwatch { bg: "#faf8f3", fg: "#2b3138", label: "Light" }
                    ReaderThemeSwatch { bg: "#1a2b1a", fg: "#c8dcc8", label: "Green" }
                    ReaderThemeSwatch { bg: "#1a1a2e", fg: "#d8c8e8", label: "Sepia" }
                }
            }

            div { style: "{content_style}",
                div { style: "{chapter_title_style}",
                    "Chapter 1: The Beginning"
                }
                div { style: "{text_style}",
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                     Sed do eiusmod tempor incididunt ut labore et dolore magna \
                     aliqua. Ut enim ad minim veniam, quis nostrud exercitation \
                     ullamco laboris nisi ut aliquip ex ea commodo consequat.\n\n\
                     Duis aute irure dolor in reprehenderit in voluptate velit \
                     esse cillum dolore eu fugiat nulla pariatur. Excepteur sint \
                     occaecat cupidatat non proident, sunt in culpa qui officia \
                     deserunt mollit anim id est laborum.\n\n\
                     Sed ut perspiciatis unde omnis iste natus error sit \
                     voluptatem accusantium doloremque laudantium, totam rem \
                     aperiam, eaque ipsa quae ab illo inventore veritatis et \
                     quasi architecto beatae vitae dicta sunt explicabo.\n\n\
                     Nemo enim ipsam voluptatem quia voluptas sit aspernatur \
                     aut odit aut fugit, sed quia consequuntur magni dolores \
                     eos qui ratione voluptatem sequi nesciunt."
                }
            }

            if show_menu_bool {
                div { style: "{bottom_bar_style}",
                    IconPrev { size: 20 }
                    div { style: "{chapter_info_style}",
                        "Chapter 1 of 42"
                    }
                    IconNext { size: 20 }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ReaderThemeSwatchProps {
    bg: &'static str,
    fg: &'static str,
    label: &'static str,
}

#[component]
fn ReaderThemeSwatch(props: ReaderThemeSwatchProps) -> Element {
    let outer_style = "display:flex;flex-direction:column;align-items:center;gap:4px;cursor:pointer;".to_string();
    let circle_style = format!(
        "width:48px;height:48px;border-radius:50%;background:{};border:2px solid #ffffff40;",
        props.bg
    );
    let label_style = format!("font-size:10px;color:{};", props.fg);

    rsx! {
        div { style: "{outer_style}",
            onclick: move |e| e.stop_propagation(),
            div { style: "{circle_style}",
            }
            div { style: "{label_style}",
                "{props.label}"
            }
        }
    }
}
