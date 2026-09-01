use clap::Parser;
use color_eyre::eyre::Result;

use crate::{
    cli::{AuthCommands, Cli, Commands},
    config::Config,
};

mod cli;
mod config;
mod oauth;
mod msg;

fn run_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Auth { command } => match command {
            AuthCommands::Setup => Config::setup_interactive_default().map(|_| ())?,
            AuthCommands::Login => {
                let mut config = Config::load()?;
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
