use clap::{Parser, Subcommand};
use mycontextport_core::{
    collector::Collector,
    collectors::ShellHistoryCollector,
    daemon::{Daemon, DaemonConfig},
};
use mycontextport_store::ContextStore;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "mycontextport",
    version,
    about = "MyContextPort — your personal AI memory layer",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the local context store
    Init {
        /// Run the interactive setup wizard
        #[arg(long)]
        wizard: bool,
        /// Initialize without any prompts (for CI)
        #[arg(long)]
        non_interactive: bool,
    },
    /// Show daemon and store status
    Status,
    /// Inspect what MyContextPort knows about you
    Inspect {
        /// Filter by collector source
        #[arg(long)]
        source: Option<String>,
        /// Show last N items
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Show injection audit log
    Log {
        /// Enable debug output
        #[arg(long)]
        debug: bool,
        /// Show last N entries
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Manage context collectors
    Collector {
        #[command(subcommand)]
        action: CollectorCommands,
    },
    /// Run the MCP server (stdio transport for Claude Desktop)
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },
    /// Developer tools for building collectors
    Dev {
        #[command(subcommand)]
        action: DevCommands,
    },
}

#[derive(Subcommand)]
enum CollectorCommands {
    /// List available and active collectors
    List,
    /// Add and enable a collector
    Add { name: String },
    /// Remove a collector
    Remove { name: String },
    /// Check collector health
    Health { name: Option<String> },
}

#[derive(Subcommand)]
enum McpCommands {
    /// Start the MCP server (stdio transport — used by Claude Desktop)
    Serve {
        #[arg(long, default_value = "8765")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum DevCommands {
    /// Scaffold a new collector
    NewCollector {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "macos,linux")]
        platform: String,
    },
    /// Validate a collector implementation
    TestCollector {
        #[arg(long)]
        collector: std::path::PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing writes to stderr — does not interfere with stdout MCP JSON-RPC channel.
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config = DaemonConfig::default();

    match cli.command {
        Commands::Init { wizard: _, non_interactive: _ } => {
            cmd_init(&config)?;
        }
        Commands::Status => {
            cmd_status(&config)?;
        }
        Commands::Inspect { source, limit } => {
            cmd_inspect(&config, source, limit)?;
        }
        Commands::Log { debug: _, limit: _ } => {
            eprintln!("Log command not yet implemented.");
        }
        Commands::Collector { action: _ } => {
            eprintln!("Collector management not yet implemented.");
        }
        Commands::Mcp { action } => match action {
            McpCommands::Serve { port: _ } => {
                cmd_mcp_serve(&config).await?;
            }
        },
        Commands::Dev { action: _ } => {
            eprintln!("Dev tools not yet implemented.");
        }
    }

    Ok(())
}

fn cmd_init(config: &DaemonConfig) -> anyhow::Result<()> {
    let store = ContextStore::open(&config.store_path)?;
    store.initialize()?;
    println!("Initialized MyContextPort.");
    println!("Store: {}", config.store_path.join("context.db").display());
    println!();
    println!("Next steps:");
    println!("  mycontextport mcp serve     Start collecting context (for Claude Desktop)");
    println!("  mycontextport status        Show store status");
    Ok(())
}

fn cmd_status(config: &DaemonConfig) -> anyhow::Result<()> {
    let db_path = config.store_path.join("context.db");
    if !db_path.exists() {
        println!("MyContextPort is not initialized.");
        println!("Run 'mycontextport init' to get started.");
        return Ok(());
    }
    let store = ContextStore::open(&config.store_path)?;
    store.initialize()?;
    let count = store.count()?;
    println!("Store:  {}", db_path.display());
    println!("Items:  {}", count);
    println!();
    println!("Run 'mycontextport mcp serve' to start collecting context.");
    Ok(())
}

fn cmd_inspect(
    config: &DaemonConfig,
    source: Option<String>,
    limit: usize,
) -> anyhow::Result<()> {
    let db_path = config.store_path.join("context.db");
    if !db_path.exists() {
        println!("Store not found. Run 'mycontextport init' first.");
        return Ok(());
    }
    let store = ContextStore::open(&config.store_path)?;
    store.initialize()?;
    let items = store.query_recent(limit)?;
    let items: Vec<_> = if let Some(src) = &source {
        items.into_iter().filter(|i| &i.source == src).collect()
    } else {
        items
    };
    if items.is_empty() {
        println!("No context items found.");
    } else {
        for item in &items {
            println!("[{}] {}", item.source, item.content);
        }
    }
    Ok(())
}

async fn cmd_mcp_serve(config: &DaemonConfig) -> anyhow::Result<()> {
    // Open (or create) the store and ensure schema exists.
    let store = Arc::new(ContextStore::open(&config.store_path)?);
    store.initialize()?;

    // Build the collector list.
    let collectors: Vec<Box<dyn Collector>> = vec![
        Box::new(ShellHistoryCollector::new()),
    ];

    // Spawn the background collection loop.
    let daemon = Daemon::new(config.clone());
    let store_for_loop = Arc::clone(&store);
    tokio::spawn(async move {
        daemon.run_loop(store_for_loop, collectors).await;
    });

    // Start the MCP stdio server on the main task — blocks until stdin closes.
    mycontextport_mcp::serve_stdio(store).await?;

    Ok(())
}
