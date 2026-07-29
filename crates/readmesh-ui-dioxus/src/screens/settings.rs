use dioxus::prelude::*;

use crate::theme::Theme;

#[component]
pub fn SettingsScreen() -> Element {
    let theme = use_context::<Signal<Theme>>();

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
    let theme_toggle_style = format!("display:flex;gap:{}px;", t.space_2);
    let reader_font_style = format!("color:{};font-size:{};", t.text_secondary, t.font_size_base);
    let font_size_slider_style = format!("display:flex;align-items:center;gap:{}px;", t.space_2);
    let size_sm_style = format!("font-size:{};", t.font_size_sm);
    let slider_bg_style = format!(
        "width:120px;height:6px;border-radius:3px;\
         background:{};position:relative;",
        t.border
    );
    let slider_fill_style = format!(
        "width:60%;height:100%;border-radius:3px;\
         background:{};",
        t.accent
    );
    let size_xl_style = format!("font-size:{};font-weight:600;", t.font_size_xl);

    rsx! {
        div { style: "{container_style}",

            div { style: "{page_title_style}",
                "Settings"
            }

            SettingsSection { title: "Appearance",
                SettingsRow { label: "Theme",
                    div { style: "{theme_toggle_style}",
                        ThemeButton { label: "Dark", active: true }
                        ThemeButton { label: "Light", active: false }
                        ThemeButton { label: "System", active: false }
                    }
                }
                SettingsRow { label: "Reader Font",
                    div { style: "{reader_font_style}",
                        "System Default"
                    }
                }
                SettingsRow { label: "Reader Text Size",
                    div { style: "{font_size_slider_style}",
                        div { style: "{size_sm_style}", "A" }
                        div { style: "{slider_bg_style}",
                            div { style: "{slider_fill_style}",
                            }
                        }
                        div { style: "{size_xl_style}", "A" }
                    }
                }
            }

            SettingsSection { title: "Reading",
                SettingsRow { label: "Auto-scroll Speed", value: Some("Off".to_string()), children: None }
                SettingsRow { label: "Tap to Scroll", value: Some("On".to_string()), children: None }
                SettingsRow { label: "Keep Screen On", value: Some("On".to_string()), children: None }
            }

            SettingsSection { title: "Library",
                SettingsRow { label: "Sort By", value: Some("Last Opened".to_string()), children: None }
                SettingsRow { label: "Show Progress", value: Some("On".to_string()), children: None }
            }

            SettingsSection { title: "Sync",
                SettingsRow { label: "Sync Server", value: Some("Not Configured".to_string()), children: None }
                SettingsRow { label: "P2P Discovery", value: Some("Enabled".to_string()), children: None }
            }

            SettingsSection { title: "About",
                SettingsRow { label: "Version", value: Some("0.1.0".to_string()), children: None }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SettingsSectionProps {
    title: String,
    children: Element,
}

#[component]
fn SettingsSection(props: SettingsSectionProps) -> Element {
    let theme = use_context::<Signal<Theme>>();

    let t = theme.read();
    let section_style = format!("margin-bottom:{}px;", t.space_4);
    let section_title_style = format!(
        "font-size:{};font-weight:600;color:{};\
         text-transform:uppercase;letter-spacing:0.5px;\
         margin-bottom:{}px;",
        t.font_size_sm, t.text_dim, t.space_2
    );
    let section_body_style = format!(
        "background:{};border:1px solid {};\
         border-radius:{};overflow:hidden;",
        t.bg_surface, t.border, t.radius_lg
    );

    rsx! {
        div { style: "{section_style}",
            div { style: "{section_title_style}",
                "{props.title}"
            }
            div { style: "{section_body_style}",
                {&props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SettingsRowProps {
    label: String,
    value: Option<String>,
    children: Option<Element>,
}

#[component]
fn SettingsRow(props: SettingsRowProps) -> Element {
    let theme = use_context::<Signal<Theme>>();

    let t = theme.read();
    let value_style = format!("color:{};font-size:{};", t.text_secondary, t.font_size_base);
    let row_style = format!(
        "display:flex;align-items:center;justify-content:space-between;\
         padding:{}px;border-bottom:1px solid {};",
        t.space_3, t.border
    );
    let label_style = format!(
        "font-size:{};font-weight:500;color:{};",
        t.font_size_base, t.text_primary
    );

    let content = if let Some(val) = &props.value {
        rsx! { div { style: "{value_style}", "{val}" } }
    } else {
        rsx! { {props.children.unwrap_or(rsx! {})} }
    };

    rsx! {
        div { style: "{row_style}",
            div { style: "{label_style}",
                "{props.label}"
            }
            {content}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ThemeButtonProps {
    label: String,
    active: bool,
}

#[component]
fn ThemeButton(props: ThemeButtonProps) -> Element {
    let theme = use_context::<Signal<Theme>>();

    let t = theme.read();
    let bg = if props.active { t.accent } else { "transparent" };
    let fg = if props.active { t.text_on_accent } else { t.text_secondary };
    let border = if props.active { t.accent } else { t.border };
    let btn_style = format!(
        "padding:6px 16px;border-radius:{};\
         background:{};color:{};\
         border:1px solid {};\
         font-size:{};font-weight:500;\
         cursor:pointer;user-select:none;",
        t.radius_full, bg, fg, border, t.font_size_sm
    );

    rsx! {
        div { style: "{btn_style}",
            "{props.label}"
        }
    }
}
