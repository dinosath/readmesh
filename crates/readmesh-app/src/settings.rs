//! Application settings, structured so new sections can be added without
//! touching existing ones.

use crate::downloads::DownloadManager;
use crate::reader::ReaderSettings;

/// Application-wide color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    /// Dark, reader-friendly theme (default).
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn toggled(self) -> Self {
        match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
        }
    }
}

/// Download behavior preferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSettings {
    /// Maximum simultaneous downloads.
    pub max_concurrent: usize,
    /// Only download on unmetered connections.
    pub wifi_only: bool,
    /// Automatically download this many upcoming unread chapters.
    pub auto_download_next: u32,
    /// Remove downloads of chapters that have been read.
    pub delete_after_read: bool,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            max_concurrent: DownloadManager::DEFAULT_MAX_CONCURRENT,
            wifi_only: false,
            auto_download_next: 0,
            delete_after_read: false,
        }
    }
}

/// Storage / cache preferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSettings {
    /// Cache size cap in MiB.
    pub cache_limit_mb: u64,
    /// Bytes currently used by cached content (mock-reported).
    pub cache_used_bytes: u64,
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            cache_limit_mb: 512,
            cache_used_bytes: 0,
        }
    }
}

/// Network preferences.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NetworkSettings {
    /// Reduce data usage (defer covers, smaller pages).
    pub data_saver: bool,
    /// Participate in P2P mirroring for followed peers.
    pub p2p_mirroring: bool,
}

/// Full application settings.
#[derive(Debug, Clone, Default)]
pub struct AppSettings {
    pub theme: ThemeMode,
    pub reader: ReaderSettings,
    pub downloads: DownloadSettings,
    pub storage: StorageSettings,
    pub network: NetworkSettings,
    /// Per-source enable flags keyed by source id (empty = all enabled).
    pub disabled_sources: Vec<String>,
    /// Pinned / favorited source ids (empty = none pinned).
    pub pinned_sources: Vec<String>,
}

impl AppSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_theme(&mut self) -> ThemeMode {
        self.theme = self.theme.toggled();
        self.theme
    }

    pub fn is_source_enabled(&self, source_id: &str) -> bool {
        !self.disabled_sources.iter().any(|s| s == source_id)
    }

    /// Toggle a source; returns the new enabled state.
    pub fn toggle_source(&mut self, source_id: &str) -> bool {
        if let Some(pos) = self.disabled_sources.iter().position(|s| s == source_id) {
            self.disabled_sources.remove(pos);
            true
        } else {
            self.disabled_sources.push(source_id.to_string());
            false
        }
    }

    // ---- pinned sources ----------------------------------------------------

    pub fn is_pinned(&self, source_id: &str) -> bool {
        self.pinned_sources.iter().any(|s| s == source_id)
    }

    pub fn pin_source(&mut self, source_id: &str) {
        if !self.is_pinned(source_id) {
            self.pinned_sources.push(source_id.to_string());
        }
    }

    pub fn unpin_source(&mut self, source_id: &str) {
        self.pinned_sources.retain(|s| s != source_id);
    }

    pub fn toggle_pin(&mut self, source_id: &str) -> bool {
        if self.is_pinned(source_id) {
            self.unpin_source(source_id);
            false
        } else {
            self.pin_source(source_id);
            true
        }
    }

    /// Clear cached content; returns the number of bytes freed.
    pub fn clear_cache(&mut self) -> u64 {
        let freed = self.storage.cache_used_bytes;
        self.storage.cache_used_bytes = 0;
        freed
    }

    /// App version string for the About section.
    pub fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_toggles_between_dark_and_light() {
        let mut s = AppSettings::new();
        assert_eq!(s.theme, ThemeMode::Dark, "dark is the default");
        assert_eq!(s.toggle_theme(), ThemeMode::Light);
        assert_eq!(s.toggle_theme(), ThemeMode::Dark);
    }

    #[test]
    fn sources_toggle() {
        let mut s = AppSettings::new();
        assert!(s.is_source_enabled("meshpress"));
        assert!(!s.toggle_source("meshpress"));
        assert!(!s.is_source_enabled("meshpress"));
        assert!(s.toggle_source("meshpress"));
        assert!(s.is_source_enabled("meshpress"));
    }

    #[test]
    fn clear_cache_frees_and_zeroes() {
        let mut s = AppSettings::new();
        s.storage.cache_used_bytes = 1024;
        assert_eq!(s.clear_cache(), 1024);
        assert_eq!(s.storage.cache_used_bytes, 0);
        assert_eq!(s.clear_cache(), 0);
    }

    #[test]
    fn download_defaults() {
        let d = DownloadSettings::default();
        assert_eq!(d.max_concurrent, 3);
        assert!(!d.wifi_only);
    }
}
