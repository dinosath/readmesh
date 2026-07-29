use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    size: Option<i32>,
    color: Option<String>,
}

fn svg_icon(size: i32, color: &str, children: Element) -> Element {
    let sz = format!("{}", size);
    rsx! {
        svg {
            width: "{sz}",
            height: "{sz}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{color}",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            {children}
        }
    }
}

#[component]
pub fn IconBook(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        path { d: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20" }
        path { d: "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" }
    })
}

#[component]
pub fn IconBookOpen(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        path { d: "M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" }
        path { d: "M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" }
    })
}

#[component]
pub fn IconCompass(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        circle { cx: "12", cy: "12", r: "10" }
        path { d: "M16.24 7.76l-2.12 6.36-6.36 2.12 2.12-6.36z" }
    })
}

#[component]
pub fn IconSearch(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        circle { cx: "11", cy: "11", r: "8" }
        path { d: "M21 21l-4.35-4.35" }
    })
}

#[component]
pub fn IconDownload(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
        polyline { points: "7 10 12 15 17 10" }
        line { x1: "12", y1: "15", x2: "12", y2: "3" }
    })
}

#[component]
pub fn IconGear(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        circle { cx: "12", cy: "12", r: "3" }
        path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
    })
}

#[component]
pub fn IconBack(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        polyline { points: "15 18 9 12 15 6" }
    })
}

#[component]
pub fn IconClose(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        line { x1: "18", y1: "6", x2: "6", y2: "18" }
        line { x1: "6", y1: "6", x2: "18", y2: "18" }
    })
}

#[component]
pub fn IconStar(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        polygon { points: "12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" }
    })
}

#[component]
pub fn IconRetry(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        polyline { points: "23 4 23 10 17 10" }
        polyline { points: "1 20 1 14 7 14" }
        path { d: "M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" }
    })
}

#[component]
pub fn IconTrash(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        polyline { points: "3 6 5 6 21 6" }
        path { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
    })
}

#[component]
pub fn IconAlert(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        path { d: "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" }
        line { x1: "12", y1: "9", x2: "12", y2: "13" }
        line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
    })
}

#[component]
pub fn IconBookmark(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        path { d: "M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" }
    })
}

#[component]
pub fn IconPrev(props: IconProps) -> Element {
    rsx! { IconBack { size: props.size, color: props.color } }
}

#[component]
pub fn IconNext(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        polyline { points: "9 18 15 12 9 6" }
    })
}

#[component]
pub fn IconAa(props: IconProps) -> Element {
    let s = props.size.unwrap_or(20);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        path { d: "M11 4H4M18 4h-3M4 20h4M14 20h6M4 12h16M6 4l-2 16M18 4l2 16" }
    })
}

#[component]
pub fn IconCheck(props: IconProps) -> Element {
    let s = props.size.unwrap_or(16);
    let c = props.color.unwrap_or_else(|| "currentColor".into());
    svg_icon(s, &c, rsx! {
        polyline { points: "20 6 9 17 4 12" }
    })
}
