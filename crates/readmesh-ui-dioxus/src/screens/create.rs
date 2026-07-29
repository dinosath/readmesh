use dioxus::prelude::*;

use crate::components::*;
use crate::theme::Theme;

#[component]
pub fn CreateNovelScreen() -> Element {
    let _t = use_context::<Signal<Theme>>();
    rsx! {
        div { style: "display:flex;flex-direction:column;padding:16px;max-width:700px;margin:0 auto;width:100%;box-sizing:border-box;",

            div { style: "font-size:24px;font-weight:600;color:#e8ecf1;margin-bottom:16px;",
                "Create Novel"
            }

            div { style: "margin-bottom:16px;",
                div { style: "font-size:11px;font-weight:500;color:#96a1b0;margin-bottom:6px;",
                    "Title"
                }
                input {
                    style: "width:100%;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:inherit;outline:none;box-sizing:border-box;",
                    placeholder: "Enter novel title...",
                }
            }

            div { style: "margin-bottom:16px;",
                div { style: "font-size:11px;font-weight:500;color:#96a1b0;margin-bottom:6px;",
                    "Author"
                }
                input {
                    style: "width:100%;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:inherit;outline:none;box-sizing:border-box;",
                    placeholder: "Enter author name...",
                }
            }

            div { style: "margin-bottom:16px;",
                div { style: "font-size:11px;font-weight:500;color:#96a1b0;margin-bottom:6px;",
                    "Description"
                }
                textarea {
                    style: "width:100%;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:inherit;outline:none;box-sizing:border-box;height:120px;resize:vertical;",
                    placeholder: "Enter a description...",
                }
            }

            RmPrimaryButton {
                on_click: move |_| {},
                "Create Novel"
            }
        }
    }
}
