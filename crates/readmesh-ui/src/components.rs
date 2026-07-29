//! Reusable ReadMesh components.
//!
//! Rust side: the `RmProgressBar` custom widget.
//! Splash side (`script_mod!`): shared templates registered into
//! `mod.widgets.*` so every screen module can use them via
//! `use mod.widgets.*` (per the Makepad 2.0 DSL skill's cross-module
//! sharing rules).

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // ---- custom Rust widgets ------------------------------------------------

    mod.widgets.RmProgressBarBase = #(RmProgressBar::register_widget(vm))
    mod.widgets.RmProgressBar = set_type_default() do mod.widgets.RmProgressBarBase{
        width: Fill
        height: 5
        draw_track +: {
            border_radius: 2.5
            color: theme.color_bg_highlight
        }
        draw_fill +: {
            border_radius: 2.5
            color: theme.color_highlight
        }
    }

    mod.widgets.RmVerticalSlider = #(RmVerticalSlider::register_widget(vm)){
        width: 12 height: Fill
        draw_track +: {
            color: #xffffff11
            border_radius: 6.0
        }
        draw_thumb +: {
            color: theme.color_highlight
            border_radius: 4.0
        }
        value: 0.0
        min_val: 0.0
        max_val: 1.0
    }

    // A clickable view: emits RmTapAction::Clicked on tap. Used as the root
    // of cards, chapter rows and other tappable list items. Children with
    // their own hit handling (checkboxes, buttons) still work — a captured
    // child area suppresses the parent's tap.
    mod.widgets.RmTap = #(RmTap::register_widget(vm)){
        width: Fill
        height: Fit
        flow: Down
        cursor: MouseCursor.Hand
        show_bg: true
        draw_bg +: {
            color: #0000
            color_hover: theme.color_bg_highlight
        }
    }

    // ---- icons (inline vector graphics, todo-example pattern) -----------------

    mod.widgets.IconBook = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M4 19.5A2.5 2.5 0 0 1 6.5 17H20" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconBookOpen = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconCompass = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M16.24 7.76l-2.12 6.36-6.36 2.12 2.12-6.36z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconSearch = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M21 21l-4.35-4.35" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconDownload = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconGear = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconBack = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M15 18l-6-6 6-6" fill: false stroke: theme.color_label_inner stroke_width: 2.2 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconClose = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M18 6L6 18M6 6l12 12" fill: false stroke: theme.color_label_inner stroke_width: 2.2 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconCheck = Vector{width: 16 height: 16 viewbox: vec4(0 0 24 24)
        Path{d: "M20 6L9 17l-5-5" fill: false stroke: mod.rm.color_ok stroke_width: 2.5 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconStar = Vector{width: 18 height: 18 viewbox: vec4(0 0 24 24)
        Path{d: "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z" fill: false stroke: theme.color_highlight stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconRetry = Vector{width: 16 height: 16 viewbox: vec4(0 0 24 24)
        Path{d: "M23 4v6h-6M1 20v-6h6" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconTrash = Vector{width: 16 height: 16 viewbox: vec4(0 0 24 24)
        Path{d: "M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2m3 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6h14z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconAlert = Vector{width: 28 height: 28 viewbox: vec4(0 0 24 24)
        Path{d: "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" fill: false stroke: theme.color_error stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M12 9v4M12 17h.01" fill: false stroke: theme.color_error stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconFilter = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M22 3H2l8 9.46V19l4 2v-8.54z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconSort = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M11 5h10M11 9h7M11 13h4" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M3 4v16M7 8l-4-4-4 4" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconBookmark = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconBookmarkFill = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" fill: theme.color_label_inner stroke: theme.color_label_inner stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconPrev = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M15 18l-6-6 6-6" fill: false stroke: theme.color_label_inner stroke_width: 2.2 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconNext = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M9 18l6-6-6-6" fill: false stroke: theme.color_label_inner stroke_width: 2.2 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconAa = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M11 4H4M18 20H3M8.5 4L3 20M21 11.5L20 8l-1 3.5M20 8l1.5 6M17 20l-1.2-4M14 20l1.5-6M17.5 16H20" fill: false stroke: theme.color_label_inner stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M3 12h5.5" fill: false stroke: theme.color_label_inner stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconTranslate = Vector{width: 20 height: 20 viewbox: vec4(0 0 24 24)
        Path{d: "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
        Path{d: "M2 12h20" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconPin = Vector{width: 18 height: 18 viewbox: vec4(0 0 24 24)
        Path{d: "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z" fill: theme.color_highlight stroke: theme.color_highlight stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }
    mod.widgets.IconPinOutline = Vector{width: 18 height: 18 viewbox: vec4(0 0 24 24)
        Path{d: "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z" fill: false stroke: theme.color_label_inner_inactive stroke_width: 2.0 stroke_linecap: "round" stroke_linejoin: "round"}
    }

    // ---- buttons ----------------------------------------------------------------

    // Primary action button (amber accent).
    mod.widgets.RmPrimaryButton = mod.widgets.ButtonFlat{
        width: Fit height: 38
        padding: theme.mspace_h_3{left: theme.space_3 * 2, right: theme.space_3 * 2}
        draw_bg +: {
            color: theme.color_highlight
            color_hover: theme.color_highlight * 1.15
            color_down: theme.color_highlight * 0.85
            border_color: theme.color_highlight
            border_color_hover: theme.color_highlight * 1.15
            border_color_down: theme.color_highlight * 0.85
            border_radius: 8.0
        }
        draw_text +: {
            color: #x1a1205
            color_hover: #x1a1205
            color_down: #x1a1205
            text_style: theme.font_bold{font_size: theme.font_size_4}
        }
    }

    // Secondary (outline) button.
    mod.widgets.RmSecondaryButton = mod.widgets.ButtonFlat{
        width: Fit height: 38
        padding: theme.mspace_h_3{left: theme.space_3 * 2, right: theme.space_3 * 2}
        draw_bg +: {
            color: theme.color_outset
            color_hover: theme.color_outset_hover
            color_down: theme.color_outset_down
            border_radius: 8.0
        }
        draw_text +: {
            color: theme.color_label_inner
            color_hover: theme.color_label_inner
            color_down: theme.color_label_inner
            text_style +: {font_size: theme.font_size_4}
        }
    }

    // Small utility button (row actions).
    mod.widgets.RmSmallButton = mod.widgets.ButtonFlatter{
        width: Fit height: 28
        padding: theme.mspace_h_2{left: theme.space_2 * 1.5, right: theme.space_2 * 1.5}
        draw_bg +: {
            border_radius: 6.0
            color_hover: theme.color_outset_hover
        }
        draw_text +: {
            color: theme.color_label_inner_inactive
            color_hover: theme.color_label_inner
            text_style +: {font_size: theme.font_size_p}
        }
    }

    // ---- content card -------------------------------------------------------------

    // A cover tile: colored placeholder with the novel's initial. Color and
    // letter are assigned from Rust when populating lists.
    mod.widgets.RmCover = #(RmCover::register_widget(vm)){
        width: Fill height: Fill
        flow: Overlay
        draw_bg +: {
            color: #x334155
            border_radius: 8.0
        }
        cover_initial := Label{
            width: Fill height: Fill
            text: "R"
            align: Center
            draw_text.color: #xffffffcc
            draw_text.text_style: theme.font_bold{font_size: theme.font_size_1}
        }
    }

    // A content card used in grids (library, browse, search results).
    mod.widgets.RmCard = mod.widgets.RmTap{
        width: Fill height: Fit
        flow: Down spacing: theme.space_1
        padding: theme.space_2
        draw_bg.color: theme.color_bg_container
        draw_bg.border_radius: 10.0
        draw_bg.color_hover: theme.color_bg_highlight
        new_batch: true

        cover_wrap := View{
            width: Fill height: 150
            flow: Overlay
            cover := mod.widgets.RmCover{}
            unread_badge := RoundedView{
                width: Fit height: Fit
                align: Align{x: 0.0 y: 0.0}
                margin: Inset{left: 6.0 top: 6.0}
                padding: Inset{left: 6.0 right: 6.0 top: 2.0 bottom: 2.0}
                show_bg: true
                visible: false
                draw_bg.color: #x000000b3
                draw_bg.border_radius: 10.0
                badge_label := Label{
                    text: "0"
                    draw_text.color: #xfff
                    draw_text.text_style: theme.font_bold{font_size: theme.font_size_code}
                }
            }
            progress_wrap := View{
                width: Fill height: Fill
                align: Align{x: 0.0 y: 1.0}
                padding: theme.space_2
                card_progress := mod.widgets.RmProgressBar{
                    height: 4
                }
            }
        }
        card_title := Label{
            width: Fill
            text: "Title"
            draw_text.color: theme.color_label_inner
            draw_text.text_style: theme.font_bold{font_size: theme.font_size_p}
        }
        card_subtitle := Label{
            width: Fill
            text: ""
            draw_text.color: theme.color_label_inner_inactive
            draw_text.text_style.font_size: theme.font_size_code
        }
        card_meta := Label{
            width: Fill
            text: ""
            draw_text.color: theme.color_highlight
            draw_text.text_style.font_size: theme.font_size_code
        }
    }

    // Grid rows: 2, 3 or 4 cards per row depending on shell mode.
    mod.widgets.RmCardRow2 = View{
        width: Fill height: Fit
        flow: Right spacing: theme.space_2
        card0 := mod.widgets.RmCard{}
        card1 := mod.widgets.RmCard{}
    }
    mod.widgets.RmCardRow3 = View{
        width: Fill height: Fit
        flow: Right spacing: theme.space_2
        card0 := mod.widgets.RmCard{}
        card1 := mod.widgets.RmCard{}
        card2 := mod.widgets.RmCard{}
    }
    mod.widgets.RmCardRow4 = View{
        width: Fill height: Fit
        flow: Right spacing: theme.space_2
        card0 := mod.widgets.RmCard{}
        card1 := mod.widgets.RmCard{}
        card2 := mod.widgets.RmCard{}
        card3 := mod.widgets.RmCard{}
    }

    // ---- section header ------------------------------------------------------------

    mod.widgets.RmSectionHeader = View{
        width: Fill height: Fit
        flow: Right
        align: Align{y: 0.5}
        padding: theme.mspace_v_2{top: theme.space_2, bottom: theme.space_1}
        section_title := Label{
            text: "Section"
            draw_text.color: theme.color_label_inner
            draw_text.text_style: theme.font_bold{font_size: theme.font_size_3}
        }
        Filler{}
        section_count := Label{
            text: ""
            draw_text.color: theme.color_label_inner_inactive
            draw_text.text_style.font_size: theme.font_size_code
        }
    }

    // ---- tag chip ---------------------------------------------------------------------

    // A tappable, selectable chip (categories, search history). Selection is
    // shown via the animator-driven background tint.
    mod.widgets.RmTagChip = #(RmChip::register_widget(vm)){
        width: Fit height: Fit
        padding: theme.mspace_h_2{left: theme.space_2, right: theme.space_2, top: theme.space_1, bottom: theme.space_1}
        cursor: MouseCursor.Hand
        new_batch: true
        show_bg: true

        draw_bg +: {
            hover: instance(0.0)
            active: instance(0.0)
            color: #0000
            color_hover: instance(theme.color_bg_highlight)
            color_active: instance(theme.color_bg_highlight * 2.2)
            border_radius: uniform(10.0)
            border_size: uniform(1.0)
            border_color: theme.color_bg_highlight

            get_color: fn() -> vec4 {
                let base = vec4(self.border_color.xyz, self.border_color.w)
                let hover_color = vec4(self.color_hover.xyz, self.color_hover.w * self.hover)
                let active_color = vec4(self.color_active.xyz, self.color_active.w * self.active)
                return mix(base.mix(hover_color, min(1.0, self.hover + self.active)), active_color, self.active)
            }
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, max(1.0, self.border_radius))
                sdf.fill_keep(self.get_color())
                return sdf.result
            }
        }

        animator: Animator{
            hover: {
                default: @off
                off: AnimatorState{
                    from: {all: Forward{duration: 0.12}}
                    apply: {draw_bg: {hover: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {hover: 1.0}}
                }
            }
            active: {
                default: @off
                off: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {active: 0.0}}
                }
                on: AnimatorState{
                    from: {all: Snap}
                    apply: {draw_bg: {active: 1.0}}
                }
            }
        }

        chip_label := Label{
            text: "tag"
            draw_text.color: theme.color_highlight
            draw_text.text_style.font_size: theme.font_size_code
        }
    }

    // A wrapping row of up to 8 chips (extra slots hidden at draw time).
    mod.widgets.RmChipRow = View{
        width: Fill height: Fit
        flow: Flow.Right{wrap: true}
        spacing: theme.space_1
        chip0 := mod.widgets.RmTagChip{}
        chip1 := mod.widgets.RmTagChip{}
        chip2 := mod.widgets.RmTagChip{}
        chip3 := mod.widgets.RmTagChip{}
        chip4 := mod.widgets.RmTagChip{}
        chip5 := mod.widgets.RmTagChip{}
        chip6 := mod.widgets.RmTagChip{}
        chip7 := mod.widgets.RmTagChip{}
    }

    // ---- state views (empty / loading / error) ----------------------------------------

    mod.widgets.RmEmptyState = View{
        width: Fill height: 280
        flow: Down spacing: theme.space_2
        align: Center
        mod.widgets.IconBookOpen{}
        empty_title := Label{
            text: "Nothing here yet"
            draw_text.color: theme.color_label_inner
            draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
        }
        empty_body := Label{
            text: ""
            draw_text.color: theme.color_label_inner_inactive
            draw_text.text_style.font_size: theme.font_size_p
        }
    }

    mod.widgets.RmLoadingState = View{
        width: Fill height: 280
        flow: Down spacing: theme.space_2
        align: Center
        LoadingSpinner{}
        Label{
            text: "Loading…"
            draw_text.color: theme.color_label_inner_inactive
            draw_text.text_style.font_size: theme.font_size_p
        }
    }

    mod.widgets.RmErrorState = View{
        width: Fill height: 280
        flow: Down spacing: theme.space_2
        align: Center
        mod.widgets.IconAlert{}
        error_label := Label{
            text: "Something went wrong"
            draw_text.color: theme.color_error
            draw_text.text_style.font_size: theme.font_size_4
        }
        retry_button := mod.widgets.RmSecondaryButton{
            text: "Retry"
        }
    }

    // ---- chapter row ----------------------------------------------------------------------

    mod.widgets.RmChapterRow = mod.widgets.RmTap{
        width: Fill height: Fit
        flow: Right spacing: theme.space_2
        align: Align{y: 0.5}
        padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
        draw_bg.color: theme.color_bg_container
        draw_bg.color_hover: theme.color_bg_highlight
        draw_bg.border_radius: 8.0
        new_batch: true

        read_check := CheckBox{
            text: ""
        }
        middle := View{
            width: Fill height: Fit
            flow: Down spacing: 2
            row_title := Label{
                width: Fill
                text: "Chapter 1"
                draw_text.color: theme.color_label_inner
                draw_text.text_style.font_size: theme.font_size_p
            }
            row_meta := Label{
                width: Fill
                text: ""
                draw_text.color: theme.color_label_inner_inactive
                draw_text.text_style.font_size: theme.font_size_code
            }
        }
        download_state := Label{
            text: ""
            draw_text.color: mod.rm.color_ok
            draw_text.text_style.font_size: theme.font_size_code
        }
        download_button := mod.widgets.RmSmallButton{
            text: "Download"
        }
    }

    // ---- download row ------------------------------------------------------------------------

    mod.widgets.RmDownloadRow = RoundedView{
        width: Fill height: Fit
        flow: Down spacing: theme.space_1
        padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
        draw_bg.color: theme.color_bg_container
        draw_bg.border_radius: 8.0
        new_batch: true

        top_row := View{
            width: Fill height: Fit
            flow: Right spacing: theme.space_2
            align: Align{y: 0.5}
            titles := View{
                width: Fill height: Fit
                flow: Down spacing: 2
                dl_title := Label{
                    width: Fill
                    text: "Novel"
                    draw_text.color: theme.color_label_inner
                    draw_text.text_style: theme.font_bold{font_size: theme.font_size_p}
                }
                dl_chapter := Label{
                    width: Fill
                    text: "Chapter"
                    draw_text.color: theme.color_label_inner_inactive
                    draw_text.text_style.font_size: theme.font_size_code
                }
            }
            dl_status := Label{
                text: ""
                draw_text.color: theme.color_label_inner_inactive
                draw_text.text_style.font_size: theme.font_size_code
            }
        }
        dl_progress := mod.widgets.RmProgressBar{}
        actions_row := View{
            width: Fill height: Fit
            flow: Right spacing: theme.space_1
            align: Align{x: 1.0}
            retry_button := mod.widgets.RmSmallButton{text: "Retry"}
            cancel_button := mod.widgets.RmSmallButton{text: "Cancel"}
            remove_button := mod.widgets.RmSmallButton{text: "Remove"}
        }
    }

    // ---- settings rows ---------------------------------------------------------------------------

    mod.widgets.RmSettingsHeader = View{
        width: Fill height: Fit
        padding: theme.mspace_v_2{top: theme.space_3, bottom: theme.space_1}
        settings_header := Label{
            text: "Section"
            draw_text.color: theme.color_highlight
            draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
        }
    }

    // Row with title/subtitle and a toggle.
    mod.widgets.RmSettingsToggleRow = RoundedView{
        width: Fill height: Fit
        flow: Right spacing: theme.space_2
        align: Align{y: 0.5}
        padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
        draw_bg.color: theme.color_bg_container
        draw_bg.border_radius: 8.0
        new_batch: true
        texts := View{
            width: Fill height: Fit
            flow: Down spacing: 2
            set_title := Label{
                width: Fill
                text: "Setting"
                draw_text.color: theme.color_label_inner
                draw_text.text_style.font_size: theme.font_size_p
            }
            set_subtitle := Label{
                width: Fill
                text: ""
                draw_text.color: theme.color_label_inner_inactive
                draw_text.text_style.font_size: theme.font_size_code
            }
        }
        set_toggle := CheckBox{
            text: ""
        }
    }

    // Row with title/subtitle and a value + action button.
    mod.widgets.RmSettingsActionRow = RoundedView{
        width: Fill height: Fit
        flow: Right spacing: theme.space_2
        align: Align{y: 0.5}
        padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
        draw_bg.color: theme.color_bg_container
        draw_bg.border_radius: 8.0
        new_batch: true
        texts := View{
            width: Fill height: Fit
            flow: Down spacing: 2
            set_title := Label{
                width: Fill
                text: "Setting"
                draw_text.color: theme.color_label_inner
                draw_text.text_style.font_size: theme.font_size_p
            }
            set_subtitle := Label{
                width: Fill
                text: ""
                draw_text.color: theme.color_label_inner_inactive
                draw_text.text_style.font_size: theme.font_size_code
            }
        }
        set_value := Label{
            text: ""
            draw_text.color: theme.color_label_inner_inactive
            draw_text.text_style.font_size: theme.font_size_p
        }
        set_button := mod.widgets.RmSmallButton{
            text: "Edit"
        }
    }

    // ---- source row (Browse screen) ------------------------------------------------------------------

    mod.widgets.RmSourceRow = View{
        width: Fill height: Fit
        flow: Right spacing: theme.space_2
        align: Align{y: 0.5}
        padding: theme.mspace_2{left: theme.space_3, right: theme.space_3}
        new_batch: true
        source_logo := RoundedView{
            width: 36 height: 36
            draw_bg.color: theme.color_bg_highlight
            draw_bg.border_radius: 8.0
            source_initial := Label{
                width: Fill height: Fill
                align: Center
                text: "M"
                draw_text.color: theme.color_label_inner_inactive
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_4}
            }
        }
        texts := View{
            width: Fill height: Fit
            flow: Down spacing: 1
            source_name := Label{
                width: Fill
                text: "Source"
                draw_text.color: theme.color_label_inner
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_p}
            }
            source_lang := Label{
                width: Fill
                text: ""
                draw_text.color: theme.color_label_inner_inactive
                draw_text.text_style.font_size: theme.font_size_code
            }
        }
        browse_tap := mod.widgets.RmTap{
            width: Fit height: Fit
            cursor: MouseCursor.Hand
            show_bg: false
            browse_link := Label{
                text: "Browse"
                draw_text.color: theme.color_highlight
                draw_text.text_style: theme.font_bold{font_size: theme.font_size_p}
            }
        }
        pin_icon := View{
            width: 20 height: 20
        }
    }

    // ---- search field ------------------------------------------------------------------------------

    mod.widgets.RmTextInput = mod.widgets.TextInput{
        width: Fill height: 40
        padding: Inset{left: 14.0 right: 14.0 top: 10.0 bottom: 0.0}
        label_align: Align{y: 0.5}
        empty_text: "Search…"
        return_key_type: Search
        draw_bg +: {
            color: theme.color_inset
            color_hover: theme.color_inset_hover
            color_focus: theme.color_inset_focus
            color_empty: theme.color_inset_empty
            border_color: theme.color_bg_highlight
            border_color_hover: theme.color_label_inner_inactive
            border_color_focus: theme.color_highlight
            border_color_empty: theme.color_bg_highlight
            border_radius: 10.0
        }
        draw_text +: {
            color: theme.color_text
            color_hover: theme.color_text
            color_focus: theme.color_text
            color_down: theme.color_text
            color_empty: theme.color_text_placeholder
            color_empty_hover: theme.color_text_placeholder
            color_empty_focus: theme.color_text_placeholder
            text_style +: {font_size: theme.font_size_4}
        }
        draw_cursor +: {
            color: theme.color_cursor
        }
    }
}

/// A thin determinate progress bar (track + fill), used for reading progress
/// and download progress.
#[derive(Script, ScriptHook, Widget)]
pub struct RmProgressBar {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[live]
    draw_track: DrawQuad,
    #[redraw]
    #[live]
    draw_fill: DrawQuad,
    #[live]
    progress: f32,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
}

impl Widget for RmProgressBar {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle(walk);
        self.draw_track.draw_abs(cx, rect);
        let fraction = self.progress.clamp(0.0, 1.0) as f64;
        if fraction > 0.0 {
            let fill = Rect {
                pos: rect.pos,
                size: dvec2(rect.size.x * fraction, rect.size.y),
            };
            self.draw_fill.draw_abs(cx, fill);
        }
        DrawStep::done()
    }
}

impl RmProgressBarRef {
    /// Set the displayed fraction (`0.0..=1.0`) and redraw.
    pub fn set_progress(&self, cx: &mut Cx, progress: f32) {
        if let Some(mut inner) = self.borrow_mut()
            && (inner.progress - progress).abs() > f32::EPSILON
        {
            inner.progress = progress;
            inner.redraw(cx);
        }
    }
}

/// Actions emitted by [`RmTap`].
#[derive(Clone, Debug, Default)]
pub enum RmTapAction {
    #[default]
    None,
    Clicked,
}

/// A clickable container view. Emits [`RmTapAction::Clicked`] on tap.
///
/// Child widgets with their own hit handling (checkboxes, buttons) capture
/// their area first, which suppresses the tap for presses that land on them.
#[derive(Script, ScriptHook, Widget)]
pub struct RmTap {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
}

impl Widget for RmTap {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Observe taps on our own area before forwarding to children.
        if let Hit::FingerUp(fe) = event.hits(cx, self.view.area())
            && fe.is_over
            && fe.is_primary_hit()
            && fe.was_tap()
        {
            cx.widget_action(self.widget_uid(), RmTapAction::Clicked);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl RmTapRef {
    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            return matches!(item.cast(), RmTapAction::Clicked);
        }
        false
    }
}

/// Check whether a widget (by uid) was tap-clicked in this action batch.
pub fn tap_clicked(actions: &Actions, uid: WidgetUid) -> bool {
    if let Some(item) = actions.find_widget_action(uid) {
        return matches!(item.cast(), RmTapAction::Clicked);
    }
    false
}

/// A cover tile with a programmatic background color (placeholder covers
/// until real cover fetching lands).
#[derive(Script, ScriptHook, Widget)]
pub struct RmCover {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[redraw]
    #[live]
    draw_bg: DrawColor,
}

impl Widget for RmCover {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.peek_walk_turtle(walk);
        self.draw_bg.draw_abs(cx, rect);
        self.view.draw_walk(cx, scope, walk)
    }
}

impl RmCoverRef {
    /// Set the cover background color.
    pub fn set_color(&self, color: Vec4f) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.draw_bg.color = color;
        }
    }
}

/// Actions emitted by [`RmChip`].
#[derive(Clone, Debug, Default)]
pub enum RmChipAction {
    #[default]
    None,
    Clicked,
}

/// A tappable, selectable chip (View + Animator, same selection model as
/// the navigation buttons).
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct RmChip {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[apply_default]
    animator: Animator,
}

impl Widget for RmChip {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        let area = self.view.area();
        let widget_uid = self.widget_uid();
        match event.hits(cx, area) {
            Hit::FingerHoverIn(_) => self.animator_play(cx, ids!(hover.on)),
            Hit::FingerHoverOut(_) => self.animator_play(cx, ids!(hover.off)),
            Hit::FingerUp(fe) => {
                if fe.is_over && fe.is_primary_hit() && fe.was_tap() {
                    cx.widget_action(widget_uid, RmChipAction::Clicked);
                }
                if fe.device.has_hovers() && fe.is_over {
                    self.animator_play(cx, ids!(hover.on));
                } else {
                    self.animator_play(cx, ids!(hover.off));
                }
            }
            _ => {}
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl RmChipRef {
    pub fn set_selected(&self, cx: &mut Cx, selected: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.animator_toggle(cx, selected, Animate::No, ids!(active.on), ids!(active.off));
        }
    }

    pub fn clicked(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            return matches!(item.cast(), RmChipAction::Clicked);
        }
        false
    }
}

/// Actions emitted by [`RmVerticalSlider`].
#[derive(Clone, Debug, Default)]
pub enum RmVerticalSliderAction {
    #[default]
    None,
    /// The slider value changed (new value in `0.0..=1.0`).
    Changed,
}

/// A vertical scrubber slider for the reader chapter position indicator.
#[derive(Script, ScriptHook, Widget)]
pub struct RmVerticalSlider {
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[redraw]
    #[live]
    draw_track: DrawQuad,
    #[redraw]
    #[live]
    draw_thumb: DrawQuad,
    #[live]
    value: f32,
    #[live]
    min_val: f32,
    #[live]
    max_val: f32,
}

impl Widget for RmVerticalSlider {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        let area = self.draw_track.area();
        match event.hits(cx, area) {
            Hit::FingerDown(fe) => {
                let rel_y = ((fe.abs.y - fe.rect.pos.y) / fe.rect.size.y) as f32;
                self.value = self.min_val + rel_y * (self.max_val - self.min_val);
                self.value = self.value.clamp(self.min_val, self.max_val);
                cx.widget_action(self.widget_uid(), RmVerticalSliderAction::Changed);
            }
            Hit::FingerUp(fe) if fe.is_over => {
                let rel_y = ((fe.abs.y - fe.rect.pos.y) / fe.rect.size.y) as f32;
                self.value = self.min_val + rel_y * (self.max_val - self.min_val);
                self.value = self.value.clamp(self.min_val, self.max_val);
                cx.widget_action(self.widget_uid(), RmVerticalSliderAction::Changed);
            }
            _ => {}
        }
        self.view.handle_event(cx, event, _scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.peek_walk_turtle(walk);
        self.draw_track.draw_abs(cx, rect);
        let fraction = if (self.max_val - self.min_val).abs() > 0.001 {
            ((self.value - self.min_val) / (self.max_val - self.min_val)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let thumb_size = 8.0f64;
        let thumb_y = rect.pos.y + (rect.size.y - thumb_size) * fraction as f64;
        let thumb_rect = Rect {
            pos: dvec2(rect.pos.x, thumb_y),
            size: dvec2(rect.size.x, thumb_size),
        };
        self.draw_thumb.draw_abs(cx, thumb_rect);
        self.view.draw_walk(cx, scope, walk)
    }
}

impl RmVerticalSliderRef {
    pub fn set_range(&self, _cx: &mut Cx, min: f32, max: f32) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.min_val = min;
            inner.max_val = max;
        }
    }

    pub fn set_value(&self, _cx: &mut Cx, value: f32) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.value = value.clamp(inner.min_val, inner.max_val);
        }
    }

    pub fn get_value(&self) -> f32 {
        self.borrow().map(|b| b.value).unwrap_or(0.0)
    }

    pub fn changed(&self, actions: &Actions) -> bool {
        if let Some(item) = actions.find_widget_action(self.widget_uid()) {
            return matches!(item.cast(), RmVerticalSliderAction::Changed);
        }
        false
    }
}
