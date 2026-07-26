use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{ChapterId, NodeId, NovelId};

/// An announcement gossiped between nodes indicating chapter availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAnnouncement {
    pub node_id: NodeId,
    pub novel_id: NovelId,
    pub chapter_id: ChapterId,
    pub content_hash: blake3::Hash,
    pub seen_at: DateTime<Utc>,
    pub have: bool,
}
