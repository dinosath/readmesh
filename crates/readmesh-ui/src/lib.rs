//! # readmesh-ui — ReadMesh Makepad application
//!
//! The native multiplatform UI for ReadMesh, built with Makepad 2.0
//! (`script_mod!` + Splash). This crate is a thin projection layer over
//! [`readmesh_app`]: all navigation, state machines and domain logic live in
//! `readmesh-app` and are unit tested there.
//!
//! Structure:
//! - [`app`] — application entry point, event routing, global state bridge
//! - [`theme`] — ReadMesh design system (colors, typography, spacing)
//! - [`components`] — reusable widget templates (cards, rows, states)
//! - [`shell`] — adaptive app shell (navigation rail / bottom bar)
//! - [`screens`] — the eight application screens

pub use makepad_widgets;

pub mod app;
pub mod components;
pub mod screens;
pub mod shell;
pub mod state;
pub mod theme;
