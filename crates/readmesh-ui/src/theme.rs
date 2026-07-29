//! ReadMesh design system: colors, typography and spacing, layered on top of
//! Makepad's theme system (`mod.themes`).
//!
//! Two themes are defined — `readmesh_dark` (default, reader-friendly) and
//! `readmesh_light` (warm paper) — by spreading the base Makepad themes and
//! overriding the tokens that carry the ReadMesh identity. The active theme
//! is selected via `mod.theme` before widgets are registered, and can be
//! switched at runtime from the Settings screen.

use makepad_widgets::*;

script_mod! {
    // ReadMesh brand palette (also usable directly in templates via rm.*).
    mod.rm = {
        // Dark theme palette
        dark_bg: #x0f1319
        dark_surface: #x171d26
        dark_surface_2: #x1e2632
        dark_border: #x2a3442
        dark_text: #xe8ecf1
        dark_text_dim: #x96a1b0
        // Light theme palette
        light_bg: #xf6f3ec
        light_surface: #xffffff
        light_surface_2: #xefeae0
        light_border: #xddd5c6
        light_text: #x232a33
        light_text_dim: #x6b7280
        // Shared brand accents
        accent: #xe8a33d
        accent_dim: #x9a6b24
        accent_soft: #xe8a33d
        color_ok: #x4caf7d
        err: #xe05d5d
        // Reader themes (reader screen backgrounds)
        reader_dark_bg: #x12161c
        reader_dark_text: #xd8dee6
        reader_light_bg: #xfaf8f3
        reader_light_text: #x2b3138
        reader_sepia_bg: #xf3e9d7
        reader_sepia_text: #x4a3f2f
    }

    // Dark is the default ReadMesh theme: deep ink surfaces, amber accent.
    mod.themes.readmesh_dark = {
        ..mod.themes.dark

        color_bg_app: mod.rm.dark_bg
        color_fg_app: mod.rm.dark_surface
        color_bg_container: #x1e2632cc
        color_bg_even: mod.rm.dark_surface
        color_bg_odd: mod.rm.dark_surface_2
        color_bg_highlight: #xe8a33d22
        color_app_caption_bar: mod.rm.dark_bg

        color_highlight: mod.rm.accent
        color_selection_focus: #xe8a33d55
        color_cursor: mod.rm.accent
        color_cursor_focus: mod.rm.accent

        color_text: mod.rm.dark_text
        color_text_hover: mod.rm.dark_text
        color_text_focus: mod.rm.dark_text
        color_text_placeholder: mod.rm.dark_text_dim
        color_text_meta: mod.rm.dark_text_dim
        color_text_cursor: mod.rm.accent

        color_label_inner: mod.rm.dark_text
        color_label_inner_hover: mod.rm.dark_text
        color_label_inner_down: mod.rm.dark_text
        color_label_inner_focus: mod.rm.dark_text
        color_label_inner_active: mod.rm.accent
        color_label_inner_inactive: mod.rm.dark_text_dim
        color_label_outer: mod.rm.dark_text
        color_label_outer_off: mod.rm.dark_text_dim

        color_outset: mod.rm.dark_surface_2
        color_outset_hover: #x252f3d
        color_outset_down: #x141a22
        color_outset_active: #x3d2f14
        color_outset_focus: #x252f3d

        color_inset: mod.rm.dark_bg
        color_inset_hover: #x131920
        color_inset_focus: #x131920
        color_inset_empty: mod.rm.dark_bg

        color_error: mod.rm.err
        color_warning: mod.rm.accent

        container_corner_radius: 8.0
    }

    // Light theme: warm paper surfaces, deeper amber accent.
    mod.themes.readmesh_light = {
        ..mod.themes.light

        color_bg_app: mod.rm.light_bg
        color_fg_app: mod.rm.light_surface
        color_bg_container: #xffffffcc
        color_bg_even: mod.rm.light_surface
        color_bg_odd: mod.rm.light_surface_2
        color_bg_highlight: #xb07d2a22
        color_app_caption_bar: mod.rm.light_bg

        color_highlight: mod.rm.accent_dim
        color_selection_focus: #xb07d2a44
        color_cursor: mod.rm.accent_dim
        color_cursor_focus: mod.rm.accent_dim

        color_text: mod.rm.light_text
        color_text_hover: mod.rm.light_text
        color_text_focus: mod.rm.light_text
        color_text_placeholder: mod.rm.light_text_dim
        color_text_meta: mod.rm.light_text_dim
        color_text_cursor: mod.rm.accent_dim

        color_label_inner: mod.rm.light_text
        color_label_inner_hover: mod.rm.light_text
        color_label_inner_down: mod.rm.light_text
        color_label_inner_focus: mod.rm.light_text
        color_label_inner_active: mod.rm.accent_dim
        color_label_inner_inactive: mod.rm.light_text_dim
        color_label_outer: mod.rm.light_text
        color_label_outer_off: mod.rm.light_text_dim

        color_outset: mod.rm.light_surface_2
        color_outset_hover: #xe7e0d2
        color_outset_down: #xded5c2
        color_outset_active: #xe8d9b8
        color_outset_focus: #xe7e0d2

        color_inset: mod.rm.light_surface
        color_inset_hover: mod.rm.light_surface
        color_inset_focus: mod.rm.light_surface
        color_inset_empty: mod.rm.light_surface

        color_error: mod.rm.err
        color_warning: mod.rm.accent_dim

        container_corner_radius: 8.0
    }

    // Default to the dark, reader-friendly theme.
    mod.theme = mod.themes.readmesh_dark
}

/// Switch the active theme at runtime (called from the Settings screen).
pub fn apply_theme(cx: &mut Cx, mode: readmesh_app::ThemeMode) {
    match mode {
        readmesh_app::ThemeMode::Dark => {
            script_eval!(cx, {
                mod.theme = mod.themes.readmesh_dark
            });
        }
        readmesh_app::ThemeMode::Light => {
            script_eval!(cx, {
                mod.theme = mod.themes.readmesh_light
            });
        }
    }
}
