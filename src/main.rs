use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use agentos_tui::app::Config;
use agentos_tui::ui::run_app;

#[derive(Parser)]
#[command(name = "agentos-tui")]
#[command(author, version, about, long_about = None)]
#[command(about = "AgentOS Terminal - Mission control for AI agent orchestration")]
struct Cli {
    /// Polling interval (milliseconds)
    #[arg(short, long, default_value = "500", value_name = "MS")]
    poll_interval: u64,

    /// Lines to capture from each pane
    #[arg(short, long, default_value = "100", value_name = "LINES")]
    capture_lines: u32,

    /// Config file path
    #[arg(short = 'f', long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// AgentOS API URL (default: http://localhost:3100)
    #[arg(long, default_value = "http://localhost:3100", value_name = "URL")]
    agentos_url: String,

    /// Write debug logs to agentos-tui.log
    #[arg(short, long)]
    debug: bool,

    /// Show config file path
    #[arg(long)]
    show_config_path: bool,

    /// Generate default config file
    #[arg(long)]
    init_config: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Show config path and exit
    if cli.show_config_path {
        if let Some(path) = Config::default_path() {
            println!("{}", path.display());
        } else {
            println!("Config directory not found");
        }
        return Ok(());
    }

    // Initialize config file and exit
    if cli.init_config {
        let config = Config::default();
        if let Err(e) = config.save() {
            eprintln!("Failed to create config: {}", e);
            std::process::exit(1);
        }
        if let Some(path) = Config::default_path() {
            println!("Config created: {}", path.display());
        }
        return Ok(());
    }

    // Setup logging
    if cli.debug {
        let log_file = std::fs::File::create("agentos-tui.log")?;
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(log_file)
            .with_ansi(false);

        tracing_subscriber::registry()
            .with(file_layer)
            .with(tracing_subscriber::filter::LevelFilter::DEBUG)
            .init();
    }

    // Load config (from file or CLI args)
    let mut config = if let Some(config_path) = &cli.config {
        Config::load_from(config_path).unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        })
    } else {
        Config::load()
    };

    // CLI args override config file
    config.poll_interval_ms = cli.poll_interval;
    config.capture_lines = cli.capture_lines;
    config.agentos_url = Some(cli.agentos_url);

    // Run the application
    run_app(config).await
}
