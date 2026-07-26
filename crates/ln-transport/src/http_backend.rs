use async_trait::async_trait;
use bytes::Bytes;
use ln_metalink::HashType;

use super::{BackendPriority, TransportBackend, TransportError, TransportResult};

/// HTTP/HTTPS transport backend using reqwest.
pub struct HttpBackend {
    client: reqwest::Client,
}

impl HttpBackend {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn verify_hash(data: &[u8], hash_type: &HashType, expected: &str) -> TransportResult<()> {
        let computed = match hash_type {
            HashType::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data);
                hex::encode(hasher.finalize())
            }
            HashType::Blake3 => blake3::hash(data).to_hex().to_string(),
            HashType::Other(_) => return Ok(()),
        };
        if computed != expected {
            return Err(TransportError::HashMismatch {
                expected: expected.to_string(),
                got: computed,
            });
        }
        Ok(())
    }
}

impl Default for HttpBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TransportBackend for HttpBackend {
    fn scheme(&self) -> &'static str {
        "https"
    }

    fn priority(&self) -> BackendPriority {
        BackendPriority::Fallback
    }

    async fn fetch(
        &self,
        url: &str,
        hash_type: &HashType,
        expected: &str,
    ) -> TransportResult<Bytes> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TransportError::Backend {
                backend: "http".into(),
                message: e.to_string(),
            })?;

        let data = response
            .bytes()
            .await
            .map_err(|e| TransportError::Backend {
                backend: "http".into(),
                message: e.to_string(),
            })?;

        Self::verify_hash(&data, hash_type, expected)?;

        Ok(data)
    }
}
