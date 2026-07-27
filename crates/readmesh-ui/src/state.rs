//! Global bridge to the application state.
//!
//! All state lives in [`readmesh_app::AppState`]; this module exposes it to
//! widgets. Screen widgets read state during `draw_walk`; mutations happen
//! exclusively in `App::handle_actions` / `MatchEvent` handlers, followed by
//! a `redraw`. This mirrors the canonical Makepad example pattern and keeps
//! a strict one-way data flow:
//!
//! ```text
//! user input -> App actions -> mutate APP_STATE -> redraw -> widgets read
//! ```

use std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use readmesh_app::AppState;

static APP_STATE: LazyLock<RwLock<AppState>> = LazyLock::new(|| RwLock::new(AppState::demo()));

/// Read-only access to the application state.
pub fn state() -> RwLockReadGuard<'static, AppState> {
    match APP_STATE.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Mutable access to the application state (event handlers only).
pub fn state_mut() -> RwLockWriteGuard<'static, AppState> {
    match APP_STATE.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Convenience: run a closure against the state read-only.
pub fn with_state<R>(f: impl FnOnce(&AppState) -> R) -> R {
    f(&state())
}

/// Convenience: run a closure against the state mutably.
pub fn with_state_mut<R>(f: impl FnOnce(&mut AppState) -> R) -> R {
    f(&mut state_mut())
}
