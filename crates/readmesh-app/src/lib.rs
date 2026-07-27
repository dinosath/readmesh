//! # readmesh-app — ReadMesh application core
//!
//! UI-agnostic application state and domain services for the ReadMesh
//! reader application. This crate sits between the Makepad UI (`readmesh-ui`) and
//! the data layer (`readmesh-core`, future daemon/P2P backends):
//!
//! ```text
//! Makepad UI (readmesh-ui)
//!     ↓
//! AppState / navigation / screen states   (this crate)
//!     ↓
//! Domain services & repository traits     (this crate)
//!     ↓
//! Repositories: MockCatalog today, ReadMesh daemon / P2P tomorrow
//! ```
//!
//! Everything here is free of UI-framework dependencies so application and
//! UI-adjacent logic is unit testable without a graphical environment.

pub mod chapters;
pub mod downloads;
pub mod library;
pub mod mock;
pub mod navigation;
pub mod reader;
pub mod repository;
pub mod search;
pub mod settings;
pub mod state;

pub use chapters::{ChapterFilter, ChapterListState, ChapterSort};
pub use downloads::{Download, DownloadCounts, DownloadManager, DownloadStatus};
pub use library::{ContinueReadingItem, LibraryState};
pub use mock::{MockCatalog, NovelMeta};
pub use navigation::{NavMode, NavigationState, PrimaryTab, Route};
pub use reader::{ReaderSettings, ReaderState, ReaderTheme};
pub use repository::{ContentRepository, SourceInfo};
pub use search::{SearchFilter, SearchPhase, SearchSort, SearchState};
pub use settings::{AppSettings, ThemeMode};
pub use state::AppState;
