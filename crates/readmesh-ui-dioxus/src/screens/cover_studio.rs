use dioxus::prelude::*;

use readmesh_core::NovelId;

use crate::components::*;

#[component]
pub fn CoverStudioScreen(novel_id: NovelId) -> Element {
    let _ = novel_id;

    rsx! {
        div { style: "display:flex;flex-direction:column;align-items:center;padding:16px;max-width:700px;margin:0 auto;width:100%;box-sizing:border-box;",

            div { style: "font-size:24px;font-weight:600;color:#e8ecf1;margin-bottom:16px;",
                "Cover Studio"
            }

            div { style: "width:200px;height:300px;border-radius:12px;background:#e8a33d33;margin-bottom:24px;display:flex;align-items:center;justify-content:center;font-size:64px;",
                "\u{1F4D6}"
            }

            div { style: "display:flex;flex-wrap:wrap;gap:8px;margin-bottom:24px;",
                RmSmallButton { "Upload Image" }
                RmSmallButton { "Use AI Generate" }
                RmSmallButton { "Pick Color" }
                RmSmallButton { "Add Title Text" }
            }

            RmPrimaryButton { full_width: true, "Save Cover" }
        }
    }
}
