use async_trait::async_trait;
use bytes::Bytes;
use readmesh_core::chapter::Chapter;
use readmesh_core::id::NovelId;
use readmesh_core::novel::Novel;
use readmesh_core::source::PluginManifest;

use crate::{PluginError, PluginResult, SourcePlugin};
use blake3;

/// A WASM-based plugin that runs untrusted source plugins in a sandboxed
/// wasmtime runtime.
///
/// Plugins communicate via JSON-serialized bytes across the WASM boundary.
/// The guest exports four functions: `plugin_manifest`, `plugin_search`,
/// `plugin_fetch_novel`, `plugin_fetch_chapter_list`,
/// `plugin_fetch_chapter_content`.
pub struct WasmPlugin {
    engine: wasmtime::Engine,
    module: wasmtime::Module,
    manifest: PluginManifest,
}

impl WasmPlugin {
    /// Load a WASM module from bytes.
    pub fn new(wasm_bytes: &[u8], manifest: PluginManifest) -> Result<Self, PluginError> {
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, wasm_bytes)
            .map_err(|e| PluginError::Internal(format!("failed to compile WASM: {e}")))?;
        Ok(Self {
            engine,
            module,
            manifest,
        })
    }

    fn call_wasm_fn(&self, fn_name: &str, input: &str) -> Result<String, PluginError> {
        let mut store = wasmtime::Store::new(&self.engine, ());

        let linker = wasmtime::Linker::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| PluginError::Internal(format!("instantiation failed: {e}")))?;

        let f = instance
            .get_func(&mut store, fn_name)
            .ok_or_else(|| PluginError::Internal(format!("export '{fn_name}' not found")))?;

        let ty = f.ty(&store);
        let params_count = ty.params().len();
        let results_count = ty.results().len();

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| PluginError::Internal("no memory export".into()))?;

        // Write input to WASM memory
        let input_bytes = input.as_bytes();
        let alloc = instance
            .get_func(&mut store, "alloc")
            .ok_or_else(|| PluginError::Internal("alloc not exported".into()))?;

        let alloc_ty = alloc.ty(&store);
        let alloc_results_count = alloc_ty.results().len();

        let mut alloc_results = vec![wasmtime::Val::I32(0); alloc_results_count];
        alloc
            .call(
                &mut store,
                &[wasmtime::Val::I32(input_bytes.len() as i32)],
                &mut alloc_results,
            )
            .map_err(|e| PluginError::Internal(format!("alloc failed: {e}")))?;

        let ptr = match alloc_results.into_iter().next() {
            Some(wasmtime::Val::I32(p)) => p as usize,
            _ => return Err(PluginError::Internal("alloc returned non-i32".into())),
        };

        memory
            .write(&mut store, ptr, input_bytes)
            .map_err(|e| PluginError::Internal(format!("memory write failed: {e}")))?;

        let mut params = Vec::new();
        params.push(wasmtime::Val::I32(ptr as i32));
        params.push(wasmtime::Val::I32(input_bytes.len() as i32));
        if params_count > 2 {
            params.push(wasmtime::Val::I32(0));
        }

        let mut results = vec![wasmtime::Val::I32(0); results_count];
        f.call(&mut store, &params, &mut results)
            .map_err(|e| PluginError::Internal(format!("call '{fn_name}' failed: {e}")))?;

        let result_ptr = match results.into_iter().next() {
            Some(wasmtime::Val::I32(p)) => p as u32,
            _ => return Err(PluginError::Internal("result not i32".into())),
        };

        // Read result length from WASM memory (prepend length field)
        let mut len_buf = [0u8; 4];
        memory
            .read(&store, result_ptr as usize, &mut len_buf)
            .ok();
        let result_len = u32::from_le_bytes(len_buf) as usize;

        let mut result_buf = vec![0u8; result_len.min(1024 * 1024)];
        memory
            .read(&store, result_ptr as usize + 4, &mut result_buf)
            .map_err(|e| PluginError::Internal(format!("memory read failed: {e}")))?;

        let output =
            String::from_utf8(result_buf[..result_len.min(result_buf.len())].to_vec())
                .map_err(|_| PluginError::Internal("invalid UTF-8 from WASM".into()))?;

        Ok(output)
    }
}

#[async_trait]
impl SourcePlugin for WasmPlugin {
    fn manifest(&self) -> PluginManifest {
        self.manifest.clone()
    }

    async fn search(&self, query: &str, page: u32) -> PluginResult<Vec<Novel>> {
        let input = serde_json::json!({ "query": query, "page": page }).to_string();
        let output = self.call_wasm_fn("plugin_search", &input)?;
        serde_json::from_str(&output)
            .map_err(|e| PluginError::Parse(format!("invalid search response: {e}")))
    }

    async fn fetch_novel(&self, url: &str) -> PluginResult<Novel> {
        let input = serde_json::json!({ "url": url }).to_string();
        let output = self.call_wasm_fn("plugin_fetch_novel", &input)?;
        serde_json::from_str(&output)
            .map_err(|e| PluginError::Parse(format!("invalid novel response: {e}")))
    }

    async fn fetch_chapter_list(&self, novel_url: &str) -> PluginResult<Vec<Chapter>> {
        let input = serde_json::json!({ "novel_url": novel_url }).to_string();
        let output = self.call_wasm_fn("plugin_fetch_chapter_list", &input)?;
        serde_json::from_str(&output)
            .map_err(|e| PluginError::Parse(format!("invalid chapter list response: {e}")))
    }

    async fn fetch_chapter_content(&self, chapter_url: &str) -> PluginResult<Bytes> {
        let input = serde_json::json!({ "chapter_url": chapter_url }).to_string();
        let output = self.call_wasm_fn("plugin_fetch_chapter_content", &input)?;
        Ok(Bytes::from(output.into_bytes()))
    }
}

/// A manifest-driven declarative scraper that extracts content from HTML
/// without requiring WASM plugins for common source patterns.
pub struct DeclarativeScraper {
    manifest: ScraperManifest,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScraperManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub base_url: String,
    pub selectors: ScraperSelectors,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScraperSelectors {
    pub search_form: Option<String>,
    pub search_results: String,
    pub result_title: String,
    pub result_url: String,
    pub result_author: Option<String>,
    pub result_cover: Option<String>,
    pub novel_title: String,
    pub novel_author: Option<String>,
    pub novel_summary: Option<String>,
    pub novel_cover: Option<String>,
    pub chapter_list: String,
    pub chapter_title: String,
    pub chapter_url: String,
    pub chapter_content: String,
    pub next_page: Option<String>,
}

impl DeclarativeScraper {
    pub fn new(manifest: ScraperManifest) -> Self {
        Self { manifest }
    }

    pub async fn scrape_search(&self, query: &str, page: u32) -> PluginResult<Vec<Novel>> {
        let url = if page > 1 {
            if let Some(ref next) = self.manifest.selectors.next_page {
                format!("{}{}?page={page}", self.manifest.base_url, next)
            } else {
                format!(
                    "{}/search?q={}&page={page}",
                    self.manifest.base_url,
                    urlencoding(query)
                )
            }
        } else {
            format!(
                "{}/search?q={}",
                self.manifest.base_url,
                urlencoding(query)
            )
        };

        let html = self.fetch_html(&url).await?;
        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse(&self.manifest.selectors.search_results)
            .map_err(|e| PluginError::Parse(format!("CSS selector error: {e}")))?;

        let mut novels = Vec::new();
        for element in document.select(&selector) {
            let title = self.extract_text(&element, &self.manifest.selectors.result_title);
            let url_val = self.extract_attr(&element, &self.manifest.selectors.result_url, "href");
            if let (Some(title), Some(url_val)) = (title, url_val) {
                let full_url = self.resolve_url(&url_val);
                let mut novel = Novel::new(&title, &full_url);
                if let Some(ref author_sel) = self.manifest.selectors.result_author {
                    if let Some(author) = self.extract_text(&element, author_sel) {
                        novel = novel.with_author(&author);
                    }
                }
                if let Some(ref cover_sel) = self.manifest.selectors.result_cover {
                    if let Some(cover_url) = self.extract_attr(&element, cover_sel, "src") {
                        novel = novel.with_cover(blake3::hash(self.resolve_url(&cover_url).as_bytes()));
                    }
                }
                novels.push(novel);
            }
        }

        Ok(novels)
    }

    pub async fn scrape_novel(&self, url: &str) -> PluginResult<Novel> {
        let html = self.fetch_html(url).await?;
        let document = scraper::Html::parse_document(&html);

        let title = self
            .extract_text_first(&document, &self.manifest.selectors.novel_title)
            .ok_or_else(|| PluginError::Parse("novel title not found".into()))?;

        let mut novel = Novel::new(&title, url);
        if let Some(ref sel) = self.manifest.selectors.novel_author {
            if let Some(author) = self.extract_text_first(&document, sel) {
                novel = novel.with_author(&author);
            }
        }
        if let Some(ref sel) = self.manifest.selectors.novel_summary {
            if let Some(summary) = self.extract_text_first(&document, sel) {
                novel = novel.with_summary(&summary);
            }
        }
        if let Some(ref sel) = self.manifest.selectors.novel_cover {
            if let Some(cover) = self.extract_attr_first(&document, sel, "src") {
                novel = novel.with_cover(blake3::hash(self.resolve_url(&cover).as_bytes()));
            }
        }

        Ok(novel)
    }

    pub async fn scrape_chapters(&self, novel_url: &str) -> PluginResult<Vec<Chapter>> {
        let html = self.fetch_html(novel_url).await?;
        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse(&self.manifest.selectors.chapter_list)
            .map_err(|e| PluginError::Parse(format!("CSS selector error: {e}")))?;

        let mut chapters = Vec::new();
        for (i, element) in document.select(&selector).enumerate() {
            let title = self
                .extract_text(&element, &self.manifest.selectors.chapter_title)
                .unwrap_or_else(|| format!("Chapter {}", i + 1));
            let url_val = self
                .extract_attr(&element, &self.manifest.selectors.chapter_url, "href")
                .unwrap_or_default();
            let full_url = self.resolve_url(&url_val);

            let novel_id = NovelId::compute(novel_url, "");
            let chapter = Chapter::new(
                novel_id,
                i as u32,
                &title,
                blake3::hash(full_url.as_bytes()),
            )
            .with_url(&full_url);
            chapters.push(chapter);
        }

        Ok(chapters)
    }

    pub async fn scrape_content(&self, chapter_url: &str) -> PluginResult<Bytes> {
        let html = self.fetch_html(chapter_url).await?;
        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse(&self.manifest.selectors.chapter_content)
            .map_err(|e| PluginError::Parse(format!("CSS selector error: {e}")))?;

        let mut content = String::new();
        for element in document.select(&selector) {
            content.push_str(&element.inner_html());
            content.push('\n');
        }

        Ok(Bytes::from(content.into_bytes()))
    }

    async fn fetch_html(&self, url: &str) -> PluginResult<String> {
        let response = reqwest::get(url)
            .await
            .map_err(|e| PluginError::Network(format!("fetch failed: {e}")))?;
        response
            .text()
            .await
            .map_err(|e| PluginError::Network(format!("body error: {e}")))
    }

    fn extract_text(
        &self,
        element: &scraper::ElementRef,
        selector_str: &str,
    ) -> Option<String> {
        let selector = scraper::Selector::parse(selector_str).ok()?;
        element
            .select(&selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
    }

    fn extract_attr(
        &self,
        element: &scraper::ElementRef,
        selector_str: &str,
        attr: &str,
    ) -> Option<String> {
        let selector = scraper::Selector::parse(selector_str).ok()?;
        element
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr(attr))
            .map(|s| s.to_string())
    }

    fn extract_text_first(&self, doc: &scraper::Html, selector_str: &str) -> Option<String> {
        let selector = scraper::Selector::parse(selector_str).ok()?;
        doc.select(&selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
    }

    fn extract_attr_first(
        &self,
        doc: &scraper::Html,
        selector_str: &str,
        attr: &str,
    ) -> Option<String> {
        let selector = scraper::Selector::parse(selector_str).ok()?;
        doc.select(&selector)
            .next()
            .and_then(|el| el.value().attr(attr))
            .map(|s| s.to_string())
    }

    fn resolve_url(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else if url.starts_with('/') {
            let base = self.manifest.base_url.trim_end_matches('/');
            format!("{base}{url}")
        } else {
            format!("{}/{}", self.manifest.base_url.trim_end_matches('/'), url)
        }
    }
}

fn urlencoding(s: &str) -> String {
    s.split(' ').collect::<Vec<_>>().join("%20")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_plugin_new() {
        let wat = r#"
            (module
                (memory (export "memory") 1)
                (export "alloc" (func $alloc))
                (func $alloc (param $size i32) (result i32)
                    i32.const 0
                )
                (export "plugin_search" (func $search))
                (func $search (param i32 i32 i32) (result i32)
                    i32.const 0
                )
            )
        "#;
        let wasm = wat::parse_str(wat).unwrap();
        let manifest = PluginManifest {
            id: readmesh_core::id::PluginId("test-wasm".into()),
            name: "Test WASM".into(),
            version: "0.1.0".into(),
            supported_sites: vec!["test.com".into()],
            capabilities: readmesh_core::source::PluginCapabilities {
                search: true,
                fetch_novel: true,
                fetch_chapters: true,
                fetch_content: true,
            },
        };
        let plugin = WasmPlugin::new(&wasm, manifest);
        assert!(plugin.is_ok());
    }

    #[test]
    fn declarative_scraper_manifest_roundtrip() {
        let manifest = ScraperManifest {
            id: "test".into(),
            name: "Test".into(),
            version: "1.0".into(),
            base_url: "https://example.com".into(),
            selectors: ScraperSelectors {
                search_form: None,
                search_results: "div.results".into(),
                result_title: "h2.title".into(),
                result_url: "a".into(),
                result_author: Some("span.author".into()),
                result_cover: Some("img.cover".into()),
                novel_title: "h1.novel-title".into(),
                novel_author: Some("span.author".into()),
                novel_summary: Some("div.summary".into()),
                novel_cover: Some("img.cover".into()),
                chapter_list: "ul.chapters li".into(),
                chapter_title: "a".into(),
                chapter_url: "a".into(),
                chapter_content: "div.content".into(),
                next_page: Some("/search".into()),
            },
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: ScraperManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test");
        assert_eq!(parsed.selectors.result_title, "h2.title");
    }
}
