//! Makepad UI smoke tests (makepad-test).
//!
//! These run the real application with the headless backend and drive it
//! through the Studio remote protocol. Like Makepad's own UI suites, they
//! are currently validated on macOS — CI runs them there. On other platforms
//! the pure-Rust suites (`readmesh-app` unit tests + `tests/script_eval.rs`)
//! provide coverage.

#![cfg(target_os = "macos")]

use makepad_test::{Selector, TestApp, makepad_test};

#[makepad_test]
fn app_launches_into_library(app: TestApp) {
    app.locator(Selector::id("rail_library")).wait_visible();
    app.locator(Selector::all().text_exact("ReadMesh"))
        .wait_visible();
    app.locator(Selector::all().text_exact("Continue Reading"))
        .wait_visible();
}

#[makepad_test]
fn navigation_screens_are_reachable(app: TestApp) {
    app.locator(Selector::id("rail_browse")).click();
    app.locator(Selector::all().text_exact("Trending"))
        .wait_visible();

    app.locator(Selector::id("rail_downloads")).click();
    app.locator(Selector::all().text_exact("Downloads"))
        .wait_visible();

    app.locator(Selector::id("rail_settings")).click();
    app.locator(Selector::all().text_exact("Appearance"))
        .wait_visible();

    app.locator(Selector::id("rail_library")).click();
    app.locator(Selector::all().text_exact("Continue Reading"))
        .wait_visible();
}

#[makepad_test]
fn search_returns_results(app: TestApp) {
    app.locator(Selector::id("rail_search")).click();
    app.locator(Selector::id("search_input"))
        .fill("moon")
        .wait_value("moon");
    app.locator(Selector::id("search_button")).click();
    app.locator(Selector::all().text_exact("Results for “moon”"))
        .wait_visible();
}

#[makepad_test]
fn open_detail_and_chapter_list(app: TestApp) {
    // First card in the library grid opens the detail screen.
    app.locator(Selector::id("card0")).wait_visible().click();
    app.locator(Selector::all().text_exact("Chapters"))
        .wait_visible();
    app.locator(Selector::id("back_button"))
        .wait_visible()
        .click();
    app.locator(Selector::all().text_exact("Continue Reading"))
        .wait_visible();
}
