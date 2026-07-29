use readmesh_ui_dioxus::app::App;

#[cfg(feature = "desktop")]
fn main() {
    dioxus::LaunchBuilder::desktop().launch(App);
}

#[cfg(not(feature = "desktop"))]
fn main() {
    // Web entry point is handled by dx serve; this binary is not used.
}
