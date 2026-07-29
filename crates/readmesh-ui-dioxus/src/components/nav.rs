use dioxus::prelude::*;

use readmesh_app::navigation::PrimaryTab;

use crate::theme::Theme;
use crate::components::*;

#[derive(Props, Clone, PartialEq)]
pub struct NavRailProps {
    active: PrimaryTab,
    on_tab: EventHandler<PrimaryTab>,
}

#[component]
pub fn NavRail(props: NavRailProps) -> Element {
    let t = use_context::<Signal<Theme>>();
    let nav_style = format!(
        "display:flex;flex-direction:column;width:72px;\
         background:{};border-right:1px solid {};\
         padding-top:16px;align-items:center;",
        t.read().bg_surface, t.read().border,
    );

    let tabs = [
        (PrimaryTab::Library, "Library"),
        (PrimaryTab::Browse, "Discover"),
        (PrimaryTab::Search, "Search"),
        (PrimaryTab::Downloads, "Downloads"),
        (PrimaryTab::Settings, "Settings"),
    ];

    rsx! {
        nav { style: "{nav_style}",
            for (tab, label) in &tabs {
                NavRailItem {
                    active: *tab == props.active,
                    label: label,
                    icon: match tab {
                        PrimaryTab::Library => rsx! { IconBook { size: 22 } },
                        PrimaryTab::Browse => rsx! { IconCompass { size: 22 } },
                        PrimaryTab::Search => rsx! { IconSearch { size: 22 } },
                        PrimaryTab::Downloads => rsx! { IconDownload { size: 22 } },
                        PrimaryTab::Settings => rsx! { IconGear { size: 22 } },
                    },
                    on_click: {
                        let tab = *tab;
                        move |_| props.on_tab.call(tab)
                    },
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct NavRailItemProps {
    active: bool,
    label: &'static str,
    icon: Element,
    on_click: EventHandler<MouseEvent>,
}

#[component]
fn NavRailItem(props: NavRailItemProps) -> Element {
    let t = use_context::<Signal<Theme>>();
    let bg = if props.active { t.read().accent_soft } else { "transparent" };
    let fg = if props.active { t.read().accent } else { t.read().text_secondary };
    let item_style = format!(
        "display:flex;flex-direction:column;align-items:center;justify-content:center;\
         width:72px;height:64px;gap:4px;\
         background:{bg};border-radius:{};\
         cursor:pointer;user-select:none;\
         transition:all 0.15s;",
        t.read().radius_md,
    );

    rsx! {
        div {
            style: "{item_style}",
            onclick: move |e| props.on_click.call(e),
            div { style: "display:flex;color:{fg};", {props.icon} }
            div { style: "font-size:10px;font-weight:500;color:{fg};", "{props.label}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct BottomNavProps {
    active: PrimaryTab,
    on_tab: EventHandler<PrimaryTab>,
}

#[component]
pub fn BottomNav(props: BottomNavProps) -> Element {
    let t = use_context::<Signal<Theme>>();
    let nav_style = format!(
        "display:flex;height:64px;background:{};\
         border-top:1px solid {};\
         align-items:center;justify-content:space-around;",
        t.read().bg_surface, t.read().border,
    );

    rsx! {
        nav { style: "{nav_style}",
            BottomNavTab { active: props.active == PrimaryTab::Library, label: "Library", tab: PrimaryTab::Library, on_tab: props.on_tab }
            BottomNavTab { active: props.active == PrimaryTab::Browse, label: "Discover", tab: PrimaryTab::Browse, on_tab: props.on_tab }
            BottomNavTab { active: props.active == PrimaryTab::Search, label: "Search", tab: PrimaryTab::Search, on_tab: props.on_tab }
            BottomNavTab { active: props.active == PrimaryTab::Downloads, label: "Downloads", tab: PrimaryTab::Downloads, on_tab: props.on_tab }
            BottomNavTab { active: props.active == PrimaryTab::Settings, label: "Settings", tab: PrimaryTab::Settings, on_tab: props.on_tab }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct BottomNavTabProps {
    active: bool,
    label: &'static str,
    tab: PrimaryTab,
    on_tab: EventHandler<PrimaryTab>,
}

#[component]
fn BottomNavTab(props: BottomNavTabProps) -> Element {
    let t = use_context::<Signal<Theme>>();
    let fg = if props.active { t.read().accent } else { t.read().text_dim };

    let icon = match props.tab {
        PrimaryTab::Library => rsx! { IconBook { size: 20 } },
        PrimaryTab::Browse => rsx! { IconCompass { size: 20 } },
        PrimaryTab::Search => rsx! { IconSearch { size: 20 } },
        PrimaryTab::Downloads => rsx! { IconDownload { size: 20 } },
        PrimaryTab::Settings => rsx! { IconGear { size: 20 } },
    };

    rsx! {
        div {
            style: "display:flex;flex-direction:column;align-items:center;gap:2px;\
                    cursor:pointer;user-select:none;padding:8px 16px;color:{fg};",
            onclick: move |_| props.on_tab.call(props.tab),
            {icon}
            div { style: "font-size:10px;font-weight:500;", "{props.label}" }
        }
    }
}
