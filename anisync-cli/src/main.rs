use clap::Parser;
use color_eyre::eyre::Result;
use oauth2::{ClientId, RedirectUrl, basic::BasicClient};

use crate::{
    cli::{Cli, Commands},
    config::Config,
};

mod cli;
mod config;

//fn get_token() {
//let client = BasicClient::new(ClientId::new("asda".to_string()))
//.set_redirect_uri(RedirectUrl::new("".to_string()));
//}

fn run_command(cli: &Cli) {
    match cli.command {
        Commands::Auth => match Config::load() {
            Ok(config) => println!("{}", config.myanimelist.client_id),
            Err(err) => println!("{}", err),
        },
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    run_command(&cli);
    Ok(())
}
