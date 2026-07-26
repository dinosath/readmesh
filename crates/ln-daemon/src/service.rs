use std::sync::Arc;

use ln_core::federation::{FederationStatus, FollowedPeer};
use ln_core::id::{ChapterId, NodeId, NovelId, PluginId};
use ln_core::library::Library;
use ln_core::novel::Novel;
use ln_core::source::PluginManifest;
use ln_plugin::PluginHost;
use ln_plugin::ReferencePlugin;
use ln_rpc::{RpcRequest, RpcResponse};
use ln_store::Store;

/// The core daemon service that ties together store, plugins, and networking.
pub struct DaemonService {
    store: Arc<Store>,
    plugin_host: PluginHost,
    library: tokio::sync::RwLock<Library>,
    device_id: NodeId,
    federation: tokio::sync::RwLock<FederationStatus>,
}

impl DaemonService {
    /// Create a new daemon service backed by the given data directory.
    pub async fn new(data_dir: &std::path::Path) -> anyhow::Result<Self> {
        let store = Arc::new(Store::open(data_dir)?);

        // Register the reference plugin
        let mut plugin_host = PluginHost::new();
        plugin_host.register(ReferencePlugin::new());

        // Load library from store into memory
        let mut library = Library::new();
        if let Ok(novels) = store.list_novels() {
            for novel in novels {
                let nid = novel.id;
                library.add_novel(novel);
                if let Ok(chapters) = store.list_chapters_for_novel(&nid) {
                    for chapter in chapters {
                        library.add_chapter(chapter);
                    }
                }
            }
        }
        let novel_count = library.novel_count();
        tracing::info!("Loaded {novel_count} novels from store");

        // Placeholder: net node not started yet
        let device_id = NodeId([0u8; 32]);

        Ok(Self {
            store,
            plugin_host,
            library: tokio::sync::RwLock::new(library),
            device_id,
            federation: tokio::sync::RwLock::new(FederationStatus::default()),
        })
    }

    /// Process an RPC request and return the response.
    pub async fn handle(&self, request: RpcRequest) -> RpcResponse {
        match request {
            RpcRequest::GetLibrary => {
                let lib = self.library.read().await;
                let novels: Vec<Novel> = lib.novels().cloned().collect();
                RpcResponse::Library { novels }
            }
            RpcRequest::AddNovel { novel } => {
                let id = novel.id;
                // Persist to store
                if let Err(e) = self.store.insert_novel(&novel) {
                    return RpcResponse::Error {
                        message: format!("failed to insert novel: {e}"),
                    };
                }
                // Update in-memory library
                {
                    let mut lib = self.library.write().await;
                    lib.add_novel(novel);
                }
                let lib = self.library.read().await;
                let result = lib.get_novel(&id).cloned();
                RpcResponse::Novel { novel: result }
            }
            RpcRequest::RemoveNovel { novel_id } => {
                if let Some(nid) = bytes_to_novel_id(&novel_id) {
                    if let Err(e) = self.store.delete_novel(&nid) {
                        return RpcResponse::Error {
                            message: format!("failed to delete novel: {e}"),
                        };
                    }
                    let mut lib = self.library.write().await;
                    lib.remove_novel(&nid);
                }
                RpcResponse::Ok
            }
            RpcRequest::GetNovel { novel_id } => {
                let nid = bytes_to_novel_id(&novel_id);
                let novel = match nid {
                    Some(id) => {
                        let lib = self.library.read().await;
                        lib.get_novel(&id).cloned()
                    }
                    None => None,
                };
                RpcResponse::Novel { novel }
            }
            RpcRequest::GetChapters { novel_id } => {
                let nid = bytes_to_novel_id(&novel_id);
                let chapters = match nid {
                    Some(id) => {
                        let lib = self.library.read().await;
                        lib.chapters_for_novel(&id)
                            .into_iter()
                            .cloned()
                            .collect()
                    }
                    None => vec![],
                };
                RpcResponse::Chapters { chapters }
            }
            RpcRequest::GetChapterContent {
                chapter_id,
                metalink_hash: _,
            } => {
                if let Some(cid) = bytes_to_chapter_id(&chapter_id) {
                    let lib = self.library.read().await;
                    if let Some(chapter) = lib.get_chapter(&cid) {
                        match self.store.get_blob(&chapter.content_hash).await {
                            Ok(Some(data)) => {
                                return RpcResponse::ChapterContent { data };
                            }
                            Ok(None) => {
                                return RpcResponse::Error {
                                    message: "chapter content not available locally".into(),
                                };
                            }
                            Err(e) => {
                                return RpcResponse::Error {
                                    message: format!("blob error: {e}"),
                                };
                            }
                        }
                    }
                }
                RpcResponse::Error {
                    message: "chapter not found".into(),
                }
            }
            RpcRequest::GetProgress { novel_id } => {
                let nid = bytes_to_novel_id(&novel_id);
                let progress = match nid {
                    Some(id) => {
                        let lib = self.library.read().await;
                        lib.get_progress(&id, &self.device_id).cloned()
                    }
                    None => None,
                };
                RpcResponse::Progress { progress }
            }
            RpcRequest::SetProgress { progress } => {
                let novel_id = progress.novel_id;
                self.store.set_progress(&progress).ok();
                {
                    let mut lib = self.library.write().await;
                    lib.set_progress(progress);
                }
                let lib = self.library.read().await;
                let result = lib.get_progress(&novel_id, &self.device_id).cloned();
                RpcResponse::Progress { progress: result }
            }
            RpcRequest::Search {
                plugin_id,
                query,
                page,
            } => {
                let pid = PluginId(plugin_id);
                match self.plugin_host.search(Some(&pid), &query, page).await {
                    Ok(results) => RpcResponse::SearchResults { results },
                    Err(e) => RpcResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            RpcRequest::FetchNovel { plugin_id, url } => {
                let pid = PluginId(plugin_id);
                match self.plugin_host.fetch_novel(&pid, &url).await {
                    Ok(novel) => RpcResponse::Novel {
                        novel: Some(novel),
                    },
                    Err(e) => RpcResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            RpcRequest::FetchChapters {
                plugin_id,
                novel_url,
            } => {
                let pid = PluginId(plugin_id);
                match self
                    .plugin_host
                    .fetch_chapter_list(&pid, &novel_url)
                    .await
                {
                    Ok(chapters) => RpcResponse::Chapters { chapters },
                    Err(e) => RpcResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            RpcRequest::ListPlugins => {
                let plugins: Vec<PluginManifest> = self
                    .plugin_host
                    .list_plugins()
                    .iter()
                    .map(|p| p.manifest())
                    .collect();
                RpcResponse::Plugins { plugins }
            }
            RpcRequest::InstallPlugin { manifest } => {
                if let Err(e) = self.store.insert_plugin(&manifest) {
                    return RpcResponse::Error {
                        message: format!("failed to persist plugin: {e}"),
                    };
                }
                RpcResponse::Ok
            }
            RpcRequest::GetPeers => RpcResponse::Peers {
                peer_ids: vec![],
            },
            RpcRequest::GetNodeId => RpcResponse::NodeId {
                node_id: self.device_id.as_bytes().to_vec(),
            },
            RpcRequest::GetFederationStatus => {
                let status = self.federation.read().await.clone();
                RpcResponse::FederationStatus { status }
            }
            RpcRequest::SetMirrorConfig { config } => {
                self.federation.write().await.mirror_config = config;
                RpcResponse::Ok
            }
            RpcRequest::FollowPeer { node_id, alias } => {
                if let Some(bytes) = node_id_to_bytes(&node_id) {
                    let peer = FollowedPeer {
                        node_id: NodeId(bytes),
                        alias,
                        since: chrono::Utc::now(),
                        auto_mirror: true,
                    };
                    self.federation.write().await.followed_peers.push(peer);
                    RpcResponse::Ok
                } else {
                    RpcResponse::Error {
                        message: "invalid node ID".into(),
                    }
                }
            }
            RpcRequest::UnfollowPeer { node_id } => {
                if let Some(bytes) = node_id_to_bytes(&node_id) {
                    let nid = NodeId(bytes);
                    self.federation
                        .write()
                        .await
                        .followed_peers
                        .retain(|p| p.node_id != nid);
                    RpcResponse::Ok
                } else {
                    RpcResponse::Error {
                        message: "invalid node ID".into(),
                    }
                }
            }
        }
    }
}

fn node_id_to_bytes(bytes: &[u8]) -> Option<[u8; 32]> {
    if bytes.len() == 32 {
        bytes.try_into().ok()
    } else {
        None
    }
}

fn bytes_to_novel_id(bytes: &[u8]) -> Option<NovelId> {
    if bytes.len() == 32 {
        let arr: [u8; 32] = bytes.try_into().ok()?;
        Some(NovelId(blake3::Hash::from(arr)))
    } else {
        None
    }
}

fn bytes_to_chapter_id(bytes: &[u8]) -> Option<ChapterId> {
    if bytes.len() == 32 {
        let arr: [u8; 32] = bytes.try_into().ok()?;
        Some(ChapterId(blake3::Hash::from(arr)))
    } else {
        None
    }
}
