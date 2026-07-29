use std::collections::HashMap;
use std::sync::Arc;

use iroh::Endpoint;
use iroh::endpoint::presets;
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::BlobsProtocol;
use iroh_gossip::{Gossip, TopicId};
use iroh_gossip::api::{Event as GossipEvent, GossipSender};
use n0_future::StreamExt;
use readmesh_core::id::NodeId;
use readmesh_core::peer::PeerAnnouncement;
use tokio::io::AsyncReadExt;
use tokio::sync::{Mutex, RwLock, mpsc};

use super::{NetConfig, NetError};
use crate::{P2pBackend, P2pEvent};

pub struct IrohBackend {
    node_id: NodeId,
    _endpoint: Arc<Endpoint>,
    _router: iroh::protocol::Router,
    store: Arc<MemStore>,
    gossip: Gossip,
    event_tx: mpsc::Sender<P2pEvent>,
    event_rx: Mutex<mpsc::Receiver<P2pEvent>>,
    topic_subscriptions: RwLock<HashMap<TopicId, GossipSender>>,
}

impl IrohBackend {
    pub async fn new(config: &NetConfig) -> Result<Arc<Self>, NetError> {
        let data_dir = &config.data_dir;
        tokio::fs::create_dir_all(data_dir).await.map_err(NetError::Io)?;

        let key_path = data_dir.join("secret_key");
        let secret_key = if key_path.exists() {
            let bytes = tokio::fs::read(&key_path).await.map_err(NetError::Io)?;
            iroh::SecretKey::try_from(&bytes[..])
                .map_err(|e| NetError::Backend(format!("invalid secret key: {e}")))?
        } else {
            let key = iroh::SecretKey::generate();
            let bytes = key.to_bytes();
            tokio::fs::write(&key_path, &bytes).await.map_err(NetError::Io)?;
            key
        };

        let mut builder = Endpoint::builder(presets::Minimal)
            .secret_key(secret_key.clone())
            .relay_mode(iroh::RelayMode::Default);

        if config.bind_port != 0 {
            let sock_addr: std::net::SocketAddr =
                format!("0.0.0.0:{}", config.bind_port).parse().unwrap();
            builder = builder
                .clear_ip_transports()
                .bind_addr(sock_addr)
                .map_err(|e| NetError::Backend(format!("bind addr: {e}")))?;
        }

        let endpoint = builder
            .bind()
            .await
            .map_err(|e| NetError::Backend(format!("endpoint bind: {e}")))?;

        let node_id = NodeId(*endpoint.id().as_bytes());

        let store = Arc::new(MemStore::new());
        let gossip = Gossip::builder().spawn(endpoint.clone());

        let router = iroh::protocol::Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, BlobsProtocol::new(&*store, None))
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let (event_tx, event_rx) = mpsc::channel(4096);

        let backend = Arc::new(Self {
            node_id,
            _endpoint: Arc::new(endpoint),
            _router: router,
            store,
            gossip,
            event_tx,
            event_rx: Mutex::new(event_rx),
            topic_subscriptions: RwLock::new(HashMap::new()),
        });

        Ok(backend)
    }

    async fn ensure_subscribed(&self, topic_id: TopicId) -> Result<GossipSender, NetError> {
        {
            let guard = self.topic_subscriptions.read().await;
            if let Some(sender) = guard.get(&topic_id) {
                return Ok(sender.clone());
            }
        }

        let topic = self
            .gossip
            .subscribe(topic_id, vec![])
            .await
            .map_err(|e| NetError::Backend(format!("gossip subscribe: {e}")))?;

        let (sender, receiver) = topic.split();
        let event_tx = self.event_tx.clone();

        tokio::spawn(gossip_event_loop(receiver, event_tx));

        self.topic_subscriptions
            .write()
            .await
            .insert(topic_id, sender.clone());

        Ok(sender)
    }
}

async fn gossip_event_loop(
    mut receiver: iroh_gossip::api::GossipReceiver,
    event_tx: mpsc::Sender<P2pEvent>,
) {
    while let Some(event) = receiver.next().await {
        match event {
            Ok(GossipEvent::Received(msg)) => {
                if let Ok(ann) = postcard::from_bytes::<PeerAnnouncement>(&msg.content) {
                    let _ = event_tx.send(P2pEvent::Announce(ann)).await;
                }
            }
            Ok(GossipEvent::NeighborUp(peer_id)) => {
                let _ = event_tx
                    .send(P2pEvent::PeerJoined(NodeId(*peer_id.as_bytes())))
                    .await;
            }
            Ok(GossipEvent::NeighborDown(peer_id)) => {
                let _ = event_tx
                    .send(P2pEvent::PeerLeft(NodeId(*peer_id.as_bytes())))
                    .await;
            }
            _ => {}
        }
    }
}

#[async_trait::async_trait]
impl P2pBackend for IrohBackend {
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    async fn announce(&self, announcement: PeerAnnouncement) -> Result<(), NetError> {
        let topic_id = TopicId::from_bytes(*announcement.novel_id.0.as_bytes());
        let sender = self.ensure_subscribed(topic_id).await?;

        let bytes = postcard::to_allocvec(&announcement)
            .map_err(|e| NetError::Backend(format!("serialize: {e}")))?;

        sender
            .broadcast(bytes.into())
            .await
            .map_err(|e| NetError::Backend(format!("gossip broadcast: {e}")))?;

        Ok(())
    }

    async fn put_blob(&self, _hash: blake3::Hash, data: Vec<u8>) -> Result<(), NetError> {
        let _ = self
            .store
            .blobs()
            .add_slice(data)
            .temp_tag()
            .await
            .map_err(|e| NetError::Backend(format!("add_slice: {e}")))?;
        Ok(())
    }

    async fn get_blob(&self, hash: blake3::Hash) -> Result<Option<Vec<u8>>, NetError> {
        let iroh_hash = iroh_blobs::Hash::from(hash);
        let mut reader = self.store.blobs().reader(iroh_hash);
        let mut buf = Vec::new();
        match reader.read_to_end(&mut buf).await {
            Ok(_) => Ok(Some(buf)),
            Err(_) => Ok(None),
        }
    }

    async fn next_event(&self) -> Result<P2pEvent, NetError> {
        let mut rx = self.event_rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| NetError::Backend("event channel closed".into()))
    }
}

#[cfg(test)]
#[cfg(feature = "iroh-backend")]
mod tests {
    use super::*;
    use readmesh_core::id::{ChapterId, NovelId};
    use tempfile::TempDir;

    fn make_config(tmp: &TempDir, port: u16) -> NetConfig {
        NetConfig {
            data_dir: tmp.path().join("node"),
            bind_port: port,
        }
    }

    #[tokio::test]
    async fn iroh_backend_create() {
        let tmp = TempDir::new().unwrap();
        let config = make_config(&tmp, 0);
        let backend = IrohBackend::new(&config).await.unwrap();
        let _id = backend.node_id();
    }

    #[tokio::test]
    async fn iroh_backend_put_get_blob() {
        let tmp = TempDir::new().unwrap();
        let node1 = IrohBackend::new(&make_config(&tmp, 0)).await.unwrap();

        let data = b"hello iroh blobs".to_vec();
        let hash = blake3::hash(&data);
        node1.put_blob(hash, data.clone()).await.unwrap();
        let result = node1.get_blob(hash).await.unwrap();
        assert_eq!(result, Some(data));
    }

    #[tokio::test]
    #[ignore = "requires external bootstrap for gossip peers to discover each other"]
    async fn iroh_backend_announce_and_receive() {
        let tmp = TempDir::new().unwrap();
        let config_a = make_config(&tmp, 0);
        let config_b = make_config(&tmp, 0);

        let node_a = IrohBackend::new(&config_a).await.unwrap();
        let node_b = IrohBackend::new(&config_b).await.unwrap();

        let novel_id = NovelId::compute("https://test.com", "Test Novel");
        let announcement = PeerAnnouncement {
            node_id: node_a.node_id(),
            novel_id,
            chapter_id: ChapterId(blake3::Hash::from([0u8; 32])),
            content_hash: blake3::hash(b"test content"),
            seen_at: chrono::Utc::now(),
            have: true,
        };

        node_a.announce(announcement.clone()).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let event = tokio::time::timeout(
            tokio::time::Duration::from_secs(10),
            node_b.next_event(),
        )
        .await
        .unwrap()
        .unwrap();

        assert!(matches!(event, P2pEvent::Announce(_)));
    }
}
