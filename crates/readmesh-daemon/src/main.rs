//! Headless node daemon (`readmeshd`).

use std::path::PathBuf;

use clap::Parser;
use readmesh_daemon::service::DaemonService;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
#[command(name = "readmeshd", about = "readmesh headless daemon")]
struct Args {
    /// Data directory
    #[arg(short, long, default_value = "./readmesh-data")]
    data_dir: PathBuf,

    /// Port for the RPC endpoint (0 = stdio mode)
    #[arg(short, long, default_value = "0")]
    rpc_port: u16,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("readmeshd=info".parse()?),
        )
        .init();

    let args = Args::parse();
    tracing::info!(
        "Starting readmeshd with data dir: {}",
        args.data_dir.display()
    );

    // Ensure data directory exists
    std::fs::create_dir_all(&args.data_dir)?;

    let service = DaemonService::new(&args.data_dir).await?;

    if args.rpc_port > 0 {
        // Network RPC mode
        let addr = format!("127.0.0.1:{}", args.rpc_port);
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("RPC server listening on {addr}");

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer)) => {
                            tracing::debug!("Accepted connection from {peer}");
                            tokio::spawn(handle_client(stream));
                        }
                        Err(e) => {
                            tracing::error!("Accept error: {e}");
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Shutting down");
                    break;
                }
            }
        }
    } else {
        // Stdio mode: read JSON lines from stdin, write to stdout
        tracing::info!("Running in stdio mode. Send JSON-RPC requests on stdin.");
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        let mut stdout = stdout;

        loop {
            tokio::select! {
                result = lines.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            if line.trim().is_empty() {
                                continue;
                            }
                            let request: readmesh_rpc::RpcRequest = match serde_json::from_str(&line) {
                                Ok(req) => req,
                                Err(e) => {
                                    let err = readmesh_rpc::RpcResponse::Error {
                                        message: format!("invalid JSON: {e}"),
                                    };
                                    let mut resp = serde_json::to_string(&err).unwrap();
                                    resp.push('\n');
                                    let _ = stdout.write_all(resp.as_bytes()).await;
                                    continue;
                                }
                            };
                            let response = service.handle(request).await;
                            let mut resp = serde_json::to_string(&response).unwrap();
                            resp.push('\n');
                            let _ = stdout.write_all(resp.as_bytes()).await;
                        }
                        Ok(None) => break,
                        Err(e) => {
                            tracing::error!("stdin read error: {e}");
                            break;
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Shutting down");
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn handle_client(stream: tokio::net::TcpStream) {
    let _ = stream;
    // TODO: implement TCP RPC client handler
}
