//! Source plugin trait and runtime.
//!
//! Plugins scrape novel sites to produce canonical `readmesh-core` types.
//! They are intended to run as WASM components, but native implementations
//! are supported for development and testing.

use readmesh_core::chapter::Chapter;
use readmesh_core::novel::Novel;
use readmesh_core::source::PluginManifest;

mod host;
mod reference;
mod wasm_host;

pub use host::PluginHost;
pub use reference::ReferencePlugin;
pub use wasm_host::{DeclarativeScraper, ScraperManifest, ScraperSelectors, WasmPlugin};

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("rate limited")]
    RateLimited,
    #[error("plugin internal error: {0}")]
    Internal(String),
}

/// The trait every source plugin must implement.
#[async_trait::async_trait]
pub trait SourcePlugin: Send + Sync {
    /// Return this plugin's manifest.
    fn manifest(&self) -> PluginManifest;

    /// Search for novels matching a query.
    async fn search(&self, query: &str, page: u32) -> PluginResult<Vec<Novel>>;

    /// Fetch full details for a novel.
    async fn fetch_novel(&self, url: &str) -> PluginResult<Novel>;

    /// Fetch the list of chapters for a novel.
    async fn fetch_chapter_list(&self, novel_url: &str) -> PluginResult<Vec<Chapter>>;

    /// Fetch the raw content (HTML/text) of a chapter.
    async fn fetch_chapter_content(&self, chapter_url: &str) -> PluginResult<bytes::Bytes>;
}
