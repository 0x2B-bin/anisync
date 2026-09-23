use std::{io::ErrorKind, path::PathBuf};
use thiserror::Error;

use crate::{
    config::{Config, ConfigError},
    state::{AppState, StateError},
};

pub struct AppContext {
    pub config: Config,
    pub state: AppState,
}

#[derive(Error, Debug)]
pub enum ContextError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    State(#[from] StateError),
}

impl AppContext {
    pub fn load() -> Result<Self, ContextError> {
        Ok(Self {
            config: Config::load()?,
            state: AppState::load()?,
        })
    }

    pub fn load_or_setup() -> Result<Self, ContextError> {
        let config = match Config::load() {
            Ok(cfg) => cfg,
            Err(ConfigError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                Config::setup_interactive_default()?
            }
            Err(err) => return Err(err.into()),
        };

        let state = AppState::load()?;

        Ok(AppContext { config, state })
    }

    pub fn save_state(&self) -> Result<(), StateError> {
        self.state.save()
    }

    pub fn save_config(&self) -> Result<(), ConfigError> {
        self.config.serialize()
    }
}
