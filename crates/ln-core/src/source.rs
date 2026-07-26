use serde::{Deserialize, Serialize};

use crate::id::PluginId;

/// Metadata about an installed source plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub supported_sites: Vec<String>,
    pub capabilities: PluginCapabilities,
}

/// Capabilities a plugin declares.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginCapabilities {
    pub search: bool,
    pub fetch_novel: bool,
    pub fetch_chapters: bool,
    pub fetch_content: bool,
}
