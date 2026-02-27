use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "contextgenos",
    version,
    about = "ContextGenOS — your personal AI memory layer",
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
    /// Inspect what ContextGenOS knows about you
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
    /// Run the MCP server
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
    /// Start the MCP server
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
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { wizard, non_interactive } => {
            println!("Initializing ContextGenOS...");
            let _ = (wizard, non_interactive);
            // TODO: implement init
        }
        Commands::Status => {
            println!("ContextGenOS status:");
            // TODO: implement status
        }
        Commands::Inspect { source, limit } => {
            let _ = (source, limit);
            // TODO: implement inspect
        }
        Commands::Log { debug, limit } => {
            let _ = (debug, limit);
            // TODO: implement log
        }
        Commands::Collector { action } => {
            let _ = action;
            // TODO: implement collector commands
        }
        Commands::Mcp { action } => {
            let _ = action;
            // TODO: start MCP server
        }
        Commands::Dev { action } => {
            let _ = action;
            // TODO: implement dev tools
        }
    }

    Ok(())
}
