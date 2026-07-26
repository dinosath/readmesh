//! Client for communicating with the daemon.
//!
//! In v0.1, the client embeds the daemon in-process.
//! Later this will support connecting to a remote daemon over TCP.

use ln_rpc::{RpcRequest, RpcResponse};

/// A handle to the daemon service, embedded in-process.
pub struct DaemonClient {
    service: ln_daemon::service::DaemonService,
}

impl DaemonClient {
    /// Create a client by embedding the daemon in-process.
    pub async fn embed(data_dir: &str) -> anyhow::Result<Self> {
        let service = ln_daemon::service::DaemonService::new(
            std::path::Path::new(data_dir),
        )
        .await?;
        Ok(Self { service })
    }

    /// Send an RPC request and return the response.
    pub async fn request(&self, request: RpcRequest) -> RpcResponse {
        self.service.handle(request).await
    }

    /// Get the node ID of the embedded daemon.
    pub fn device_id(&self) -> Vec<u8> {
        // This is the default placeholder — in production this would come from the net layer
        vec![0u8; 32]
    }
}
