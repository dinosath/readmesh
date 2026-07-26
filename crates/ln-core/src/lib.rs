//! Domain model types for lnreader-rs.
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

pub use peer::PeerAnnouncement;
