use std::sync::Arc;

use chrono::Utc;
use iroh::Endpoint;
use iroh_blobs::store::mem::MemStore;
use iroh_gossip::Gossip;
use readmesh_core::id::{ChapterId, NodeId, NovelId};
use readmesh_core::novel::Novel;
use readmesh_core::progress::ReadingProgress;
use tokio::sync::mpsc;

use crate::SyncError;

#[derive(Debug, Clone)]
pub enum SyncEvent {
    ProgressUpdated {
        novel_id: NovelId,
        chapter_id: ChapterId,
        scroll_offset: u64,
    },
    NovelAdded {
        novel: Novel,
    },
    NovelRemoved {
        novel_id: NovelId,
    },
}

pub struct DocsBackend {
    device_id: NodeId,
    _endpoint: Arc<Endpoint>,
    store: Arc<MemStore>,
    _gossip: Gossip,
    author: iroh_docs::AuthorId,
    doc: iroh_docs::api::Doc,
    event_tx: mpsc::Sender<SyncEvent>,
    event_rx: tokio::sync::RwLock<mpsc::Receiver<SyncEvent>>,
}

impl DocsBackend {
    pub async fn new(
        device_id: NodeId,
        endpoint: &Endpoint,
        store: &Arc<MemStore>,
        gossip: &Gossip,
    ) -> Result<Self, SyncError> {
        let gossip = gossip.clone();

        let docs = iroh_docs::Docs::memory()
            .spawn(endpoint.clone(), (**store).clone(), gossip.clone())
            .await
            .map_err(|e| SyncError::Network(format!("docs spawn: {e}")))?;

        let author = docs
            .author_default()
            .await
            .map_err(|e| SyncError::Network(format!("author_default: {e}")))?;

        let doc = docs
            .create()
            .await
            .map_err(|e| SyncError::Network(format!("create doc: {e}")))?;

        let (event_tx, event_rx) = mpsc::channel(4096);

        let evt_tx = event_tx.clone();
        let sub_doc = doc.clone();
        let sub_store = Arc::clone(store);
        let sub_author = author;
        tokio::spawn(async move {
            use tokio_stream::StreamExt;
            let Ok(stream) = sub_doc.subscribe().await else { return };
            tokio::pin!(stream);
            while let Some(result) = stream.next().await {
                let Ok(event) = result else { continue };
                match event {
                    iroh_docs::engine::LiveEvent::InsertLocal { entry }
                    | iroh_docs::engine::LiveEvent::InsertRemote { entry, .. } => {
                        let key = entry.key();
                        if let Ok(key_str) = std::str::from_utf8(key) {
                            if let Some(rest) = key_str.strip_prefix("progress/") {
                                let parts: Vec<&str> = rest.splitn(2, '/').collect();
                                if parts.len() == 2 {
                                    let hash = entry.content_hash();
                                    if let Ok(bytes) = sub_store.get_bytes(hash).await {
                                        if let Ok(reading) =
                                            postcard::from_bytes::<ReadingProgress>(&bytes)
                                        {
                                            let _ = evt_tx
                                                .send(SyncEvent::ProgressUpdated {
                                                    novel_id: reading.novel_id,
                                                    chapter_id: reading.chapter_id,
                                                    scroll_offset: reading.scroll_offset,
                                                })
                                                .await;
                                        }
                                    }
                                }
                            } else if key_str.starts_with("novel/") {
                                if entry.is_empty() {
                                    let novel_id_str = key_str.strip_prefix("novel/").unwrap_or("");
                                    let novel_id =
                                        NovelId::compute(novel_id_str, novel_id_str);
                                    let _ = evt_tx
                                        .send(SyncEvent::NovelRemoved { novel_id })
                                        .await;
                                } else {
                                    let hash = entry.content_hash();
                                    if let Ok(bytes) = sub_store.get_bytes(hash).await {
                                        if let Ok(novel) =
                                            postcard::from_bytes::<Novel>(&bytes)
                                        {
                                            let _ = evt_tx
                                                .send(SyncEvent::NovelAdded {
                                                    novel,
                                                })
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(Self {
            device_id,
            _endpoint: Arc::new(endpoint.clone()),
            store: Arc::clone(store),
            _gossip: gossip,
            author,
            doc,
            event_tx,
            event_rx: tokio::sync::RwLock::new(event_rx),
        })
    }

    pub fn device_id(&self) -> NodeId {
        self.device_id
    }

    pub async fn set_progress(
        &self,
        novel_id: NovelId,
        chapter_id: ChapterId,
        scroll_offset: u64,
    ) -> Result<(), SyncError> {
        let key = format!("progress/{}/{}", novel_id, chapter_id);
        let value = postcard::to_allocvec(&ReadingProgress {
            novel_id,
            chapter_id,
            scroll_offset,
            updated_at: Utc::now(),
            device_id: self.device_id,
        })
        .map_err(|e| SyncError::Network(format!("serialize: {e}")))?;

        self.doc
            .set_bytes(self.author, key.as_bytes().to_vec(), value)
            .await
            .map_err(|e| SyncError::Network(format!("doc set: {e}")))?;
        Ok(())
    }

    pub async fn get_progress(
        &self,
        novel_id: &NovelId,
        chapter_id: &ChapterId,
    ) -> Option<ReadingProgress> {
        let key = format!("progress/{}/{}", novel_id, chapter_id);
        let entry = self
            .doc
            .get_exact(self.author, key.as_bytes(), false)
            .await
            .ok()
            .flatten()?;
        let bytes = self
            .store
            .get_bytes(entry.content_hash())
            .await
            .ok()?;
        postcard::from_bytes(&bytes).ok()
    }

    pub async fn add_novel(&self, novel: &Novel) -> Result<(), SyncError> {
        let key = format!("novel/{}", novel.id);
        let value = postcard::to_allocvec(novel)
            .map_err(|e| SyncError::Network(format!("serialize: {e}")))?;
        self.doc
            .set_bytes(self.author, key.as_bytes().to_vec(), value)
            .await
            .map_err(|e| SyncError::Network(format!("doc set: {e}")))?;
        Ok(())
    }

    pub async fn remove_novel(&self, novel_id: &NovelId) -> Result<(), SyncError> {
        let key = format!("novel/{novel_id}");
        self.doc
            .del(self.author, key.as_bytes().to_vec())
            .await
            .map_err(|e| SyncError::Network(format!("doc del: {e}")))?;
        Ok(())
    }

    pub async fn watch_events(&self) -> mpsc::Receiver<SyncEvent> {
        let (tx, rx) = mpsc::channel(256);
        let mut inner_rx = self.event_rx.write().await;
        while let Ok(evt) = inner_rx.try_recv() {
            let _ = tx.send(evt).await;
        }
        rx
    }
}
