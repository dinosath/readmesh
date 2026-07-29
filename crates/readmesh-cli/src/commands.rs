//! CLI command implementations.

use crate::client::DaemonClient;
use readmesh_rpc::{RpcRequest, RpcResponse};

/// List all novels in the library.
pub async fn cmd_library(client: &DaemonClient) -> anyhow::Result<()> {
    match client.request(RpcRequest::GetLibrary).await {
        RpcResponse::Library { novels } => {
            if novels.is_empty() {
                println!("Library is empty.");
                println!("Use 'readmesh-cli search <plugin> <query>' to find novels.");
                return Ok(());
            }
            println!("Library ({novels_len} novels):", novels_len = novels.len());
            for novel in &novels {
                let status = match novel.status {
                    readmesh_core::novel::NovelStatus::Ongoing => "[ongoing]",
                    readmesh_core::novel::NovelStatus::Completed => "[completed]",
                    readmesh_core::novel::NovelStatus::Hiatus => "[hiatus]",
                    readmesh_core::novel::NovelStatus::Dropped => "[dropped]",
                    readmesh_core::novel::NovelStatus::Unknown => "[unknown]",
                };
                let authors: Vec<String> = novel.authors.iter().map(|a| a.name.clone()).collect();
                println!("  {} - {}  {status}", novel.id, novel.title,);
                if !authors.is_empty() {
                    println!("    By: {}", authors.join(", "));
                }
                if let Some(ref summary) = novel.summary {
                    let short: String = summary.chars().take(120).collect();
                    println!(
                        "    {short}{}",
                        if summary.len() > 120 { "..." } else { "" }
                    );
                }
            }
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// List chapters for a novel.
pub async fn cmd_chapters(client: &DaemonClient, novel_id_hex: &str) -> anyhow::Result<()> {
    let novel_id_bytes = hex::decode(novel_id_hex)?;
    let response = client
        .request(RpcRequest::GetChapters {
            novel_id: novel_id_bytes,
        })
        .await;

    match response {
        RpcResponse::Chapters { chapters } => {
            if chapters.is_empty() {
                println!("No chapters found.");
                return Ok(());
            }
            println!("Chapters ({}):", chapters.len());
            for ch in &chapters {
                let read = " ";
                println!("  [{read}] Ch {}: {}  ({})", ch.index, ch.title, ch.id);
            }
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// Read a chapter's content.
pub async fn cmd_read(client: &DaemonClient, chapter_id_hex: &str) -> anyhow::Result<()> {
    let chapter_id_bytes = hex::decode(chapter_id_hex)?;
    let response = client
        .request(RpcRequest::GetChapterContent {
            chapter_id: chapter_id_bytes,
            metalink_hash: None,
        })
        .await;

    match response {
        RpcResponse::ChapterContent { data } => {
            let text = String::from_utf8_lossy(&data);
            // Simple HTML stripping
            let clean = strip_html(&text);
            println!("{clean}");
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// Search for novels using a plugin.
pub async fn cmd_search(client: &DaemonClient, plugin_id: &str, query: &str) -> anyhow::Result<()> {
    let response = client
        .request(RpcRequest::Search {
            plugin_id: plugin_id.to_string(),
            query: query.to_string(),
            page: 1,
        })
        .await;

    match response {
        RpcResponse::SearchResults { results } => {
            if results.is_empty() {
                println!("No results found for '{query}'.");
                return Ok(());
            }
            println!(
                "Search results for '{query}' ({results_len}):",
                results_len = results.len()
            );
            for novel in &results {
                println!("  {} - {}", novel.id, novel.title);
                if let Some(ref summary) = novel.summary {
                    let short: String = summary.chars().take(100).collect();
                    println!(
                        "    {short}{}",
                        if summary.len() > 100 { "..." } else { "" }
                    );
                }
            }
            println!();
            println!("To add a novel to your library, use the daemon RPC directly.");
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// List installed plugins.
pub async fn cmd_plugins(client: &DaemonClient) -> anyhow::Result<()> {
    let response = client.request(RpcRequest::ListPlugins).await;

    match response {
        RpcResponse::Plugins { plugins } => {
            if plugins.is_empty() {
                println!("No plugins installed.");
                return Ok(());
            }
            println!("Installed plugins:");
            for plugin in &plugins {
                println!("  {} v{}", plugin.name, plugin.version);
                println!("    ID: {}", plugin.id);
                println!("    Sites: {}", plugin.supported_sites.join(", "));
            }
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// Start the daemon in foreground (no user interaction, just runs).
pub async fn cmd_daemon(_data_dir: &str) -> anyhow::Result<()> {
    println!("Daemon mode: run 'readmeshd' directly instead.");
    println!("  cargo run --bin readmeshd -- --data-dir ./readmesh-data");
    Ok(())
}

/// Show federation and network status.
pub async fn cmd_federation(client: &DaemonClient) -> anyhow::Result<()> {
    let response = client.request(RpcRequest::GetFederationStatus).await;

    match response {
        RpcResponse::FederationStatus { status } => {
            let mc = &status.mirror_config;
            println!("=== Federation Status ===");
            println!("Node ID: {} bytes", client.device_id().len());
            println!();
            println!("Mirror config:");
            println!("  Enabled: {}", mc.enabled);
            println!("  Storage cap: {} bytes", mc.storage_cap_bytes);
            println!("  Max chapters: {}", mc.max_chapters);
            println!("  Only followed: {}", mc.only_followed);
            println!();
            println!("Stat:");
            println!(
                "  Mirrored: {} bytes / {} chapters",
                status.mirrored_bytes, status.mirrored_chapters
            );
            println!();
            println!("Followed peers ({}):", status.followed_peers.len());
            for peer in &status.followed_peers {
                let alias = peer.alias.as_deref().unwrap_or("<no alias>");
                println!("  {} - {alias}", peer.node_id);
                println!("    Since: {}", peer.since);
                println!("    Auto-mirror: {}", peer.auto_mirror);
            }
            println!();
            println!("Known peers: {}", status.known_peers.len());
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// Follow a peer node.
pub async fn cmd_follow_peer(
    client: &DaemonClient,
    node_id_hex: &str,
    alias: Option<&str>,
) -> anyhow::Result<()> {
    let node_id_bytes = hex::decode(node_id_hex)?;
    let response = client
        .request(RpcRequest::FollowPeer {
            node_id: node_id_bytes,
            alias: alias.map(|s| s.to_string()),
        })
        .await;

    match response {
        RpcResponse::Ok => {
            if let Some(alias) = alias {
                println!("Now following peer '{alias}' ({node_id_hex})");
            } else {
                println!("Now following peer {node_id_hex}");
            }
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// Stop following a peer node.
pub async fn cmd_unfollow_peer(client: &DaemonClient, node_id_hex: &str) -> anyhow::Result<()> {
    let node_id_bytes = hex::decode(node_id_hex)?;
    let response = client
        .request(RpcRequest::UnfollowPeer {
            node_id: node_id_bytes,
        })
        .await;

    match response {
        RpcResponse::Ok => {
            println!("Unfollowed peer {node_id_hex}");
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// Create a new novel project via the authoring crate.
pub async fn cmd_create_project(client: &DaemonClient, title: &str) -> anyhow::Result<()> {
    match client.request(RpcRequest::CreateProject {
        title: title.to_string(),
    })
    .await
    {
        RpcResponse::ProjectData { data } => {
            println!("Created project '{title}' ({} bytes CRDT data)", data.len());
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// Import a novel from a website using a plugin.
pub async fn cmd_import(
    client: &DaemonClient,
    plugin_id: &str,
    url: &str,
) -> anyhow::Result<()> {
    println!("Importing from {url} using plugin '{plugin_id}'...");
    match client
        .request(RpcRequest::ImportFromSource {
            plugin_id: plugin_id.to_string(),
            url: url.to_string(),
        })
        .await
    {
        RpcResponse::Novel { novel: Some(n) } => {
            println!("Imported: {} ({})", n.title, n.id);
        }
        RpcResponse::Novel { novel: None } => {
            eprintln!("Novel not found at {url}");
        }
        RpcResponse::Error { message } => {
            eprintln!("Error: {message}");
        }
        _ => {
            eprintln!("Unexpected response");
        }
    }
    Ok(())
}

/// Simple HTML tag stripper.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    // Replace common entities
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&quot;", "\"");
    result = result.replace("&apos;", "'");
    result
}
