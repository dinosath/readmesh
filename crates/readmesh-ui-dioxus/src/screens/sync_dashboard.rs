use dioxus::prelude::*;

use crate::components::*;
use crate::theme::Theme;

#[component]
pub fn SyncDashboardScreen() -> Element {
    let t = use_context::<Signal<Theme>>();

    rsx! {
        div { style: "display:flex;flex-direction:column;padding:16px;max-width:700px;margin:0 auto;width:100%;box-sizing:border-box;",

            div { style: "font-size:24px;font-weight:600;color:#e8ecf1;margin-bottom:16px;",
                "Sync Dashboard"
            }

            div { style: "display:flex;gap:8px;margin-bottom:24px;",
                StatCard { label: "Pending", value: "3" }
                StatCard { label: "Syncing", value: "1" }
                StatCard { label: "Conflicts", value: "0" }
            }

            div { style: "font-size:16px;font-weight:600;color:#e8ecf1;margin-bottom:8px;",
                "Recent Sync Activity"
            }

            div { style: "display:flex;align-items:center;gap:16px;padding:8px 0;border-bottom:1px solid #2a3442;",
                div { style: "flex:1;",
                    div { style: "font-weight:500;color:#e8ecf1;", "The Wandering Inn" }
                    div { style: "font-size:11px;color:#96a1b0;", "Synced 2m ago" }
                }
                IconCheck { size: 16, color: Some("#4caf7d".to_string()) }
            }

            div { style: "display:flex;align-items:center;gap:16px;padding:8px 0;border-bottom:1px solid #2a3442;",
                div { style: "flex:1;",
                    div { style: "font-weight:500;color:#e8ecf1;", "Lord of the Mysteries" }
                    div { style: "font-size:11px;color:#96a1b0;", "Synced 5m ago" }
                }
                IconCheck { size: 16, color: Some("#4caf7d".to_string()) }
            }

            div { style: "display:flex;align-items:center;gap:16px;padding:8px 0;border-bottom:1px solid #2a3442;",
                div { style: "flex:1;",
                    div { style: "font-weight:500;color:#e8ecf1;", "Reverend Insanity" }
                    div { style: "font-size:11px;color:#96a1b0;", "Pending..." }
                }
                IconCheck { size: 16, color: Some("#4caf7d".to_string()) }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct StatCardProps {
    label: &'static str,
    value: &'static str,
}

#[component]
fn StatCard(props: StatCardProps) -> Element {
    let _t = use_context::<Signal<Theme>>();

    rsx! {
        div { style: "flex:1;background:#171d26;border:1px solid #2a3442;border-radius:12px;padding:16px;text-align:center;",
            div { style: "font-size:24px;font-weight:700;color:#e8ecf1;",
                "{props.value}"
            }
            div { style: "font-size:11px;color:#96a1b0;",
                "{props.label}"
            }
        }
    }
}
