use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about = "CLI tool for anisyncd")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Login,
    Setup,

    Msg {
        #[command(subcommand)]
        command: IpcMsg,
    },
    Status,
}

#[derive(Subcommand, Debug)]
pub enum IpcMsg {
    Sync,
}
