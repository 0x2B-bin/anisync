use clap::Parser;
use color_eyre::eyre::Result;

use crate::cli::{AuthCommands, Cli, Commands};
use anisync_lib::config::{Config, ConfigError};

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
        },
        Err(err) => Err(err.into())
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
            println!("Command Sent");
        }
    };

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    run_command(&cli)
}
