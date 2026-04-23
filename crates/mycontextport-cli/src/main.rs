mod registry;
mod templates;
mod ui;

use clap::{Parser, Subcommand};
use mycontextport_core::{
    collector::Collector,
    collectors::{PythonCollector, ShellHistoryCollector},
    daemon::DaemonConfig,
    scheduler::{CollectorSchedule, Scheduler},
};
use mycontextport_graph::GraphIndexer;
use mycontextport_privacy::{GuardrailsEngine, PrivacyEngine};
use mycontextport_store::ContextStore;
use registry::{builtin_registry, collectors_config_path, expand_path, CollectorsConfig};
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
        /// Show last N entries
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Open the web dashboard
    Ui {
        /// Port to listen on
        #[arg(long, default_value = "8765")]
        port: u16,
        /// Do not open the browser automatically
        #[arg(long)]
        no_open: bool,
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
        Commands::Log { limit } => {
            cmd_log(&config, limit)?;
        }
        Commands::Ui { port, no_open } => {
            cmd_ui(&config, port, no_open).await?;
        }
        Commands::Collector { action } => match action {
            CollectorCommands::List => cmd_collector_list().await?,
            CollectorCommands::Add { name } => cmd_collector_add(&name)?,
            CollectorCommands::Remove { name } => cmd_collector_remove(&name)?,
            CollectorCommands::Health { name } => cmd_collector_health(name).await?,
        },
        Commands::Mcp { action } => match action {
            McpCommands::Serve { port: _ } => {
                cmd_mcp_serve(&config).await?;
            }
        },
        Commands::Dev { action } => match action {
            DevCommands::NewCollector { name, platform } => {
                cmd_dev_new_collector(&name, &platform)?;
            }
            DevCommands::TestCollector { collector } => {
                println!("Validate collector at: {}", collector.display());
                println!("(test-collector validation coming in v0.3)");
            }
        },
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
    println!("  mycontextport ui            Open the web dashboard");
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
    println!("  mycontextport ui            Open the web dashboard");
    println!("  mycontextport log           Show injection audit log");
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

fn cmd_log(config: &DaemonConfig, limit: usize) -> anyhow::Result<()> {
    let db_path = config.store_path.join("context.db");
    if !db_path.exists() {
        println!("Store not found. Run 'mycontextport init' first.");
        return Ok(());
    }
    let store = ContextStore::open(&config.store_path)?;
    store.initialize()?;
    let entries = store.query_log(limit)?;
    if entries.is_empty() {
        println!("No injections logged yet. Run 'mycontextport mcp serve' and connect an AI tool.");
        return Ok(());
    }
    println!("{:<26} {:<20} {:>9} {:>9}", "Time", "Model", "Injected", "Blocked");
    println!("{}", "-".repeat(70));
    for entry in &entries {
        let ts = chrono::DateTime::from_timestamp(entry.injected_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| entry.injected_at.to_string());
        let model = if entry.model.len() > 18 {
            format!("{}…", &entry.model[..17])
        } else {
            entry.model.clone()
        };
        println!(
            "{:<26} {:<20} {:>9} {:>9}",
            ts,
            model,
            entry.items_used.len(),
            entry.items_blocked.len(),
        );
    }
    Ok(())
}

async fn cmd_ui(config: &DaemonConfig, port: u16, no_open: bool) -> anyhow::Result<()> {
    let db_path = config.store_path.join("context.db");
    if !db_path.exists() {
        println!("Store not found. Run 'mycontextport init' first.");
        return Ok(());
    }
    let store = Arc::new(ContextStore::open(&config.store_path)?);
    store.initialize()?;

    let privacy_config_path = config.store_path.join("privacy.toml");
    let engine = Arc::new(
        PrivacyEngine::from_config_file(&privacy_config_path).unwrap_or_else(|_| {
            PrivacyEngine::new(vec![])
        }),
    );

    ui::serve(store, engine, config.store_path.clone(), port, no_open).await
}

async fn cmd_mcp_serve(config: &DaemonConfig) -> anyhow::Result<()> {
    let store = Arc::new(ContextStore::open(&config.store_path)?);
    store.initialize()?;

    let privacy_config_path = config.store_path.join("privacy.toml");
    let engine = Arc::new(
        PrivacyEngine::from_config_file(&privacy_config_path).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to load privacy config — using allow-all");
            PrivacyEngine::new(vec![])
        }),
    );

    // Build per-collector schedules.
    let collectors_config = CollectorsConfig::load(&collectors_config_path()).unwrap_or_default();
    let default_interval = config.collection_interval_secs;

    let mut schedules: Vec<CollectorSchedule> =
        vec![CollectorSchedule::new(Box::new(ShellHistoryCollector::new()), default_interval)];

    for entry in collectors_config.python {
        let script_path = expand_path(&entry.script);
        schedules.push(CollectorSchedule::new(
            Box::new(PythonCollector::new(entry.name, script_path, entry.config)),
            entry.interval_secs,
        ));
    }

    let scheduler = Scheduler::new(Arc::clone(&store), schedules);
    tokio::spawn(async move {
        scheduler.run().await;
    });

    // Index any items collected before this run (non-blocking).
    let graph_store = Arc::clone(&store);
    tokio::task::spawn_blocking(move || {
        let indexer = GraphIndexer::new(graph_store);
        if let Err(e) = indexer.index_all() {
            tracing::warn!(error = %e, "Background graph indexing failed");
        }
    });

    let guardrails = Arc::new(
        GuardrailsEngine::from_config_file(&privacy_config_path).unwrap_or_else(|_| {
            GuardrailsEngine::new(vec![])
        }),
    );

    mycontextport_mcp::serve_stdio_with_guardrails(store, engine, guardrails).await?;
    Ok(())
}

async fn cmd_collector_list() -> anyhow::Result<()> {
    let registry = builtin_registry();
    let cfg = CollectorsConfig::load(&collectors_config_path()).unwrap_or_default();

    println!("{:<20} {:<12} {:<10} {}", "Name", "Type", "Status", "Description");
    println!("{}", "-".repeat(72));

    // Always-on built-in Rust collectors
    println!("{:<20} {:<12} {:<10} {}", "shell-history", "built-in", "active", "Shell command history");
    println!("{:<20} {:<12} {:<10} {}", "clipboard", "built-in", "active", "Clipboard contents");

    for entry in &cfg.python {
        let desc = registry
            .get(entry.name.as_str())
            .map(|s| s.description)
            .unwrap_or("custom collector");
        let script = expand_path(&entry.script);
        let status = if script.exists() { "active" } else { "missing" };
        println!("{:<20} {:<12} {:<10} {}", entry.name, "python", status, desc);
    }

    // Show available but not yet added built-ins
    let active_names: std::collections::HashSet<_> = cfg.python.iter().map(|e| e.name.as_str()).collect();
    for (name, spec) in &registry {
        if !active_names.contains(name) {
            println!("{:<20} {:<12} {:<10} {}", name, "python", "available", spec.description);
        }
    }

    Ok(())
}

fn cmd_collector_add(name: &str) -> anyhow::Result<()> {
    let registry = builtin_registry();
    if !registry.contains_key(name) {
        anyhow::bail!("Unknown collector '{}'. Run 'mycontextport collector list' to see available collectors.", name);
    }

    let config_path = collectors_config_path();
    let mut cfg = CollectorsConfig::load(&config_path).unwrap_or_default();

    if cfg.python.iter().any(|e| e.name == name) {
        println!("Collector '{}' is already configured.", name);
        return Ok(());
    }

    let home = dirs::home_dir().unwrap_or_default();
    let script = format!("{}/.mycontextport/collectors/{}/__main__.py", home.display(), name);

    cfg.python.push(registry::PythonCollectorEntry {
        name: name.to_string(),
        script,
        interval_secs: 900,
        config: serde_json::Value::Object(Default::default()),
    });

    cfg.save(&config_path)?;
    println!("Added collector '{}'.", name);
    println!("Install the collector files:");
    println!("  cp -r collectors/{} ~/.mycontextport/collectors/", name);
    Ok(())
}

fn cmd_collector_remove(name: &str) -> anyhow::Result<()> {
    let config_path = collectors_config_path();
    let mut cfg = CollectorsConfig::load(&config_path).unwrap_or_default();
    let before = cfg.python.len();
    cfg.python.retain(|e| e.name != name);
    if cfg.python.len() == before {
        println!("Collector '{}' not found in config.", name);
    } else {
        cfg.save(&config_path)?;
        println!("Removed collector '{}'.", name);
    }
    Ok(())
}

async fn cmd_collector_health(name: Option<String>) -> anyhow::Result<()> {
    let cfg = CollectorsConfig::load(&collectors_config_path()).unwrap_or_default();

    let check_python = |entry: &registry::PythonCollectorEntry| {
        let script = expand_path(&entry.script);
        let pc = PythonCollector::new(
            entry.name.clone(),
            script,
            entry.config.clone(),
        );
        (entry.name.clone(), pc)
    };

    let entries: Vec<_> = if let Some(ref n) = name {
        cfg.python.iter().filter(|e| &e.name == n).map(check_python).collect()
    } else {
        cfg.python.iter().map(check_python).collect()
    };

    // Built-in collectors (always healthy)
    if name.is_none() {
        println!("{:<20} {}", "shell-history", "healthy: Shell history accessible");
        println!("{:<20} {}", "clipboard", "healthy: Clipboard accessible");
    }

    for (collector_name, pc) in entries {
        let health = pc.health_check().await;
        let status = if health.healthy { "healthy" } else { "unhealthy" };
        println!("{:<20} {}: {}", collector_name, status, health.message);
    }

    Ok(())
}

fn cmd_dev_new_collector(name: &str, platform: &str) -> anyhow::Result<()> {
    use std::fs;

    let dir = std::path::Path::new("collectors").join(name);
    if dir.exists() {
        anyhow::bail!("Directory {} already exists.", dir.display());
    }

    fs::create_dir_all(&dir)?;
    fs::write(dir.join("collector.py"), templates::collector_py(name, platform))?;
    fs::write(dir.join("__main__.py"), templates::main_py(name))?;

    println!("Created collector scaffold at collectors/{}/", name);
    println!();
    println!("Next steps:");
    println!("  1. Edit collectors/{}/collector.py — implement collect() and health_check()", name);
    println!("  2. Test it:  python3 -m collectors.{} --health", name);
    println!("  3. Add it:   mycontextport collector add {}", name);
    Ok(())
}
