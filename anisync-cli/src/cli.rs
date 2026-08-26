use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "CLI tool for anisyncd")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Auth {
        #[command(subcommand)]
        command: AuthCommands
    }
}

#[derive(Subcommand, Debug)]
pub enum AuthCommands {
    Setup,
    Login
}
