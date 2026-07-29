mod client;
mod commands;

use clap::{Parser, Subcommand};
use client::DaemonClient;
use commands::{
    cmd_chapters, cmd_create_project, cmd_daemon, cmd_federation, cmd_follow_peer, cmd_import,
    cmd_library, cmd_plugins, cmd_read, cmd_search, cmd_unfollow_peer,
};

#[derive(Parser)]
#[command(name = "readmesh-cli", about = "ReadMesh CLI client")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to the daemon data directory
    #[arg(short, long, default_value = "./readmesh-data")]
    data_dir: String,
}

#[derive(Subcommand)]
enum Command {
    /// Show the library (all tracked novels)
    Library,
    /// Show chapters for a novel
    Chapters {
        /// Novel ID in hex
        novel_id: String,
    },
    /// Read a chapter (fetch and display content)
    Read {
        /// Chapter ID in hex
        chapter_id: String,
    },
    /// Search for novels using a plugin
    Search {
        /// Plugin ID to use (e.g. "reference-plugin")
        plugin_id: String,
        /// Search query
        query: String,
    },
    /// List installed plugins
    Plugins,
    /// Show federation/network status
    Federation,
    /// Follow a peer node
    Follow {
        /// Node ID in hex
        node_id: String,
        /// Optional alias
        #[arg(short, long)]
        alias: Option<String>,
    },
    /// Stop following a peer node
    Unfollow {
        /// Node ID in hex
        node_id: String,
    },
    /// Show daemon info
    Daemon,
    /// Create a new novel project
    CreateProject {
        /// Novel title
        title: String,
    },
    /// Import a novel from a website
    Import {
        /// Plugin ID to use (e.g. "reference-plugin")
        plugin_id: String,
        /// Novel URL to import
        url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("readmesh_cli=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Daemon => {
            cmd_daemon(&cli.data_dir).await?;
        }
        Command::Library => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_library(&client).await?;
        }
        Command::Chapters { novel_id } => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_chapters(&client, &novel_id).await?;
        }
        Command::Read { chapter_id } => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_read(&client, &chapter_id).await?;
        }
        Command::Search { plugin_id, query } => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_search(&client, &plugin_id, &query).await?;
        }
        Command::Plugins => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_plugins(&client).await?;
        }
        Command::Federation => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_federation(&client).await?;
        }
        Command::Follow { node_id, alias } => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_follow_peer(&client, &node_id, alias.as_deref()).await?;
        }
        Command::Unfollow { node_id } => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_unfollow_peer(&client, &node_id).await?;
        }
        Command::CreateProject { title } => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_create_project(&client, &title).await?;
        }
        Command::Import { plugin_id, url } => {
            let client = DaemonClient::embed(&cli.data_dir).await?;
            cmd_import(&client, &plugin_id, &url).await?;
        }
    }

    Ok(())
}
