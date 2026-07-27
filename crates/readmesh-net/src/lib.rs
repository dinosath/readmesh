//! P2P networking layer for readmesh federation.
//!
//! Provides:
//! - Node identity and peer addressing
//! - Gossip-based chapter announcement for novel topics
//! - Content-addressed blob transfer for chapter content
//!
//! The transport is abstracted behind the `P2pBackend` trait so that
//! implementations (iroh, in-memory, etc.) can be swapped without
//! touching the gossip or blob layers.

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Utc;
use readmesh_core::id::{ChapterId, NodeId, NovelId};
use readmesh_core::peer::PeerAnnouncement;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

mod backend;
mod federation;

pub use backend::{InMemoryBackend, P2pBackend, P2pEvent};
pub use federation::FederatedNode;

/// Configuration for the networking layer.
#[derive(Debug, Clone)]
pub struct NetConfig {
    /// Local data directory.
    pub data_dir: std::path::PathBuf,
    /// Optional port to bind on (0 = random).
    pub bind_port: u16,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            data_dir: std::path::PathBuf::from("./readmesh-data"),
            bind_port: 0,
        }
    }
}

/// The main networking node handle.
///
/// Wraps a P2P backend and provides high-level operations:
/// - Announce chapter availability to novel topics
/// - Listen for announcements from peers
/// - Transfer chapter content blobs
pub struct NetNode<B: P2pBackend> {
    node_id: NodeId,
    backend: Arc<B>,
    /// Outstanding gossip subscriptions: topic -> sender
    subscriptions: RwLock<BTreeMap<NovelId, mpsc::Sender<PeerAnnouncement>>>,
}

impl<B: P2pBackend> NetNode<B> {
    /// Create a new NetNode backed by the given P2P backend.
    /// Automatically starts the event loop in a background task.
    pub async fn new(backend: Arc<B>) -> Result<Arc<Self>, NetError> {
        let node_id = backend.node_id();
        let node = Arc::new(Self {
            node_id,
            backend,
            subscriptions: RwLock::new(BTreeMap::new()),
        });

        // Spawn the event loop to process incoming P2P events
        let event_node = node.clone();
        tokio::spawn(async move {
            if let Err(e) = event_node.run_event_loop().await {
                tracing::error!("Event loop error: {e}");
            }
        });

        Ok(node)
    }

    /// Get this node's ID.
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Return a reference to the backend.
    pub fn backend(&self) -> &Arc<B> {
        &self.backend
    }

    /// Subscribe to announcements for a novel topic.
    ///
    /// Returns a receiver that yields `PeerAnnouncement` messages
    /// whenever any peer in the topic announces a chapter.
    pub async fn subscribe(&self, novel_id: NovelId) -> mpsc::Receiver<PeerAnnouncement> {
        let (tx, rx) = mpsc::channel(256);
        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(novel_id, tx);
        }
        rx
    }

    /// Announce chapter availability to all peers subscribed to this novel's topic.
    pub async fn announce_chapter(
        &self,
        novel_id: NovelId,
        chapter_id: ChapterId,
        content_hash: blake3::Hash,
        have: bool,
    ) -> Result<(), NetError> {
        let announcement = PeerAnnouncement {
            node_id: self.node_id,
            novel_id,
            chapter_id,
            content_hash,
            seen_at: Utc::now(),
            have,
        };
        self.backend.announce(announcement.clone()).await?;

        // Also fan-out to local subscribers (other parts of the app)
        if let Some(tx) = self.subscriptions.read().await.get(&novel_id) {
            let _ = tx.send(announcement).await;
        }

        Ok(())
    }

    /// Store a blob addressed by its content hash.
    pub async fn put_blob(&self, hash: &blake3::Hash, data: Vec<u8>) -> Result<(), NetError> {
        self.backend.put_blob(*hash, data).await
    }

    /// Fetch a blob by its content hash from any available peer.
    pub async fn get_blob(&self, hash: &blake3::Hash) -> Result<Option<Vec<u8>>, NetError> {
        self.backend.get_blob(*hash).await
    }

    /// Run the event loop: process incoming P2P events.
    ///
    /// This should be spawned as a background task. It routes
    /// announcements to the appropriate topic subscribers.
    pub async fn run_event_loop(&self) -> Result<(), NetError> {
        loop {
            let event = self.backend.next_event().await?;
            match event {
                P2pEvent::Announce(announcement) => {
                    tracing::debug!(
                        "Received announce from {} for novel {}, chapter {}",
                        announcement.node_id,
                        announcement.novel_id,
                        announcement.chapter_id
                    );
                    if let Some(tx) = self.subscriptions.read().await.get(&announcement.novel_id) {
                        let _ = tx.send(announcement).await;
                    }
                }
                P2pEvent::PeerJoined(_node_id) => {
                    // Future: track known peers
                }
                P2pEvent::PeerLeft(_node_id) => {
                    // Future: update peer state
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetError {
    #[error("backend error: {0}")]
    Backend(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
