use dioxus::prelude::*;

use crate::components::*;

#[component]
pub fn PeerConnectionsScreen() -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;padding:16px;max-width:700px;margin:0 auto;width:100%;box-sizing:border-box;",

            div { style: "font-size:24px;font-weight:600;color:#e8ecf1;margin-bottom:16px;",
                "Peer Connections"
            }

            div { style: "display:flex;align-items:center;gap:16px;background:#171d26;border:1px solid #2a3442;border-radius:12px;padding:16px;margin-bottom:16px;",
                IconRetry { size: 18, color: Some("#4caf7d".to_string()) }
                div { style: "flex:1;",
                    div { style: "font-weight:600;color:#e8ecf1;", "Local Network" }
                    div { style: "font-size:11px;color:#96a1b0;font-family:monospace;", "192.168.1.42" }
                }
                div { style: "font-size:11px;padding:2px 8px;border-radius:9999px;background:#4caf7d;color:#0f1319;",
                    "Online"
                }
            }

            div { style: "display:flex;align-items:center;gap:16px;background:#171d26;border:1px solid #2a3442;border-radius:12px;padding:16px;",
                IconRetry { size: 18, color: Some("#96a1b0".to_string()) }
                div { style: "flex:1;",
                    div { style: "font-weight:600;color:#e8ecf1;", "Remote Node" }
                    div { style: "font-size:11px;color:#96a1b0;font-family:monospace;", "10.0.0.7:8432" }
                }
                div { style: "font-size:11px;padding:2px 8px;border-radius:9999px;background:#96a1b0;color:#fff;",
                    "Offline"
                }
            }
        }
    }
}
