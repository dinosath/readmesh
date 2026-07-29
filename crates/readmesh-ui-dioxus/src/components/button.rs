use dioxus::prelude::*;

use crate::theme::Theme;

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    children: Element,
    on_click: Option<EventHandler<MouseEvent>>,
    disabled: Option<bool>,
    full_width: Option<bool>,
}

#[component]
pub fn RmPrimaryButton(props: ButtonProps) -> Element {
    let theme = use_context::<Signal<Theme>>();
    let disabled = props.disabled.unwrap_or(false);

    let style = format!(
        "display:flex;align-items:center;justify-content:center;\
         width:{};height:48px;padding:0 24px;\
         background:{};color:{};\
         border:none;border-radius:{};\
         font-size:{};font-weight:600;\
         cursor:{};opacity:{};\
         transition:opacity 0.15s;",
        if props.full_width.unwrap_or(true) { "100%" } else { "fit-content" },
        theme.read().accent,
        theme.read().text_on_accent,
        theme.read().radius_md,
        theme.read().font_size_lg,
        if disabled { "default" } else { "pointer" },
        if disabled { "0.5" } else { "1" },
    );

    rsx! {
        button {
            style: "{style}",
            onclick: move |e| if let Some(cb) = props.on_click.as_ref() { cb(e) },
            {&props.children}
        }
    }
}

#[component]
pub fn RmSecondaryButton(props: ButtonProps) -> Element {
    let theme = use_context::<Signal<Theme>>();

    let style = format!(
        "display:flex;align-items:center;justify-content:center;\
         width:{};height:40px;padding:0 20px;\
         background:transparent;color:{};\
         border:1px solid {};border-radius:{};\
         font-size:{};font-weight:500;\
         cursor:pointer;transition:all 0.15s;",
        if props.full_width.unwrap_or(false) { "100%" } else { "fit-content" },
        theme.read().text_primary,
        theme.read().border,
        theme.read().radius_md,
        theme.read().font_size_base,
    );

    rsx! {
        button {
            style: "{style}",
            onclick: move |e| if let Some(cb) = props.on_click.as_ref() { cb(e) },
            {&props.children}
        }
    }
}

#[component]
pub fn RmSmallButton(props: ButtonProps) -> Element {
    let theme = use_context::<Signal<Theme>>();

    let style = format!(
        "display:flex;align-items:center;justify-content:center;\
         gap:4px;width:fit-content;height:32px;padding:0 12px;\
         background:{};color:{};\
         border:1px solid {};border-radius:{};\
         font-size:{};font-weight:500;\
         cursor:pointer;transition:all 0.15s;",
        theme.read().bg_surface_2,
        theme.read().text_primary,
        theme.read().border,
        theme.read().radius_sm,
        theme.read().font_size_sm,
    );

    rsx! {
        button {
            style: "{style}",
            onclick: move |e| if let Some(cb) = props.on_click.as_ref() { cb(e) },
            {&props.children}
        }
    }
}
