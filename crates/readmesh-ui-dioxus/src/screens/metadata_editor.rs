use dioxus::prelude::*;

use readmesh_core::NovelId;

use crate::components::*;

#[component]
pub fn MetadataEditorScreen(novel_id: NovelId) -> Element {
    let _ = novel_id;

    rsx! {
        div { style: "display:flex;flex-direction:column;padding:16px;max-width:700px;margin:0 auto;width:100%;box-sizing:border-box;",

            div { style: "font-size:24px;font-weight:600;color:#e8ecf1;margin-bottom:16px;",
                "Edit Metadata"
            }

            div { style: "margin-bottom:16px;",
                div { style: "font-size:11px;font-weight:500;color:#96a1b0;margin-bottom:6px;", "Title" }
                input { style: "width:100%;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:inherit;outline:none;box-sizing:border-box;", value: "Novel Title" }
            }

            div { style: "margin-bottom:16px;",
                div { style: "font-size:11px;font-weight:500;color:#96a1b0;margin-bottom:6px;", "Author" }
                input { style: "width:100%;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:inherit;outline:none;box-sizing:border-box;", value: "Author Name" }
            }

            div { style: "margin-bottom:16px;",
                div { style: "font-size:11px;font-weight:500;color:#96a1b0;margin-bottom:6px;", "Genre" }
                input { style: "width:100%;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:inherit;outline:none;box-sizing:border-box;", value: "Fantasy" }
            }

            div { style: "margin-bottom:16px;",
                div { style: "font-size:11px;font-weight:500;color:#96a1b0;margin-bottom:6px;", "Status" }
                input { style: "width:100%;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:inherit;outline:none;box-sizing:border-box;", value: "Ongoing" }
            }

            div { style: "margin-bottom:16px;",
                div { style: "font-size:11px;font-weight:500;color:#96a1b0;margin-bottom:6px;", "Tags" }
                input { style: "width:100%;padding:12px;border:1px solid #2a3442;border-radius:8px;background:#171d26;color:#e8ecf1;font-size:14px;font-family:inherit;outline:none;box-sizing:border-box;", value: "magic, adventure" }
            }

            RmPrimaryButton { on_click: move |_| {}, "Save Changes" }
        }
    }
}
