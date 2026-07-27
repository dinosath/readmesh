// This stub is necessary because some platforms require building
// as a shared library (mobile) and some as an executable (desktop).
// Cargo doesn't facilitate both without a main.rs stub.

// Hide the console window on Windows release builds.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    readmesh_ui::app::app_main();
}
