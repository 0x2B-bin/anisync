use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use oauth2::{ClientId, RedirectUrl, basic::BasicClient};

use crate::{
    cli::{AuthCommands, Cli, Commands},
    config::Config,
};

mod cli;
mod config;
mod oauth;

//fn get_token() {
//let client = BasicClient::new(ClientId::new("asda".to_string()))
//.set_redirect_uri(RedirectUrl::new("".to_string()));
//}

fn run_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Auth { command } => match command {
            AuthCommands::Setup => Config::setup_interactive_default().map(|_| ()),
            AuthCommands::Login => {
                let config = Config::load()?;
                oauth::run(&config)
            }
        },
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    run_command(&cli)
}
