use async_trait::async_trait;
use bytes::Bytes;
use readmesh_core::chapter::Chapter;
use readmesh_core::id::PluginId;
use readmesh_core::novel::{Novel, NovelStatus};
use readmesh_core::source::{PluginCapabilities, PluginManifest};

use crate::{PluginError, PluginResult, SourcePlugin};

/// A reference/test plugin that returns deterministic, well-known data.
///
/// This plugin simulates a real novel source for testing purposes.
/// It serves data from a fixed in-memory catalog.
pub struct ReferencePlugin {
    manifest: PluginManifest,
}

impl ReferencePlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                id: PluginId("reference-plugin".into()),
                name: "Reference Test Plugin".into(),
                version: "0.1.0".into(),
                supported_sites: vec!["example.com".into()],
                capabilities: PluginCapabilities {
                    search: true,
                    fetch_novel: true,
                    fetch_chapters: true,
                    fetch_content: true,
                },
            },
        }
    }
}

impl Default for ReferencePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SourcePlugin for ReferencePlugin {
    fn manifest(&self) -> PluginManifest {
        self.manifest.clone()
    }

    async fn search(&self, query: &str, _page: u32) -> PluginResult<Vec<Novel>> {
        let lower = query.to_lowercase();
        let all = Self::catalog();
        let results: Vec<Novel> = all
            .into_iter()
            .filter(|n| n.title.to_lowercase().contains(&lower))
            .collect();
        Ok(results)
    }

    async fn fetch_novel(&self, url: &str) -> PluginResult<Novel> {
        Self::catalog()
            .into_iter()
            .find(|n| n.source_refs.iter().any(|s| s.remote_url == url))
            .ok_or_else(|| PluginError::NotFound(format!("novel not found at {url}")))
    }

    async fn fetch_chapter_list(&self, novel_url: &str) -> PluginResult<Vec<Chapter>> {
        let novel = self.fetch_novel(novel_url).await?;
        let novel_id = novel.id;
        Ok(Self::chapters_for_novel(novel_id))
    }

    async fn fetch_chapter_content(&self, chapter_url: &str) -> PluginResult<Bytes> {
        let content = format!(
            "<html><body><h1>Chapter Content</h1><p>This is the content for {chapter_url}</p></body></html>"
        );
        Ok(Bytes::from(content))
    }
}

impl ReferencePlugin {
    fn catalog() -> Vec<Novel> {
        vec![
            Novel::new("Test Novel One", "https://example.com/novels/test-one")
                .with_author("Alice Writer")
                .with_tag("fantasy")
                .with_tag("adventure")
                .with_summary("A test fantasy novel about heroic adventures.")
                .with_status(NovelStatus::Ongoing),
            Novel::new("Test Novel Two", "https://example.com/novels/test-two")
                .with_author("Bob Author")
                .with_tag("sci-fi")
                .with_summary("A science fiction tale of distant galaxies.")
                .with_status(NovelStatus::Completed),
            Novel::new("Test Novel Three", "https://example.com/novels/test-three")
                .with_author("Carol Storyteller")
                .with_tag("romance")
                .with_tag("drama")
                .with_summary("A dramatic romance set in modern times.")
                .with_status(NovelStatus::Hiatus),
        ]
    }

    fn chapters_for_novel(novel_id: readmesh_core::id::NovelId) -> Vec<Chapter> {
        (0..5)
            .map(|i| {
                let content = format!("Chapter {i} content for novel {}", novel_id);
                let hash = blake3::hash(content.as_bytes());
                Chapter::new(novel_id, i, format!("Chapter {i}"), hash)
                    .with_url(format!("https://example.com/novels/ch-{i}"))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search_by_title() {
        let plugin = ReferencePlugin::new();
        let results = plugin.search("one", 1).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Novel One");
    }

    #[tokio::test]
    async fn search_case_insensitive() {
        let plugin = ReferencePlugin::new();
        let results = plugin.search("ONE", 1).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn fetch_novel() {
        let plugin = ReferencePlugin::new();
        let novel = plugin
            .fetch_novel("https://example.com/novels/test-one")
            .await
            .unwrap();
        assert_eq!(novel.title, "Test Novel One");
        assert_eq!(novel.authors.len(), 1);
        assert_eq!(novel.status, NovelStatus::Ongoing);
    }

    #[tokio::test]
    async fn fetch_novel_not_found() {
        let plugin = ReferencePlugin::new();
        let result = plugin
            .fetch_novel("https://example.com/novels/nonexistent")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fetch_chapters() {
        let plugin = ReferencePlugin::new();
        let chapters = plugin
            .fetch_chapter_list("https://example.com/novels/test-one")
            .await
            .unwrap();
        assert_eq!(chapters.len(), 5);
        assert_eq!(chapters[0].index, 0);
        assert_eq!(chapters[4].index, 4);
    }

    #[tokio::test]
    async fn fetch_chapter_content() {
        let plugin = ReferencePlugin::new();
        let content = plugin
            .fetch_chapter_content("https://example.com/novels/ch-0")
            .await
            .unwrap();
        assert!(!content.is_empty());
        let text = String::from_utf8_lossy(&content);
        assert!(text.contains("Chapter Content"));
    }
}
