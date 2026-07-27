use std::collections::HashMap;
use std::sync::Arc;

use readmesh_core::chapter::Chapter;
use readmesh_core::id::PluginId;
use readmesh_core::novel::Novel;

use crate::{PluginError, PluginResult, SourcePlugin};

/// Manages a set of installed source plugins.
pub struct PluginHost {
    plugins: HashMap<PluginId, Arc<dyn SourcePlugin>>,
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a plugin with the host.
    pub fn register(&mut self, plugin: impl SourcePlugin + 'static) {
        let manifest = plugin.manifest();
        self.plugins.insert(manifest.id.clone(), Arc::new(plugin));
    }

    /// List all registered plugins.
    pub fn list_plugins(&self) -> Vec<&Arc<dyn SourcePlugin>> {
        self.plugins.values().collect()
    }

    /// Get a specific plugin by ID.
    pub fn get_plugin(&self, id: &PluginId) -> Option<&Arc<dyn SourcePlugin>> {
        self.plugins.get(id)
    }

    /// Search across all plugins (or a specific one).
    pub async fn search(
        &self,
        plugin_id: Option<&PluginId>,
        query: &str,
        page: u32,
    ) -> PluginResult<Vec<Novel>> {
        if let Some(id) = plugin_id {
            let plugin = self
                .plugins
                .get(id)
                .ok_or_else(|| PluginError::NotFound(format!("plugin not found: {id}")))?;
            plugin.search(query, page).await
        } else {
            let mut results = Vec::new();
            for plugin in self.plugins.values() {
                if let Ok(mut novels) = plugin.search(query, page).await {
                    results.append(&mut novels);
                }
            }
            Ok(results)
        }
    }

    /// Fetch a novel by URL using the specified plugin.
    pub async fn fetch_novel(&self, plugin_id: &PluginId, url: &str) -> PluginResult<Novel> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(format!("plugin not found: {plugin_id}")))?;
        plugin.fetch_novel(url).await
    }

    /// Fetch chapter list for a novel using the specified plugin.
    pub async fn fetch_chapter_list(
        &self,
        plugin_id: &PluginId,
        novel_url: &str,
    ) -> PluginResult<Vec<Chapter>> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(format!("plugin not found: {plugin_id}")))?;
        plugin.fetch_chapter_list(novel_url).await
    }

    /// Fetch chapter content using the specified plugin.
    pub async fn fetch_chapter_content(
        &self,
        plugin_id: &PluginId,
        chapter_url: &str,
    ) -> PluginResult<bytes::Bytes> {
        let plugin = self
            .plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(format!("plugin not found: {plugin_id}")))?;
        plugin.fetch_chapter_content(chapter_url).await
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference::ReferencePlugin;

    #[tokio::test]
    async fn plugin_host_register_and_search() {
        let mut host = PluginHost::new();
        host.register(ReferencePlugin::new());

        let plugins = host.list_plugins();
        assert_eq!(plugins.len(), 1);

        let results = host.search(None, "test", 1).await.unwrap();
        // Reference plugin returns one matching novel
        assert!(!results.is_empty());
    }
}
