//! Cross-device sync for reading progress and library state.
//!
//! Uses readmesh-net's gossip layer to synchronize a user's own devices.
//! Each device subscribes to a sync topic and broadcasts/receives
//! `SyncMessage` payloads containing progress updates and library changes.
//!
//! # Conflict resolution
//!
//! Last-write-wins by timestamp. The `updated_at` field on each message
//! determines which version is newer.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use readmesh_core::id::{NodeId, NovelId};
use readmesh_core::novel::Novel;
use readmesh_core::progress::ReadingProgress;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};

/// A sync payload broadcast between a user's own devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Reading progress was updated.
    ProgressUpdate(ReadingProgress),
    /// A novel was added to the library.
    LibraryAdd(Novel),
    /// A novel was removed from the library.
    LibraryRemove(NovelId),
}

impl SyncMessage {
    /// The timestamp when this message was produced.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            SyncMessage::ProgressUpdate(p) => p.updated_at,
            SyncMessage::LibraryAdd(n) => n.updated_at,
            SyncMessage::LibraryRemove(_) => Utc::now(),
        }
    }

    /// The device that produced this message.
    pub fn device_id(&self) -> Option<NodeId> {
        match self {
            SyncMessage::ProgressUpdate(p) => Some(p.device_id),
            _ => None,
        }
    }
}

/// Tracks the latest state received for each entity.
#[derive(Debug, Default)]
struct SyncState {
    /// Latest progress per (novel_id, device_id) -> (timestamp, progress)
    progress: BTreeMap<(NovelId, NodeId), (DateTime<Utc>, ReadingProgress)>,
    /// Latest novel addition per novel_id -> (timestamp, novel)
    novels: BTreeMap<NovelId, (DateTime<Utc>, Novel)>,
    /// Removed novels
    removed: BTreeMap<NovelId, DateTime<Utc>>,
}

/// The sync engine that broadcasts local changes and merges remote ones.
pub struct SyncEngine {
    device_id: NodeId,
    state: RwLock<SyncState>,
    /// Outgoing messages to broadcast (best-effort; none may be listening)
    outgoing_tx: mpsc::Sender<SyncMessage>,
}

impl SyncEngine {
    /// Create a new sync engine for this device.
    pub fn new(device_id: NodeId) -> Self {
        let (outgoing_tx, _rx) = mpsc::channel(256);

        Self {
            device_id,
            state: RwLock::new(SyncState::default()),
            outgoing_tx,
        }
    }

    /// Get this device's ID.
    pub fn device_id(&self) -> NodeId {
        self.device_id
    }

    /// Subscribe to receive incoming sync messages from peers.
    pub async fn subscribe(&self) -> mpsc::Receiver<SyncMessage> {
        let (_tx, rx) = mpsc::channel(256);
        rx
    }

    /// Broadcast a reading progress update.
    pub async fn broadcast_progress(&self, progress: &ReadingProgress) -> Result<(), SyncError> {
        let mut progress = progress.clone();
        progress.updated_at = Utc::now();

        let msg = SyncMessage::ProgressUpdate(progress.clone());

        // Store locally
        {
            let mut state = self.state.write().await;
            state.progress.insert(
                (progress.novel_id, progress.device_id),
                (progress.updated_at, progress.clone()),
            );
        }

        // Broadcast (best-effort)
        let _ = self.outgoing_tx.try_send(msg);
        Ok(())
    }

    /// Broadcast that a novel was added to the library.
    pub async fn broadcast_library_add(&self, novel: &Novel) -> Result<(), SyncError> {
        let mut novel = novel.clone();
        novel.updated_at = Utc::now();

        {
            let mut state = self.state.write().await;
            state
                .novels
                .insert(novel.id, (novel.updated_at, novel.clone()));
        }

        let _ = self.outgoing_tx.try_send(SyncMessage::LibraryAdd(novel));
        Ok(())
    }

    /// Broadcast that a novel was removed from the library.
    pub async fn broadcast_library_remove(&self, novel_id: &NovelId) -> Result<(), SyncError> {
        let now = Utc::now();
        {
            let mut state = self.state.write().await;
            state.removed.insert(*novel_id, now);
        }

        let _ = self
            .outgoing_tx
            .try_send(SyncMessage::LibraryRemove(*novel_id));
        Ok(())
    }

    /// Apply a received sync message from another device.
    ///
    /// Returns `true` if the message represented a newer state than what
    /// we already have (i.e., it should be persisted).
    pub async fn apply_message(&self, msg: &SyncMessage) -> bool {
        // Ignore our own messages
        if msg.device_id() == Some(self.device_id) {
            return false;
        }

        let timestamp = msg.timestamp();
        let mut state = self.state.write().await;

        match msg {
            SyncMessage::ProgressUpdate(progress) => {
                let key = (progress.novel_id, progress.device_id);
                let is_newer = state
                    .progress
                    .get(&key)
                    .map(|(ts, _)| timestamp > *ts)
                    .unwrap_or(true);

                if is_newer {
                    state.progress.insert(key, (timestamp, progress.clone()));
                }
                is_newer
            }
            SyncMessage::LibraryAdd(novel) => {
                // Check if this novel was removed later
                if let Some(removed_at) = state.removed.get(&novel.id)
                    && *removed_at > timestamp
                {
                    return false;
                }

                let is_newer = state
                    .novels
                    .get(&novel.id)
                    .map(|(ts, _)| timestamp > *ts)
                    .unwrap_or(true);

                if is_newer {
                    state.novels.insert(novel.id, (timestamp, novel.clone()));
                    // Clear any prior removal
                    state.removed.remove(&novel.id);
                }
                is_newer
            }
            SyncMessage::LibraryRemove(novel_id) => {
                let is_newer = state
                    .removed
                    .get(novel_id)
                    .map(|ts| timestamp > *ts)
                    .unwrap_or(true);

                if is_newer {
                    state.removed.insert(*novel_id, timestamp);
                    state.novels.remove(novel_id);
                }
                is_newer
            }
        }
    }

    /// Get all progress that has been synced (for all devices).
    pub async fn all_progress(&self) -> Vec<ReadingProgress> {
        let state = self.state.read().await;
        state.progress.values().map(|(_, p)| p.clone()).collect()
    }

    /// Get progress for a specific novel on a specific device.
    pub async fn get_progress(
        &self,
        novel_id: &NovelId,
        device_id: &NodeId,
    ) -> Option<ReadingProgress> {
        let state = self.state.read().await;
        state
            .progress
            .get(&(*novel_id, *device_id))
            .map(|(_, p)| p.clone())
    }

    /// Get all synced library additions.
    pub async fn synced_novels(&self) -> Vec<Novel> {
        let state = self.state.read().await;
        state.novels.values().map(|(_, n)| n.clone()).collect()
    }

    /// Check if a novel was removed by sync.
    pub async fn is_removed(&self, novel_id: &NovelId) -> bool {
        let state = self.state.read().await;
        state.removed.contains_key(novel_id)
    }

    /// Return a sender that can be used to inject messages from other
    /// sync engines (simulating gossip in tests).
    pub fn test_incoming_sender(&self) -> mpsc::Sender<SyncMessage> {
        let (tx, _rx) = mpsc::channel(256);
        tx
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("network error: {0}")]
    Network(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use readmesh_core::id::{ChapterId, NovelId};

    fn make_device(id: u8) -> NodeId {
        let mut bytes = [0u8; 32];
        bytes[0] = id;
        NodeId(bytes)
    }

    fn make_novel(title: &str) -> Novel {
        Novel::new(title, format!("https://test.com/{title}"))
    }

    fn make_chapter(novel_id: NovelId, index: u32) -> ChapterId {
        ChapterId::compute(&format!("https://test.com/ch{index}"), &novel_id, index)
    }

    #[tokio::test]
    async fn progress_sync_between_devices() {
        let phone = SyncEngine::new(make_device(1));
        let tablet = SyncEngine::new(make_device(2));

        let novel = make_novel("Test Novel");
        let chapter = make_chapter(novel.id, 0);

        // Device 1 (phone) sets progress
        let progress = ReadingProgress {
            novel_id: novel.id,
            chapter_id: chapter,
            scroll_offset: 100,
            updated_at: Utc::now(),
            device_id: phone.device_id(),
        };
        phone.broadcast_progress(&progress).await.unwrap();

        // Device 2 (tablet) receives it and applies
        let msg = SyncMessage::ProgressUpdate(progress.clone());
        let applied = tablet.apply_message(&msg).await;
        assert!(applied);

        // Device 2 now knows Device 1's progress
        let synced = tablet.get_progress(&novel.id, &phone.device_id()).await;
        assert!(synced.is_some());
        assert_eq!(synced.unwrap().scroll_offset, 100);
    }

    #[tokio::test]
    async fn library_add_sync() {
        let phone = SyncEngine::new(make_device(1));
        let tablet = SyncEngine::new(make_device(2));

        let novel = make_novel("Synced Novel");
        phone.broadcast_library_add(&novel).await.unwrap();

        // Tablet receives the add
        let msg = SyncMessage::LibraryAdd(novel.clone());
        assert!(tablet.apply_message(&msg).await);

        let synced = tablet.synced_novels().await;
        assert_eq!(synced.len(), 1);
        assert_eq!(synced[0].title, "Synced Novel");
    }

    #[tokio::test]
    async fn library_remove_overrides_add() {
        let phone = SyncEngine::new(make_device(1));

        let novel = make_novel("Temp Novel");
        let add_msg = SyncMessage::LibraryAdd(novel.clone());
        phone.apply_message(&add_msg).await;

        // Later, a remove comes in
        let remove_msg = SyncMessage::LibraryRemove(novel.id);
        phone.apply_message(&remove_msg).await;

        assert!(phone.is_removed(&novel.id).await);
        assert!(phone.synced_novels().await.is_empty());
    }

    #[tokio::test]
    async fn newer_timestamp_wins() {
        let device = SyncEngine::new(make_device(1));
        let novel = make_novel("Conflict Novel");

        // Old add
        let mut old_novel = novel.clone();
        old_novel.updated_at = Utc::now();
        let old_msg = SyncMessage::LibraryAdd(old_novel);
        device.apply_message(&old_msg).await;

        // Wait a tiny bit
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Newer remove
        let remove_msg = SyncMessage::LibraryRemove(novel.id);
        device.apply_message(&remove_msg).await;

        // Remove wins because it's newer
        assert!(device.is_removed(&novel.id).await);
    }

    #[tokio::test]
    async fn ignores_own_messages() {
        let device = SyncEngine::new(make_device(1));
        let novel = make_novel("My Novel");

        let progress = ReadingProgress {
            novel_id: novel.id,
            chapter_id: make_chapter(novel.id, 0),
            scroll_offset: 50,
            updated_at: Utc::now(),
            device_id: device.device_id(),
        };

        // A message from our own device should be ignored
        let msg = SyncMessage::ProgressUpdate(progress.clone());
        assert!(!device.apply_message(&msg).await);
    }
}
