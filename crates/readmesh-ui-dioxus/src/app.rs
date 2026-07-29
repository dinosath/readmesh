use dioxus::prelude::*;

use readmesh_app::navigation::{NavigationState, PrimaryTab, Route};

use crate::components::*;
use crate::screens::*;
use crate::theme::Theme;

#[allow(non_snake_case)]
pub fn App() -> Element {
    let theme = use_signal(Theme::dark);
    let nav = use_signal(NavigationState::new);

    use_context_provider(|| theme);
    use_context_provider(|| nav);

    let mut nav_clone = nav;
    let current = nav_clone.read().current();
    let chrome_hidden = nav_clone.read().chrome_hidden();
    let use_bottom = nav_clone.read().mode().uses_bottom_nav();

    let style = format!(
        "display:flex;width:100%;height:100vh;\
         background:{};color:{};\
         font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Oxygen,Ubuntu,sans-serif;\
         margin:0;overflow:hidden;",
        theme.read().bg_app,
        theme.read().text_primary,
    );

    rsx! {
        div { style: "{style}",
            if !chrome_hidden && !use_bottom {
                NavRail {
                    active: nav_clone.read().current_tab(),
                    on_tab: move |tab| { nav_clone.write().select_tab(tab); },
                }
            }
            div { style: "display:flex;flex-direction:column;flex:1;overflow:hidden;",
                if !chrome_hidden && nav_clone.read().can_go_back() {
                    BackBar {
                        on_back: move |_| { nav_clone.write().back(); }
                    }
                }
                div { style: "flex:1;overflow-y:auto;",
                    {render_route(&current, &nav_clone)}
                }
            }
            if !chrome_hidden && use_bottom {
                BottomNav {
                    active: nav_clone.read().current_tab(),
                    on_tab: move |tab| { nav_clone.write().select_tab(tab); },
                }
            }
        }
    }
}

fn render_route(route: &Route, nav: &Signal<NavigationState>) -> Element {
    match route {
        Route::Onboarding => rsx! { OnboardingScreen {} },
        Route::Tab(PrimaryTab::Library) | Route::CreateNovel => rsx! { LibraryScreen {} },
        Route::Tab(PrimaryTab::Browse) => rsx! { BrowseScreen {} },
        Route::Tab(PrimaryTab::Search) => rsx! { SearchScreen {} },
        Route::Tab(PrimaryTab::Downloads) => rsx! { DownloadsScreen {} },
        Route::Tab(PrimaryTab::Settings) => rsx! { SettingsScreen {} },
        Route::NovelDetail(id) => {
            let id = *id;
            rsx! { NovelDetailScreen { novel_id: id } }
        }
        Route::Reader { novel, chapter } => {
            let novel = *novel;
            let chapter = *chapter;
            rsx! { ReaderScreen { novel_id: novel, chapter_id: chapter } }
        }
        Route::MetadataEditor(id) => {
            let id = *id;
            rsx! { MetadataEditorScreen { novel_id: id } }
        }
        Route::CoverStudio(id) => {
            let id = *id;
            rsx! { CoverStudioScreen { novel_id: id } }
        }
        Route::ImportFromWebsite => rsx! { ImportScreen {} },
        Route::ChapterEditor { novel, chapter } => {
            let novel = *novel;
            let chapter = *chapter;
            rsx! { ChapterEditorScreen { novel_id: novel, chapter_id: chapter } }
        }
        Route::CollaborativeWorkspace(id) => {
            let id = *id;
            rsx! { CollaborativeScreen { novel_id: id } }
        }
        Route::PeerConnections => rsx! { PeerConnectionsScreen {} },
        Route::SyncDashboard => rsx! { SyncDashboardScreen {} },
    }
}

#[derive(Props, Clone, PartialEq)]
struct BackBarProps {
    on_back: EventHandler<MouseEvent>,
}

#[component]
fn BackBar(props: BackBarProps) -> Element {
    let t = use_context::<Signal<Theme>>();
    let style = format!(
        "display:flex;align-items:center;height:48px;\
         padding:0 8px;border-bottom:1px solid {};background:{};",
        t.read().border, t.read().bg_surface,
    );
    let btn_style = format!(
        "display:flex;align-items:center;gap:4px;cursor:pointer;color:{};\
         font-size:{};padding:8px 12px;border-radius:{};user-select:none;",
        t.read().text_secondary, t.read().font_size_base, t.read().radius_sm,
    );

    rsx! {
        div { style: "{style}",
            div {
                style: "{btn_style}",
                onclick: move |e| props.on_back.call(e),
                IconBack { size: 18 }
                "Back"
            }
        }
    }
}
