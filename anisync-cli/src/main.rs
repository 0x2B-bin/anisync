use clap::Parser;
use color_eyre::eyre::{Result, eyre};
use oauth2::{ClientId, RedirectUrl, basic::BasicClient};

use crate::{
    cli::{AuthCommands, Cli, Commands}, config::Config,
};

mod cli;
mod config;

//fn get_token() {
//let client = BasicClient::new(ClientId::new("asda".to_string()))
//.set_redirect_uri(RedirectUrl::new("".to_string()));
//}

fn run_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Commands::Auth { command } => match command {
            AuthCommands::Setup => {
                let mut config_dir = dirs::config_dir().ok_or(eyre!("Unable to locate config directory"))?;
                config_dir.push("anisync");
                let config_file_path = config_dir.join("config.toml");
                Config::setup_interactive(&config_dir, &config_file_path).map(|_| ())
            }
            AuthCommands::Login => unimplemented!()
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    if let Err(err) = run_command(&cli) {
        println!("{err}");
        std::process::exit(1)
    }
    Ok(())
}
