mod app;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cc-switch-tui")]
#[command(about = "CC-Switch Terminal UI", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive TUI
    Tui,
    /// Run as background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Proxy commands
    Proxy {
        #[command(subcommand)]
        action: ProxyAction,
    },
    /// Provider commands
    Provider {
        #[command(subcommand)]
        action: ProviderAction,
    },
}

#[derive(Subcommand)]
#[derive(Debug)]
enum DaemonAction {
    Start,
    Stop,
    Status,
    Restart,
}

#[derive(Subcommand)]
#[derive(Debug)]
enum ProxyAction {
    Start,
    Stop,
    Status,
}

#[derive(Subcommand)]
#[derive(Debug)]
enum ProviderAction {
    List,
    Switch { id: String },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Tui) => {
            // Start interactive TUI
            app::run_tui()?;
        }
        Some(Commands::Daemon { action }) => {
            println!("Daemon mode: {:?} - not yet implemented", action);
            // TODO: daemon::handle_daemon_command(action)?;
        }
        Some(Commands::Proxy { action }) => {
            println!("Proxy command: {:?} - not yet implemented", action);
            // TODO: proxy::handle_proxy_command(action)?;
        }
        Some(Commands::Provider { action }) => {
            println!("Provider command: {:?} - not yet implemented", action);
            // TODO: provider::handle_provider_command(action)?;
        }
    }

    Ok(())
}
