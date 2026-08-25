use std::fmt::Result;

use serde::Deserialize;
use anyhow::{Context, Result};

#[derive(Deserialize, Debug)]
struct Config {
    auth: Auth
}

#[derive(Deserialize, Debug)]
struct Auth {
    client_id: String,
    client_secret: String
}

impl Config {
    fn load() -> Result<Self> {
        let config_dir = dirs::config_dir().context("Unable to locate config directory")?;
        ()
    }
}
