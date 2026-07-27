//! Domain model types for readmesh.
//!
//! Defines the core types: Novel, Chapter, Author, Tag, ReadingProgress,
//! and related identifiers and value objects.

pub mod chapter;
pub mod federation;
pub mod id;
pub mod library;
pub mod novel;
pub mod progress;
pub mod source;

pub mod peer;

pub use chapter::Chapter;
pub use federation::{FederationStatus, FollowedPeer, KnownPeer, MirrorConfig};
pub use id::{AuthorId, ChapterId, NodeId, NovelId, PluginId};
pub use library::Library;
pub use novel::{Author, Novel, NovelStatus, SourceRef, Tag};
pub use peer::PeerAnnouncement;
pub use progress::ReadingProgress;
pub use source::{PluginCapabilities, PluginManifest};
