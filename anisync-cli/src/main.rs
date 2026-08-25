use clap::Parser;
use oauth2::{ClientId, RedirectUrl, basic::BasicClient};

use crate::cli::{Cli, Commands};

mod cli;
mod config;

fn get_token() {
    let client = BasicClient::new(
        ClientId::new("asda".to_string())
    )
    .set_redirect_uri(RedirectUrl::new("".to_string()));
}

fn run_command(cli: &Cli) {
    match cli.command {
        Commands::Auth => {}
    }
}

fn main() {
    let cli = Cli::parse();
}
