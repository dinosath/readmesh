use dioxus::prelude::*;

use readmesh_core::{ChapterId, NovelId};

use crate::components::*;

#[component]
pub fn ChapterEditorScreen(novel_id: NovelId, chapter_id: ChapterId) -> Element {
    let _ = (novel_id, chapter_id);

    rsx! {
        div { style: "display:flex;flex-direction:column;height:100vh;background:#12161c;",

            div { style: "display:flex;align-items:center;justify-content:space-between;padding:12px 24px;border-bottom:1px solid #2a3442;background:#171d26;",
                IconBack { size: 20 }
                div { style: "font-size:14px;font-weight:600;color:#e8ecf1;",
                    "Chapter Editor"
                }
                RmSmallButton { "Save" }
            }

            textarea {
                style: "flex:1;padding:24px;border:none;outline:none;background:transparent;color:#d8dee6;font-size:16px;line-height:1.8;font-family:monospace;resize:none;box-sizing:border-box;",
                placeholder: "Write your chapter content here...",
            }
        }
    }
}
