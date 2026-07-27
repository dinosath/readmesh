//! Metalink4 (RFC 5854/6249) document model, parse, and serialize.

use thiserror::Error;

mod parse;
mod serialize;

pub use parse::from_xml;
pub use serialize::to_xml;

/// A Metalink4 document describing multiple ways to fetch a file.
#[derive(Debug, Clone)]
pub struct Metalink {
    /// Origin information.
    pub origin: Option<Origin>,
    /// File metadata.
    pub file: FileEntry,
}

/// Origin/publisher info.
#[derive(Debug, Clone)]
pub struct Origin {
    pub dynamic: bool,
    pub priority: Option<u32>,
}

/// The file entry in a Metalink document.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub hashes: Vec<HashEntry>,
    pub urls: Vec<UrlEntry>,
    pub signature: Option<SignatureEntry>,
}

/// A hash entry for integrity verification.
#[derive(Debug, Clone)]
pub struct HashEntry {
    pub hash_type: HashType,
    pub value: String,
}

/// Supported hash types in Metalink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashType {
    Sha256,
    Blake3,
    Other(String),
}

/// A URL entry pointing to a mirror.
#[derive(Debug, Clone)]
pub struct UrlEntry {
    pub url: String,
    pub priority: Option<u32>,
    pub location: Option<String>,
    pub preference: Option<u32>,
}

/// Optional cryptographic signature.
#[derive(Debug, Clone)]
pub struct SignatureEntry {
    pub mediatype: String,
    pub signature: String,
}

#[derive(Error, Debug)]
pub enum MetalinkError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("validation error: {0}")]
    Validation(String),
}

impl Metalink {
    /// Get all URLs that match a given scheme (e.g. "http", "iroh", "magnet").
    pub fn urls_by_scheme(&self, scheme: &str) -> Vec<&UrlEntry> {
        self.file
            .urls
            .iter()
            .filter(|u| {
                u.url.starts_with(&format!("{}://", scheme))
                    || u.url.starts_with(&format!("{}:", scheme))
            })
            .collect()
    }

    /// Build a Metalink for a chapter with a content hash and known URLs.
    pub fn builder(name: &str, size: u64) -> MetalinkBuilder {
        MetalinkBuilder::new(name, size)
    }
}

/// Builder for Metalink documents.
pub struct MetalinkBuilder {
    name: String,
    size: u64,
    hashes: Vec<HashEntry>,
    urls: Vec<UrlEntry>,
}

impl MetalinkBuilder {
    pub fn new(name: impl Into<String>, size: u64) -> Self {
        Self {
            name: name.into(),
            size,
            hashes: vec![],
            urls: vec![],
        }
    }

    pub fn add_hash(mut self, hash_type: HashType, value: impl Into<String>) -> Self {
        self.hashes.push(HashEntry {
            hash_type,
            value: value.into(),
        });
        self
    }

    pub fn add_blake3(mut self, hash: &blake3::Hash) -> Self {
        self.hashes.push(HashEntry {
            hash_type: HashType::Blake3,
            value: hash.to_hex().to_string(),
        });
        self
    }

    pub fn add_url(
        mut self,
        url: impl Into<String>,
        priority: Option<u32>,
        location: Option<String>,
    ) -> Self {
        self.urls.push(UrlEntry {
            url: url.into(),
            priority,
            location,
            preference: None,
        });
        self
    }

    pub fn build(self) -> Metalink {
        Metalink {
            origin: None,
            file: FileEntry {
                name: self.name,
                size: self.size,
                hashes: self.hashes,
                urls: self.urls,
                signature: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let hash = blake3::hash(b"test");
        let meta = Metalink::builder("test.txt", 10)
            .add_blake3(&hash)
            .add_url("https://example.com/test.txt", Some(1), None)
            .build();

        assert_eq!(meta.file.name, "test.txt");
        assert_eq!(meta.file.size, 10);
        assert_eq!(meta.file.hashes.len(), 1);
        assert_eq!(meta.file.urls.len(), 1);
    }

    #[test]
    fn test_urls_by_scheme() {
        let hash = blake3::hash(b"test");
        let meta = Metalink::builder("test.txt", 10)
            .add_blake3(&hash)
            .add_url("https://example.com/test.txt", Some(1), None)
            .add_url("magnet:?xt=urn:btih:abc", Some(2), None)
            .add_url("iroh:blob:abc123", Some(1), None)
            .build();

        assert_eq!(meta.urls_by_scheme("https").len(), 1);
        assert_eq!(meta.urls_by_scheme("magnet").len(), 1);
        assert_eq!(meta.urls_by_scheme("iroh").len(), 1);
        assert_eq!(meta.urls_by_scheme("ipfs").len(), 0);
    }

    #[test]
    fn test_roundtrip_xml() {
        let hash = blake3::hash(b"chapter content");
        let meta = Metalink::builder("chapter-001.html", 42)
            .add_blake3(&hash)
            .add_url("https://source.com/ch1", Some(1), Some("US".into()))
            .add_url("iroh:blob:def456", Some(2), None)
            .build();

        let xml = to_xml(&meta).unwrap();
        let parsed = from_xml(&xml).unwrap();

        assert_eq!(parsed.file.name, meta.file.name);
        assert_eq!(parsed.file.size, meta.file.size);
        assert_eq!(parsed.file.hashes.len(), meta.file.hashes.len());
        assert_eq!(parsed.file.urls.len(), meta.file.urls.len());
    }
}
