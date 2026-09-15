use std::os::unix::net::UnixStream;

use chrono::{DateTime, Utc};
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr, eyre};

use crate::cli::{AuthCommands, Cli, Commands};
use anisync_lib::{config::{Config, ConfigError}, ipc::{IpcCommand, IpcResponse}};

mod cli;
mod msg;
mod oauth;

fn load_or_setup_config() -> Result<Config> {
    match Config::load() {
        Ok(config) => Ok(config),
        Err(ConfigError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            println!("Config does not exist, let's make one!");
            let config = Config::setup_interactive_default()?;
            Ok(config)
        }
        Err(err) => Err(err.into()),
    }
}

fn run_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Auth { command } => match command {
            AuthCommands::Setup => Config::setup_interactive_default().map(|_| ())?,
            AuthCommands::Login => {
                let mut config = load_or_setup_config()?;
                oauth::run(&mut config)?;
            }
        },
        Commands::Msg { command } => {
            msg::run(command)?;
        },
        Commands::Status => {
            let socket =
                UnixStream::connect("/tmp/anisync.sock").wrap_err("Failed to connect to anisync socket")?;
            IpcCommand::Status.send_command(&socket)
                .map_err(|e| eyre!(e))?;
            let response = IpcResponse::recv_response(&socket).map_err(|e| eyre!(e))?;

            match response {
                IpcResponse::Status(info) => {
                    println!("Status: {}\nLast Sync: {}", match info.working {
                        true => "\x1b[92mSyncing\x1b[0m",
                        false => "\x1b[90mSleeping\x1b[0m"
                    },
                    match info.last_sync {
                        Some(time) => {
                            let datetime: DateTime<Utc> = time.into();
                            datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                        },
                        None => "None".to_string()
                    });
                },
                _ => println!("Unexpected response")
            }
        }
    };

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    run_command(&cli)
}
