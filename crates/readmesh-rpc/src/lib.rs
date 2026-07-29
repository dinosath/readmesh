//! RPC contract shared between daemon and UI.

pub mod types;

use readmesh_core::chapter::Chapter;
use readmesh_core::federation::{FederationStatus, MirrorConfig};
use readmesh_core::novel::Novel;
use readmesh_core::progress::ReadingProgress;
use readmesh_core::source::PluginManifest;
use serde::{Deserialize, Serialize};

/// Core RPC service definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcRequest {
    // Library
    GetLibrary,
    AddNovel {
        novel: Novel,
    },
    RemoveNovel {
        novel_id: Vec<u8>,
    },
    GetNovel {
        novel_id: Vec<u8>,
    },

    // Chapters
    GetChapters {
        novel_id: Vec<u8>,
    },
    GetChapterContent {
        chapter_id: Vec<u8>,
        metalink_hash: Option<Vec<u8>>,
    },

    // Reading progress
    GetProgress {
        novel_id: Vec<u8>,
    },
    SetProgress {
        progress: ReadingProgress,
    },

    // Search/Browse
    Search {
        plugin_id: String,
        query: String,
        page: u32,
    },
    FetchNovel {
        plugin_id: String,
        url: String,
    },
    FetchChapters {
        plugin_id: String,
        novel_url: String,
    },

    // Plugins
    ListPlugins,
    InstallPlugin {
        manifest: PluginManifest,
    },

    // Authoring
    CreateProject {
        title: String,
    },
    LoadProject {
        data: Vec<u8>,
    },
    ImportFromSource {
        plugin_id: String,
        url: String,
    },

    // Network/Peers
    GetPeers,
    GetNodeId,
    GetFederationStatus,
    SetMirrorConfig {
        config: MirrorConfig,
    },
    FollowPeer {
        node_id: Vec<u8>,
        alias: Option<String>,
    },
    UnfollowPeer {
        node_id: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RpcResponse {
    // Library
    Library { novels: Vec<Novel> },
    Novel { novel: Option<Novel> },
    Ok,

    // Chapters
    Chapters { chapters: Vec<Chapter> },
    ChapterContent { data: Vec<u8> },

    // Reading progress
    Progress { progress: Option<ReadingProgress> },

    // Search
    SearchResults { results: Vec<Novel> },

    // Plugins
    Plugins { plugins: Vec<PluginManifest> },

    // Authoring
    ProjectData { data: Vec<u8> },
    ProjectInfo { title: String, chapters: usize },

    // Network
    Peers { peer_ids: Vec<String> },
    NodeId { node_id: Vec<u8> },
    FederationStatus { status: FederationStatus },

    // General
    Error { message: String },
}
