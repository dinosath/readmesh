use dioxus::prelude::*;

use crate::components::*;
use crate::theme::Theme;

#[component]
pub fn ImportScreen() -> Element {
    let _t = use_context::<Signal<Theme>>();

    rsx! {
        div { style: "display:flex;flex-direction:column;padding:16px;max-width:700px;margin:0 auto;width:100%;box-sizing:border-box;",

            div { style: "font-size:24px;font-weight:600;color:#e8ecf1;margin-bottom:16px;",
                "Import from Website"
            }

            div { style: "display:flex;align-items:center;gap:8px;background:#171d26;border:1px solid #2a3442;border-radius:8px;padding:8px 12px;margin-bottom:24px;",
                input {
                    style: "flex:1;border:none;outline:none;background:transparent;color:#e8ecf1;font-size:14px;font-family:inherit;",
                    placeholder: "Enter URL to import from...",
                    value: "",
                }
                RmSmallButton { "Import" }
            }

            div { style: "display:flex;flex-direction:column;align-items:center;padding:32px 0;color:#96a1b0;gap:8px;",
                div { style: "font-size:14px;color:#c8d0dc;", "Or paste HTML content" }
                textarea {
                    style: "width:100%;height:200px;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:monospace;outline:none;resize:vertical;box-sizing:border-box;",
                    placeholder: "Paste HTML content here...",
                }
            }
        }
    }
}
