//! Download queue state machine.
//!
//! Downloads are modeled as an explicit state machine, fully isolated from
//! the UI. The UI advances the simulation with [`DownloadManager::tick`]
//! (driven by a timer); real backends will drive the same transitions from
//! actual transfer events.

use std::collections::{HashMap, HashSet};

use readmesh_core::{ChapterId, NovelId};

/// Lifecycle states of a single download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    /// Waiting for a download slot.
    Queued,
    /// Actively transferring.
    Downloading,
    /// Finished; content is available offline.
    Completed,
    /// Failed with an error message; can be retried.
    Failed(String),
}

impl DownloadStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, DownloadStatus::Queued | DownloadStatus::Downloading)
    }
}

/// A single chapter download.
#[derive(Debug, Clone)]
pub struct Download {
    pub chapter_id: ChapterId,
    pub novel_id: NovelId,
    pub novel_title: String,
    pub chapter_index: u32,
    pub chapter_title: String,
    /// Progress in the range `0.0..=1.0`.
    pub progress: f32,
    pub status: DownloadStatus,
    pub size_bytes: u64,
}

impl Download {
    fn new(
        chapter_id: ChapterId,
        novel_id: NovelId,
        novel_title: String,
        chapter_index: u32,
        chapter_title: String,
        size_bytes: u64,
    ) -> Self {
        Self {
            chapter_id,
            novel_id,
            novel_title,
            chapter_index,
            chapter_title,
            progress: 0.0,
            status: DownloadStatus::Queued,
            size_bytes,
        }
    }
}

/// Summary counts used by the downloads screen and badges.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DownloadCounts {
    pub queued: usize,
    pub downloading: usize,
    pub completed: usize,
    pub failed: usize,
}

/// The download manager: an ordered queue with bounded concurrency.
#[derive(Debug, Clone)]
pub struct DownloadManager {
    items: Vec<Download>,
    max_concurrent: usize,
    /// Deterministic failure injection: when a download's progress crosses
    /// the mapped threshold, it fails with a simulated network error.
    /// Used by the mock backend so the failure/retry path is exercisable.
    fail_at: HashMap<ChapterId, f32>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadManager {
    pub const DEFAULT_MAX_CONCURRENT: usize = 3;

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_concurrent: Self::DEFAULT_MAX_CONCURRENT,
            fail_at: HashMap::new(),
        }
    }

    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Queue a chapter for download. Returns `false` if the chapter is
    /// already queued, downloading, or completed.
    pub fn enqueue(
        &mut self,
        chapter_id: ChapterId,
        novel_id: NovelId,
        novel_title: impl Into<String>,
        chapter_index: u32,
        chapter_title: impl Into<String>,
        size_bytes: u64,
    ) -> bool {
        if self.items.iter().any(|d| d.chapter_id == chapter_id) {
            return false;
        }
        self.items.push(Download::new(
            chapter_id,
            novel_id,
            novel_title.into(),
            chapter_index,
            chapter_title.into(),
            size_bytes,
        ));
        true
    }

    /// All downloads in queue order.
    pub fn items(&self) -> &[Download] {
        &self.items
    }

    pub fn get(&self, chapter_id: &ChapterId) -> Option<&Download> {
        self.items.iter().find(|d| &d.chapter_id == chapter_id)
    }

    pub fn status_of(&self, chapter_id: &ChapterId) -> Option<&DownloadStatus> {
        self.get(chapter_id).map(|d| &d.status)
    }

    /// Whether the chapter content is available offline.
    pub fn is_downloaded(&self, chapter_id: &ChapterId) -> bool {
        matches!(self.status_of(chapter_id), Some(DownloadStatus::Completed))
    }

    /// Number of active (queued or downloading) downloads.
    pub fn active_count(&self) -> usize {
        self.items.iter().filter(|d| d.status.is_active()).count()
    }

    pub fn counts(&self) -> DownloadCounts {
        let mut counts = DownloadCounts::default();
        for d in &self.items {
            match d.status {
                DownloadStatus::Queued => counts.queued += 1,
                DownloadStatus::Downloading => counts.downloading += 1,
                DownloadStatus::Completed => counts.completed += 1,
                DownloadStatus::Failed(_) => counts.failed += 1,
            }
        }
        counts
    }

    /// All chapter ids of completed downloads (used to render "downloaded"
    /// badges in the chapter list).
    pub fn downloaded_chapters(&self) -> HashSet<ChapterId> {
        self.items
            .iter()
            .filter(|d| matches!(d.status, DownloadStatus::Completed))
            .map(|d| d.chapter_id)
            .collect()
    }

    /// Advance the simulation by one tick.
    ///
    /// * Queued downloads are promoted to `Downloading` while slots are free.
    /// * Each active download advances by `rate` (fraction of total per tick).
    /// * A download whose progress crosses a seeded failure threshold fails.
    /// * Downloads reaching `1.0` complete.
    pub fn tick(&mut self, rate: f32) {
        let rate = rate.clamp(0.0, 1.0);
        // Promote queued items into free slots.
        let mut running = self
            .items
            .iter()
            .filter(|d| matches!(d.status, DownloadStatus::Downloading))
            .count();
        for item in &mut self.items {
            if running >= self.max_concurrent {
                break;
            }
            if matches!(item.status, DownloadStatus::Queued) {
                item.status = DownloadStatus::Downloading;
                running += 1;
            }
        }
        // Advance active downloads.
        let fail_at = self.fail_at.clone();
        for item in &mut self.items {
            if !matches!(item.status, DownloadStatus::Downloading) {
                continue;
            }
            let before = item.progress;
            item.progress = (item.progress + rate).min(1.0);
            if let Some(&threshold) = fail_at.get(&item.chapter_id)
                && before < threshold
                && item.progress >= threshold
                && item.progress < 1.0
            {
                item.status = DownloadStatus::Failed("simulated network error".to_string());
                continue;
            }
            if item.progress >= 1.0 {
                item.progress = 1.0;
                item.status = DownloadStatus::Completed;
            }
        }
    }

    /// Cancel a queued or downloading item, removing it from the queue.
    /// Returns `false` for completed downloads or unknown ids.
    pub fn cancel(&mut self, chapter_id: &ChapterId) -> bool {
        let Some(pos) = self
            .items
            .iter()
            .position(|d| &d.chapter_id == chapter_id && d.status.is_active())
        else {
            return false;
        };
        self.items.remove(pos);
        true
    }

    /// Retry a failed download: resets progress and requeues it.
    /// Also clears any seeded failure for the chapter so retries succeed.
    pub fn retry(&mut self, chapter_id: &ChapterId) -> bool {
        let Some(item) = self.items.iter_mut().find(|d| &d.chapter_id == chapter_id) else {
            return false;
        };
        if !matches!(item.status, DownloadStatus::Failed(_)) {
            return false;
        }
        item.status = DownloadStatus::Queued;
        item.progress = 0.0;
        self.fail_at.remove(chapter_id);
        true
    }

    /// Remove a download entry in any state (e.g. delete a completed
    /// download to free storage).
    pub fn remove(&mut self, chapter_id: &ChapterId) -> bool {
        let before = self.items.len();
        self.items.retain(|d| &d.chapter_id != chapter_id);
        self.fail_at.remove(chapter_id);
        self.items.len() != before
    }

    /// Remove all completed downloads. Returns the number removed.
    pub fn clear_completed(&mut self) -> usize {
        let before = self.items.len();
        self.items
            .retain(|d| !matches!(d.status, DownloadStatus::Completed));
        before - self.items.len()
    }

    /// Seed a deterministic failure at a progress threshold (mock backend).
    pub fn seed_failure(&mut self, chapter_id: ChapterId, threshold: f32) {
        self.fail_at.insert(chapter_id, threshold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u8) -> ChapterId {
        ChapterId(blake3::hash(&[n; 8]))
    }

    fn novel(n: u8) -> NovelId {
        NovelId(blake3::hash(&[n; 9]))
    }

    fn enqueue(dm: &mut DownloadManager, n: u8) -> ChapterId {
        let cid = id(n);
        assert!(dm.enqueue(
            cid,
            novel(1),
            "Novel",
            n as u32,
            format!("Chapter {n}"),
            1024
        ));
        cid
    }

    #[test]
    fn queueing_starts_queued_then_promotes_on_tick() {
        let mut dm = DownloadManager::new();
        let c = enqueue(&mut dm, 1);
        assert!(matches!(dm.status_of(&c), Some(DownloadStatus::Queued)));
        assert_eq!(dm.active_count(), 1);

        dm.tick(0.1);
        assert!(matches!(
            dm.status_of(&c),
            Some(DownloadStatus::Downloading)
        ));
    }

    #[test]
    fn duplicate_enqueue_is_rejected() {
        let mut dm = DownloadManager::new();
        let c = enqueue(&mut dm, 1);
        assert!(!dm.enqueue(c, novel(1), "Novel", 1, "Chapter 1", 1024));
        assert_eq!(dm.items().len(), 1);
    }

    #[test]
    fn concurrency_limit_is_respected() {
        let mut dm = DownloadManager::new();
        dm.set_max_concurrent(2);
        let a = enqueue(&mut dm, 1);
        let b = enqueue(&mut dm, 2);
        let c = enqueue(&mut dm, 3);

        dm.tick(0.1);
        assert!(matches!(
            dm.status_of(&a),
            Some(DownloadStatus::Downloading)
        ));
        assert!(matches!(
            dm.status_of(&b),
            Some(DownloadStatus::Downloading)
        ));
        assert!(matches!(dm.status_of(&c), Some(DownloadStatus::Queued)));

        // Finish one; the queued item promotes on the following tick.
        while !matches!(dm.status_of(&a), Some(DownloadStatus::Completed)) {
            dm.tick(0.25);
        }
        dm.tick(0.0);
        assert!(matches!(
            dm.status_of(&c),
            Some(DownloadStatus::Downloading)
        ));
    }

    #[test]
    fn progress_advances_and_completes() {
        let mut dm = DownloadManager::new();
        let c = enqueue(&mut dm, 1);
        dm.tick(0.5);
        assert!((dm.get(&c).unwrap().progress - 0.5).abs() < f32::EPSILON);
        dm.tick(0.5);
        let d = dm.get(&c).unwrap();
        assert!(matches!(d.status, DownloadStatus::Completed));
        assert!((d.progress - 1.0).abs() < f32::EPSILON);
        assert!(dm.is_downloaded(&c));
    }

    #[test]
    fn seeded_failure_and_retry() {
        let mut dm = DownloadManager::new();
        let c = enqueue(&mut dm, 1);
        dm.seed_failure(c, 0.5);
        dm.tick(0.3);
        dm.tick(0.3);
        match dm.status_of(&c) {
            Some(DownloadStatus::Failed(msg)) => assert!(msg.contains("network")),
            other => panic!("expected failure, got {other:?}"),
        }
        assert_eq!(dm.counts().failed, 1);

        assert!(dm.retry(&c));
        assert!(matches!(dm.status_of(&c), Some(DownloadStatus::Queued)));
        // Retry clears the failure seed, so it now completes.
        for _ in 0..20 {
            dm.tick(0.25);
        }
        assert!(dm.is_downloaded(&c));
    }

    #[test]
    fn cancel_removes_active_download() {
        let mut dm = DownloadManager::new();
        let c = enqueue(&mut dm, 1);
        dm.tick(0.1);
        assert!(dm.cancel(&c));
        assert!(dm.get(&c).is_none());
        assert_eq!(dm.active_count(), 0);
    }

    #[test]
    fn cancel_rejects_completed_and_unknown() {
        let mut dm = DownloadManager::new();
        let c = enqueue(&mut dm, 1);
        for _ in 0..10 {
            dm.tick(0.25);
        }
        assert!(!dm.cancel(&c), "completed downloads cannot be cancelled");
        assert!(!dm.cancel(&id(99)), "unknown ids cannot be cancelled");
    }

    #[test]
    fn remove_works_in_any_state() {
        let mut dm = DownloadManager::new();
        let c = enqueue(&mut dm, 1);
        for _ in 0..10 {
            dm.tick(0.25);
        }
        assert!(dm.remove(&c));
        assert!(dm.get(&c).is_none());
    }

    #[test]
    fn clear_completed_only_removes_completed() {
        let mut dm = DownloadManager::new();
        dm.set_max_concurrent(1);
        let a = enqueue(&mut dm, 1);
        let b = enqueue(&mut dm, 2);
        // a completes (4 ticks at 0.25); b is still queued/downloading.
        for _ in 0..6 {
            dm.tick(0.25);
        }
        assert!(dm.is_downloaded(&a));
        assert!(dm.status_of(&b).is_some_and(DownloadStatus::is_active));
        let removed = dm.clear_completed();
        assert_eq!(removed, 1);
        assert!(dm.get(&a).is_none());
        assert_eq!(dm.items().len(), 1);
    }

    #[test]
    fn counts_are_accurate() {
        let mut dm = DownloadManager::new();
        let _a = enqueue(&mut dm, 1);
        let _b = enqueue(&mut dm, 2);
        let c = enqueue(&mut dm, 3);
        dm.seed_failure(c, 0.1);
        for _ in 0..3 {
            dm.tick(0.25);
        }
        let counts = dm.counts();
        assert_eq!(counts.failed, 1);
        assert_eq!(counts.completed + counts.downloading + counts.queued, 2);
    }
}
