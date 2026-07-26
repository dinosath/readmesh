//! High-level federated node that coordinates gossip, blobs, and discovery.
//!
//! A `FederatedNode` uses `NetNode` to:
//! - Subscribe to topics for novels the user follows
//! - Announce chapters the node has cached
//! - Fetch chapter content from peers
//! - Track which peers have which chapters

use std::sync::Arc;

use chrono::Utc;
use ln_core::id::{ChapterId, NodeId, NovelId};
use ln_core::peer::PeerAnnouncement;
use tokio::sync::{mpsc, RwLock};

use crate::{InMemoryBackend, NetNode, P2pBackend};

/// Tracks which peers claim to have a specific chapter.
#[derive(Debug, Default, Clone)]
pub struct ChapterAvailability {
    /// Peers that have this chapter: peer_id -> when they announced it
    pub peers: Vec<(NodeId, chrono::DateTime<Utc>)>,
    /// The content hash reported
    pub content_hash: Option<blake3::Hash>,
}

/// A federated node that participates in the gossip swarm.
///
/// It maintains subscriptions for followed novels and tracks
/// chapter availability from other peers.
pub struct FederatedNode<B: P2pBackend> {
    net: Arc<NetNode<B>>,
    /// Chapter availability: (novel_id, chapter_id) -> availability info
    availability: RwLock<std::collections::BTreeMap<(NovelId, ChapterId), ChapterAvailability>>,
}

impl FederatedNode<InMemoryBackend> {
    /// Create a set of federated nodes that can talk to each other.
    ///
    /// All nodes share the same in-memory backend cluster so
    /// announcements propagate between them.
    pub async fn create_test_cluster(count: usize) -> Vec<Arc<Self>> {
        let backends = InMemoryBackend::create_cluster(count);
        let mut nodes = Vec::with_capacity(count);

        for backend in backends {
            let node = FederatedNode::from_backend(backend).await;
            nodes.push(Arc::new(node));
        }

        nodes
    }

    async fn from_backend(backend: Arc<InMemoryBackend>) -> Self {
        let net = NetNode::new(backend).await.unwrap();
        Self {
            net,
            availability: RwLock::new(std::collections::BTreeMap::new()),
        }
    }
}

impl<B: P2pBackend> FederatedNode<B> {
    /// Create a new federated node with the given backend.
    pub async fn new(backend: Arc<B>) -> Result<Self, crate::NetError> {
        let net = NetNode::new(backend).await?;
        Ok(Self {
            net,
            availability: RwLock::new(std::collections::BTreeMap::new()),
        })
    }

    /// Get this node's ID.
    pub fn node_id(&self) -> NodeId {
        self.net.node_id()
    }

    /// Follow a novel: subscribe to its gossip topic.
    pub async fn follow_novel(
        &self,
        novel_id: NovelId,
    ) -> mpsc::Receiver<PeerAnnouncement> {
        self.net.subscribe(novel_id).await
    }

    /// Announce that this node has a chapter available.
    pub async fn announce_chapter(
        &self,
        novel_id: NovelId,
        chapter_id: ChapterId,
        content_hash: blake3::Hash,
    ) -> Result<(), crate::NetError> {
        self.net
            .announce_chapter(novel_id, chapter_id, content_hash, true)
            .await
    }

    /// Store chapter content locally and announce it to peers.
    pub async fn store_and_announce(
        &self,
        novel_id: NovelId,
        chapter_id: ChapterId,
        content: Vec<u8>,
    ) -> Result<blake3::Hash, crate::NetError> {
        let hash = blake3::hash(&content);
        self.net.put_blob(&hash, content).await?;
        self.announce_chapter(novel_id, chapter_id, hash).await?;
        Ok(hash)
    }

    /// Fetch a chapter blob from the network.
    pub async fn fetch_chapter(
        &self,
        hash: &blake3::Hash,
    ) -> Result<Option<Vec<u8>>, crate::NetError> {
        self.net.get_blob(hash).await
    }

    /// Get known availability for a chapter (which peers claim to have it).
    pub async fn chapter_availability(
        &self,
        novel_id: &NovelId,
        chapter_id: &ChapterId,
    ) -> Option<ChapterAvailability> {
        self.availability
            .read()
            .await
            .get(&(*novel_id, *chapter_id))
            .cloned()
    }

    /// Record an announcement in the availability tracker.
    pub async fn record_announcement(&self, announcement: &PeerAnnouncement) {
        let key = (announcement.novel_id, announcement.chapter_id);
        let mut avail = self.availability.write().await;
        let entry = avail.entry(key).or_default();
        if announcement.have {
            entry.peers.push((announcement.node_id, announcement.seen_at));
            entry.content_hash = Some(announcement.content_hash);
        } else {
            entry.peers.retain(|(id, _)| *id != announcement.node_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ln_core::id::{ChapterId, NovelId};
    use tokio::time::Duration;

    #[tokio::test]
    async fn federated_nodes_exchange_chapters() {
        let cluster = FederatedNode::create_test_cluster(2).await;
        let alice = &cluster[0];
        let bob = &cluster[1];

        let novel_id = NovelId::compute("https://test.com", "Shared Novel");
        let chapter_id = ChapterId(blake3::Hash::from([1u8; 32]));

        // Alice follows the novel and announces a chapter
        let mut bob_rx = bob.follow_novel(novel_id).await;

        // Wait for subscriptions to propagate
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Alice stores chapter content and announces it
        let content = b"Chapter 1: It was a dark and stormy night...".to_vec();
        let hash = alice
            .store_and_announce(novel_id, chapter_id, content.clone())
            .await
            .unwrap();

        // Bob receives the announcement
        let announcement = tokio::time::timeout(Duration::from_secs(2), bob_rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(announcement.novel_id, novel_id);
        assert_eq!(announcement.chapter_id, chapter_id);
        assert_eq!(announcement.content_hash, hash);
        assert!(announcement.have);

        // Bob can now fetch the chapter content
        let fetched = bob.fetch_chapter(&hash).await.unwrap().unwrap();
        assert_eq!(fetched, content);
    }

    #[tokio::test]
    async fn multiple_peers_announce_same_chapter() {
        let cluster = FederatedNode::create_test_cluster(3).await;
        let alice = &cluster[0];
        let bob = &cluster[1];
        let carol = &cluster[2];

        let novel_id = NovelId::compute("https://test.com", "Multi-Peer Novel");
        let chapter_id = ChapterId(blake3::Hash::from([2u8; 32]));

        // Bob and Carol subscribe
        let mut bob_rx = bob.follow_novel(novel_id).await;
        let mut carol_rx = carol.follow_novel(novel_id).await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Alice announces
        let content = b"Multi-peer chapter content".to_vec();
        let hash = alice
            .store_and_announce(novel_id, chapter_id, content.clone())
            .await
            .unwrap();

        // Both Bob and Carol receive it
        let bob_ann = tokio::time::timeout(Duration::from_secs(2), bob_rx.recv())
            .await
            .unwrap()
            .unwrap();
        let carol_ann = tokio::time::timeout(Duration::from_secs(2), carol_rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(bob_ann.content_hash, hash);
        assert_eq!(carol_ann.content_hash, hash);

        // Both can fetch
        assert!(bob.fetch_chapter(&hash).await.unwrap().is_some());
        assert!(carol.fetch_chapter(&hash).await.unwrap().is_some());
    }
}
