use std::os::unix::net::UnixStream;

use chrono::{DateTime, Utc};
use clap::Parser;
use color_eyre::eyre::{Result, WrapErr, eyre};

use crate::cli::{Cli, Commands};
use anisync_lib::{
    config::Config,
    context::AppContext,
    ipc::{IpcCommand, IpcResponse},
};

mod cli;
mod msg;
mod oauth;

fn run_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Setup => Config::setup_interactive_default().map(|_| ())?,
        Commands::Login => {
            let mut ctx = AppContext::load_or_setup()?;
            oauth::run(&mut ctx)?;
        }
        Commands::Msg { command } => {
            msg::run(command)?;
        }
        Commands::Status => {
            let socket = UnixStream::connect("/tmp/anisync.sock")
                .wrap_err("Failed to connect to anisync socket")?;
            IpcCommand::Status
                .send_command(&socket)
                .map_err(|e| eyre!(e))?;
            let response = IpcResponse::recv_response(&socket).map_err(|e| eyre!(e))?;

            match response {
                IpcResponse::Status(info) => {
                    println!(
                        "Status: {}\nLast Sync: {}",
                        match info.working {
                            true => "\x1b[92mSyncing\x1b[0m",
                            false => "\x1b[90mSleeping\x1b[0m",
                        },
                        match info.last_sync {
                            Some(time) => {
                                let datetime: DateTime<Utc> = time.into();
                                datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                            }
                            None => "None".to_string(),
                        }
                    );
                }
                _ => println!("Unexpected response"),
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
