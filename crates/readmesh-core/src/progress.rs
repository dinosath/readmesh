use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{ChapterId, NodeId, NovelId};

/// Tracks reading progress for a novel on a specific device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub novel_id: NovelId,
    pub chapter_id: ChapterId,
    pub scroll_offset: u64,
    pub updated_at: DateTime<Utc>,
    pub device_id: NodeId,
}
