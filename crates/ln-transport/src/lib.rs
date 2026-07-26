//! Transport layer: resolves Metalink documents by fetching from the
//! fastest available backend (iroh, HTTP, BitTorrent, IPFS).

use async_trait::async_trait;
use bytes::Bytes;
use ln_metalink::{HashType, Metalink, MetalinkError};

use thiserror::Error;

mod http_backend;

pub use http_backend::HttpBackend;

/// Result type for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("no available backends could fetch the resource")]
    NoBackends,
    #[error("hash verification failed: expected {expected}, got {got}")]
    HashMismatch { expected: String, got: String },
    #[error("backend error ({backend}): {message}")]
    Backend { backend: String, message: String },
    #[error("metalink error: {0}")]
    Metalink(#[from] MetalinkError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Backend priority for resolving downloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackendPriority {
    Highest = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Fallback = 4,
}

/// A single transport backend (protocol-specific downloader).
#[async_trait]
pub trait TransportBackend: Send + Sync {
    /// The URI scheme this backend handles.
    fn scheme(&self) -> &'static str;

    /// Priority for this backend (lower = tried first).
    fn priority(&self) -> BackendPriority;

    /// Fetch data from a URL, verifying against the expected hash.
    async fn fetch(
        &self,
        url: &str,
        expected_hash: &HashType,
        expected_value: &str,
    ) -> TransportResult<Bytes>;
}

/// The resolver that tries backends in priority order.
pub struct TransportResolver {
    backends: Vec<Box<dyn TransportBackend>>,
}

impl TransportResolver {
    pub fn new() -> Self {
        Self { backends: vec![] }
    }

    /// Add a backend. Backends are sorted by priority on addition.
    pub fn add_backend(&mut self, backend: Box<dyn TransportBackend>) {
        self.backends.push(backend);
        self.backends.sort_by_key(|b| b.priority());
    }

    /// Resolve a Metalink document by trying backends in priority order.
    pub async fn resolve(&self, metalink: &Metalink) -> TransportResult<Bytes> {
        if metalink.file.hashes.is_empty() {
            return Err(TransportError::Metalink(MetalinkError::Validation(
                "no hashes in metalink".into(),
            )));
        }

        let hash = &metalink.file.hashes[0];

        for backend in &self.backends {
            let scheme_urls: Vec<_> = metalink
                .urls_by_scheme(backend.scheme())
                .into_iter()
                .collect();

            for url_entry in &scheme_urls {
                let result = backend
                    .fetch(&url_entry.url, &hash.hash_type, &hash.value)
                    .await;
                if let Ok(data) = result {
                    return Ok(data);
                }
            }
        }

        Err(TransportError::NoBackends)
    }
}

impl Default for TransportResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ln_metalink::{HashType, Metalink};

    /// A mock backend that returns predefined data for testing.
    struct MockBackend {
        scheme: &'static str,
        priority: BackendPriority,
        data_map: std::collections::HashMap<String, Bytes>,
    }

    impl MockBackend {
        fn new(
            scheme: &'static str,
            priority: BackendPriority,
        ) -> Self {
            Self {
                scheme,
                priority,
                data_map: std::collections::HashMap::new(),
            }
        }

        fn with_data(mut self, url: &str, data: &[u8]) -> Self {
            self.data_map.insert(url.to_string(), Bytes::from(data.to_vec()));
            self
        }
    }

    #[async_trait]
    impl TransportBackend for MockBackend {
        fn scheme(&self) -> &'static str {
            self.scheme
        }

        fn priority(&self) -> BackendPriority {
            self.priority
        }

        async fn fetch(
            &self,
            url: &str,
            _hash_type: &HashType,
            _expected_value: &str,
        ) -> TransportResult<Bytes> {
            self.data_map
                .get(url)
                .cloned()
                .ok_or_else(|| TransportError::Backend {
                    backend: self.scheme.to_string(),
                    message: "not found".to_string(),
                })
        }
    }

    #[tokio::test]
    async fn resolver_tries_backends_in_priority_order() {
        let mut resolver = TransportResolver::new();

        let low_prio = MockBackend::new("low", BackendPriority::Fallback)
            .with_data("low://test", b"low-data");
        let high_prio = MockBackend::new("high", BackendPriority::Highest)
            .with_data("high://test", b"high-data");

        resolver.add_backend(Box::new(low_prio));
        resolver.add_backend(Box::new(high_prio));

        let metalink = Metalink::builder("test.txt", 10)
            .add_blake3(&blake3::hash(b"high-data"))
            .add_url("low://test", Some(1), None)
            .add_url("high://test", Some(2), None)
            .build();

        let result = resolver.resolve(&metalink).await.unwrap();
        // Should have fetched from high_prio first since it has Highest priority
        assert_eq!(&result[..], b"high-data");
    }

    #[tokio::test]
    async fn resolver_falls_back_when_first_fails() {
        let mut resolver = TransportResolver::new();

        let failing = MockBackend::new("fail", BackendPriority::Highest);
        let working = MockBackend::new("work", BackendPriority::Fallback)
            .with_data("work://test", b"fallback-data");

        resolver.add_backend(Box::new(failing));
        resolver.add_backend(Box::new(working));

        let metalink = Metalink::builder("test.txt", 10)
            .add_blake3(&blake3::hash(b"fallback-data"))
            .add_url("fail://test", Some(1), None)
            .add_url("work://test", Some(2), None)
            .build();

        let result = resolver.resolve(&metalink).await.unwrap();
        assert_eq!(&result[..], b"fallback-data");
    }

    #[tokio::test]
    async fn resolver_returns_error_when_no_backend_works() {
        let mut resolver = TransportResolver::new();

        let failing = MockBackend::new("fail", BackendPriority::Highest);
        resolver.add_backend(Box::new(failing));

        let metalink = Metalink::builder("test.txt", 10)
            .add_blake3(&blake3::hash(b"anything"))
            .add_url("fail://test", Some(1), None)
            .build();

        let result = resolver.resolve(&metalink).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransportError::NoBackends));
    }

    #[tokio::test]
    async fn resolver_with_http_backend() {
        // Test that the HTTP backend compiles and can be created
        let backend = HttpBackend::new();
        assert_eq!(backend.scheme(), "https");
        assert_eq!(backend.priority(), BackendPriority::Fallback);
    }
}
