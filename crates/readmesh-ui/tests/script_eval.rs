//! Headless validation of the app's Splash scripts.
//!
//! `Cx::new` initializes the full script VM without a display, so evaluating
//! every `script_mod!` here catches Splash syntax/registration errors in
//! plain `cargo test` on any platform (including headless Linux CI).

use readmesh_ui::app::App;
use readmesh_ui::makepad_widgets::*;

/// Evaluate the complete app script stack (themes, widgets, components,
/// screens, shell, app UI) and return the resulting app object.
fn evaluate_app() -> App {
    let mut cx = Cx::new(Box::new(|_, _| {}));
    cx.with_vm(|vm| {
        let value = <App as AppMain>::script_mod(vm);
        App::script_from_value(vm, value)
    })
}

fn assert_path_registered(vm: &ScriptVm, path: &[LiveId], what: &str) {
    let heap = vm.heap();
    let value = heap.value_path(heap.modules, path, NoTrap);
    assert!(
        !value.is_nil(),
        "expected {what} to be registered in the script module tree"
    );
}

#[test]
fn app_scripts_evaluate_and_register_widgets() {
    let mut cx = Cx::new(Box::new(|_, _| {}));
    cx.with_vm(|vm| {
        <App as AppMain>::script_mod(vm);

        // ReadMesh themes.
        assert_path_registered(
            vm,
            ids!(themes.readmesh_dark).as_slice(),
            "readmesh_dark theme",
        );
        assert_path_registered(
            vm,
            ids!(themes.readmesh_light).as_slice(),
            "readmesh_light theme",
        );

        // Shared component templates.
        for (path, what) in [
            (ids!(widgets.RmCard), "RmCard"),
            (ids!(widgets.RmCardRow2), "RmCardRow2"),
            (ids!(widgets.RmCardRow3), "RmCardRow3"),
            (ids!(widgets.RmCardRow4), "RmCardRow4"),
            (ids!(widgets.RmChapterRow), "RmChapterRow"),
            (ids!(widgets.RmDownloadRow), "RmDownloadRow"),
            (ids!(widgets.RmEmptyState), "RmEmptyState"),
            (ids!(widgets.RmLoadingState), "RmLoadingState"),
            (ids!(widgets.RmErrorState), "RmErrorState"),
            (ids!(widgets.RmSectionHeader), "RmSectionHeader"),
            (ids!(widgets.RmTagChip), "RmTagChip"),
            (ids!(widgets.RmSettingsToggleRow), "RmSettingsToggleRow"),
            (ids!(widgets.RmSettingsActionRow), "RmSettingsActionRow"),
            (ids!(widgets.RmProgressBar), "RmProgressBar"),
            (ids!(widgets.RmPrimaryButton), "RmPrimaryButton"),
            (ids!(widgets.RmTextInput), "RmTextInput"),
        ] {
            assert_path_registered(vm, path.as_slice(), what);
        }
    });
}

#[test]
fn app_object_constructs() {
    let _app = evaluate_app();
}
