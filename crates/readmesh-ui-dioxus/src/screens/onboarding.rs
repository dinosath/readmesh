use dioxus::prelude::*;

use readmesh_app::navigation::NavigationState;

use crate::components::*;
use crate::theme::Theme;

#[component]
pub fn OnboardingScreen() -> Element {
    let theme = use_context::<Signal<Theme>>();
    let mut nav = use_context::<Signal<NavigationState>>();

    let t = theme.read();
    let container_style = format!(
        "display:flex;flex-direction:column;align-items:center;\
         justify-content:center;height:100vh;\
         padding:{}px;text-align:center;",
        t.space_5
    );
    let title_style = format!(
        "font-size:{};font-weight:700;color:{};margin-bottom:8px;",
        t.font_size_3xl, t.accent
    );
    let desc_style = format!(
        "font-size:{};color:{};max-width:320px;margin-bottom:{}px;",
        t.font_size_base, t.text_secondary, t.space_5
    );

    rsx! {
        div { style: "{container_style}",
            div { style: "{title_style}",
                "ReadMesh"
            }
            div { style: "{desc_style}",
                "Your modern reading companion. Download, organize, and read your favorite stories — offline-first and peer-to-peer."
            }
            RmPrimaryButton {
                on_click: move |_| nav.write().select_tab(readmesh_app::navigation::PrimaryTab::Library),
                "Get Started"
            }
        }
    }
}
