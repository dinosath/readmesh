//! Federation configuration: seeding, mirroring, and peer-following settings.

use serde::{Deserialize, Serialize};

use crate::id::NodeId;

/// Configuration for local node's seeding/mirroring behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Whether to seed/mirror chapters this node has read.
    pub enabled: bool,
    /// Maximum storage to use for mirrored content, in bytes.
    pub storage_cap_bytes: u64,
    /// Maximum number of chapters to mirror.
    pub max_chapters: u64,
    /// Only mirror chapters from followed novels.
    pub only_followed: bool,
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            storage_cap_bytes: 1024 * 1024 * 1024, // 1 GiB
            max_chapters: 10000,
            only_followed: true,
        }
    }
}

/// A peer that this node follows (subscribes to their library).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowedPeer {
    pub node_id: NodeId,
    /// Optional display name.
    pub alias: Option<String>,
    /// When we started following.
    pub since: chrono::DateTime<chrono::Utc>,
    /// Whether to auto-mirror chapters from this peer.
    pub auto_mirror: bool,
}

/// Federation status information for the node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FederationStatus {
    /// Local mirror config.
    pub mirror_config: MirrorConfig,
    /// Peers we follow.
    pub followed_peers: Vec<FollowedPeer>,
    /// Peers we've recently seen.
    pub known_peers: Vec<KnownPeer>,
    /// Total bytes mirrored locally.
    pub mirrored_bytes: u64,
    /// Number of chapters mirrored.
    pub mirrored_chapters: u64,
}

/// A peer discovered or seen on the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownPeer {
    pub node_id: NodeId,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub novels_announced: u64,
}
