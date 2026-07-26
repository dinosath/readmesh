//! P2P backend abstraction.
//!
//! The `P2pBackend` trait abstracts the underlying transport layer
//! (iroh, in-memory simulation, etc.) so that gossip and blob logic
//! does not depend on a specific P2P stack.

use std::collections::HashMap;
use std::sync::Arc;

use ln_core::id::NodeId;
use ln_core::peer::PeerAnnouncement;
use tokio::sync::{mpsc, Mutex};

use super::NetError;

/// Events emitted by a P2P backend.
#[derive(Debug, Clone)]
pub enum P2pEvent {
    /// A peer announced chapter availability.
    Announce(PeerAnnouncement),
    /// A new peer joined the swarm.
    PeerJoined(NodeId),
    /// A peer left the swarm.
    PeerLeft(NodeId),
}

/// Abstraction over a P2P transport layer.
///
/// Implementations include:
/// - `InMemoryBackend` (testing)
/// - `IrohBackend` (production, future work)
#[async_trait::async_trait]
pub trait P2pBackend: Send + Sync + 'static {
    /// Get this node's identity.
    fn node_id(&self) -> NodeId;

    /// Announce a message to all peers.
    async fn announce(&self, announcement: PeerAnnouncement) -> Result<(), NetError>;

    /// Store a blob locally and optionally share it.
    async fn put_blob(&self, hash: blake3::Hash, data: Vec<u8>) -> Result<(), NetError>;

    /// Fetch a blob from the network by hash.
    async fn get_blob(&self, hash: blake3::Hash) -> Result<Option<Vec<u8>>, NetError>;

    /// Wait for the next event from the network.
    async fn next_event(&self) -> Result<P2pEvent, NetError>;
}

/// A simulated P2P backend for testing federation logic.
///
/// All "peers" share this backend instance, which routes
/// announcements and blobs through in-memory channels.
pub struct InMemoryBackend {
    node_id: NodeId,
    /// All announcements ever sent (global shared state).
    announcements: Arc<Mutex<Vec<PeerAnnouncement>>>,
    /// Blob store (global shared state).
    blobs: Arc<Mutex<HashMap<blake3::Hash, Vec<u8>>>>,
    /// Channel for incoming events.
    #[expect(dead_code)]
    event_tx: mpsc::Sender<P2pEvent>,
    event_rx: Mutex<mpsc::Receiver<P2pEvent>>,
}

impl InMemoryBackend {
    /// Create a new backend with a deterministic node ID.
    pub fn new(node_id: NodeId) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1024);
        Self {
            node_id,
            announcements: Arc::new(Mutex::new(Vec::new())),
            blobs: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            event_rx: Mutex::new(event_rx),
        }
    }

    /// Create a cluster of backends that can communicate with each other.
    ///
    /// All backends in the cluster share the same announcement and blob stores,
    /// so announcing from one backend delivers events to all others.
    pub fn create_cluster(count: usize) -> Vec<Arc<InMemoryBackend>> {
        let shared_announcements = Arc::new(Mutex::new(Vec::new()));
        let shared_blobs = Arc::new(Mutex::new(HashMap::new()));
        let mut event_senders = Vec::new();

        let mut backends = Vec::with_capacity(count);
        for i in 0..count {
            let mut id_bytes = [0u8; 32];
            id_bytes[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            let node_id = NodeId(id_bytes);

            let (tx, rx) = mpsc::channel(1024);
            event_senders.push(tx.clone());

            backends.push(Arc::new(InMemoryBackend {
                node_id,
                announcements: shared_announcements.clone(),
                blobs: shared_blobs.clone(),
                event_tx: tx,
                event_rx: Mutex::new(rx),
            }));
        }

        // Wire up: when any backend announces, deliver to all others
        for backend in backends.iter() {
            let announcements = backend.announcements.clone();
            let _blobs = backend.blobs.clone();
            let senders = event_senders.clone();
            let _own_id = backend.node_id;

            // Spawn a task that watches for new announcements and fans out
            let _backend = backend.clone();
            tokio::spawn(async move {
                // Simple polling: periodically check for new announcements
                let mut last_len = 0usize;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    let guard = announcements.lock().await;
                    let current_len = guard.len();
                    if current_len > last_len
                        && let Some(announcement) = guard.get(current_len - 1).cloned() {
                            drop(guard);
                            // Deliver to all backends except self
                            for sender in &senders {
                                let _ = sender
                                    .send(P2pEvent::Announce(announcement.clone()))
                                    .await;
                            }
                            last_len = current_len;
                    }
                }
            });
        }

        backends
    }
}

#[async_trait::async_trait]
impl P2pBackend for InMemoryBackend {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    async fn announce(&self, announcement: PeerAnnouncement) -> Result<(), NetError> {
        let mut guard = self.announcements.lock().await;
        guard.push(announcement);
        Ok(())
    }

    async fn put_blob(&self, hash: blake3::Hash, data: Vec<u8>) -> Result<(), NetError> {
        let mut guard = self.blobs.lock().await;
        guard.insert(hash, data);
        Ok(())
    }

    async fn get_blob(&self, hash: blake3::Hash) -> Result<Option<Vec<u8>>, NetError> {
        let guard = self.blobs.lock().await;
        Ok(guard.get(&hash).cloned())
    }

    async fn next_event(&self) -> Result<P2pEvent, NetError> {
        let mut rx = self.event_rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| NetError::Backend("event channel closed".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ln_core::id::NovelId;

    #[tokio::test]
    async fn cluster_announce_to_all() {
        let cluster = InMemoryBackend::create_cluster(3);
        let node1 = &cluster[0];
        let node2 = &cluster[1];
        let node3 = &cluster[2];

        let novel_id = NovelId::compute("https://test.com", "Test Novel");
        let announcement = PeerAnnouncement {
            node_id: node1.node_id(),
            novel_id,
            chapter_id: ln_core::id::ChapterId(blake3::Hash::from([0u8; 32])),
            content_hash: blake3::hash(b"test content"),
            seen_at: chrono::Utc::now(),
            have: true,
        };

        node1.announce(announcement.clone()).await.unwrap();

        // Each other node should receive the announcement
        let event2 = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            node2.next_event(),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(matches!(event2, P2pEvent::Announce(_)));

        let event3 = node3.next_event().await.unwrap();
        assert!(matches!(event3, P2pEvent::Announce(_)));
    }

    #[tokio::test]
    async fn blob_transfer_between_peers() {
        let cluster = InMemoryBackend::create_cluster(2);
        let node1 = &cluster[0];
        let node2 = &cluster[1];

        let hash = blake3::hash(b"shared content");
        let data = b"shared content".to_vec();

        // Node1 stores
        node1.put_blob(hash, data.clone()).await.unwrap();

        // Node2 fetches (shared blob store)
        let fetched = node2.get_blob(hash).await.unwrap().unwrap();
        assert_eq!(fetched, data);
    }
}
