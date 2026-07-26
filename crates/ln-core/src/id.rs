use blake3::Hash;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A unique identifier for a novel.
///
/// Generated as the BLAKE3 hash of the canonical source URL and title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NovelId(pub Hash);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChapterId(pub Hash);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthorId(pub Hash);

/// A unique identifier for a source plugin.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PluginId(pub String);

/// A unique identifier for a node (iroh EndpointId).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 32]);

impl NovelId {
    /// Compute a NovelId from the canonical source URL and title.
    pub fn compute(source_url: &str, title: &str) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(source_url.as_bytes());
        hasher.update(b"\x00");
        hasher.update(title.as_bytes());
        Self(hasher.finalize())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl std::cmp::PartialOrd for NovelId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for NovelId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}

impl ChapterId {
    /// Compute a ChapterId from the canonical chapter URL and its index in the novel.
    pub fn compute(chapter_url: &str, novel_id: &NovelId, index: u32) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(novel_id.as_bytes());
        hasher.update(&index.to_le_bytes());
        hasher.update(chapter_url.as_bytes());
        Self(hasher.finalize())
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl std::cmp::PartialOrd for ChapterId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for ChapterId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}

impl AuthorId {
    pub fn compute(name: &str) -> Self {
        Self(blake3::hash(name.as_bytes()))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

impl std::cmp::PartialOrd for AuthorId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for AuthorId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}

impl NodeId {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for NovelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl fmt::Display for ChapterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl fmt::Display for AuthorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0[..8] {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, "..")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn novel_id_deterministic() {
        let a = NovelId::compute("https://example.com/novel", "Test Novel");
        let b = NovelId::compute("https://example.com/novel", "Test Novel");
        assert_eq!(a, b);
    }

    #[test]
    fn novel_id_different_for_different_inputs() {
        let a = NovelId::compute("https://a.com/novel", "Title A");
        let b = NovelId::compute("https://b.com/novel", "Title A");
        assert_ne!(a, b);

        let c = NovelId::compute("https://a.com/novel", "Title B");
        assert_ne!(a, c);
    }

    #[test]
    fn chapter_id_different_for_different_index() {
        let nid = NovelId::compute("https://example.com/novel", "Test");
        let c1 = ChapterId::compute("https://example.com/ch1", &nid, 0);
        let c2 = ChapterId::compute("https://example.com/ch1", &nid, 1);
        assert_ne!(c1, c2);
    }

    #[test]
    fn author_id_deterministic() {
        assert_eq!(AuthorId::compute("Alice"), AuthorId::compute("Alice"));
    }

    #[test]
    fn display_formats() {
        let nid = NovelId::compute("https://example.com", "Test");
        assert_eq!(format!("{}", nid).len(), 64);

        let pid = PluginId("test-plugin".into());
        assert_eq!(format!("{}", pid), "test-plugin");
    }
}
