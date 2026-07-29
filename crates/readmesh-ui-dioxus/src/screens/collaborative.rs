use dioxus::prelude::*;

use readmesh_core::NovelId;

#[component]
pub fn CollaborativeScreen(novel_id: NovelId) -> Element {
    let _ = novel_id;

    rsx! {
        div { style: "display:flex;flex-direction:column;align-items:center;justify-content:center;height:100%;padding:24px;color:#96a1b0;gap:12px;text-align:center;",
            div { style: "font-size:48px;", "\u{1F465}" }
            div { style: "font-size:18px;font-weight:600;color:#e8ecf1;",
                "Collaborative Workspace"
            }
            div { style: "font-size:14px;max-width:400px;color:#c8d0dc;",
                "Invite co-authors to write together in real-time."
            }
        }
    }
}
