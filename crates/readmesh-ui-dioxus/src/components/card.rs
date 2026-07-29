use dioxus::prelude::*;

use crate::theme::Theme;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    children: Element,
    on_click: Option<EventHandler<MouseEvent>>,
    padding: Option<String>,
}

#[component]
pub fn RmCard(props: CardProps) -> Element {
    let theme = use_context::<Signal<Theme>>();
    let pad = props.padding.unwrap_or_else(|| "24px".into());

    let style = format!(
        "background:{};border:1px solid {};border-radius:{};padding:{};",
        theme.read().bg_surface,
        theme.read().border,
        theme.read().radius_lg,
        pad,
    );

    if let Some(cb) = props.on_click {
        rsx! {
            div {
                style: "{style}cursor:pointer;transition:border-color 0.15s;",
                onclick: move |e| cb(e),
                {&props.children}
            }
        }
    } else {
        rsx! { div { style: "{style}", {&props.children} } }
    }
}
