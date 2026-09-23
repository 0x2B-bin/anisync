use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct AppState {
    pub myanimelist: TokenSet,
    pub anilist: TokenSet,
}

#[derive(Deserialize, Serialize, Default, Debug, PartialEq)]
pub struct TokenSet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Unable to locate system config directory")]
    MissingStateDir,

    #[error("Failed to read/write state file at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse state.json: {0}")]
    Parse(#[from] serde_json::Error),
}

impl AppState {
    fn defaut_path() -> Result<PathBuf, StateError> {
        dirs::state_dir()
            .map(|p| p.join("anisync").join("state.json"))
            .ok_or(StateError::MissingStateDir)
    }

    pub fn load() -> Result<Self, StateError> {
        let path = Self::defaut_path()?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self, StateError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path).map_err(|err| StateError::Io {
            path: path.into(),
            source: err,
        })?;

        let state = serde_json::from_str(content.as_str())?;
        Ok(state)
    }

    pub fn save(&self) -> Result<(), StateError> {
        let path = Self::defaut_path()?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), StateError> {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p).map_err(|err| StateError::Io {
                path: path.into(),
                source: err,
            })?;
        }

        let content = serde_json::to_string(self)?;

        fs::write(path, content).map_err(|err| StateError::Io {
            path: path.into(),
            source: err,
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_omits_none_fields() {
        let state = AppState::default();
        let json = serde_json::to_string(&state).unwrap();


        assert_eq!(json, r#"{"myanimelist":{},"anilist":{}}"#);
    }

    #[test]
    fn save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_state_file = temp_dir.path().join("state.json");

        let state = AppState {
            myanimelist: TokenSet {
                access_token: Some("mal_access_token".to_string()),
                refresh_token: Some("mal_refresh_token".to_string()),
            },
            anilist: TokenSet {
                access_token: Some("anilist_access_token".to_string()),
                refresh_token: Some("anilist_refresh_token".to_string()),
            },
        };

        state.save_to(&temp_state_file).unwrap();

        let state = AppState::load_from(&temp_state_file).unwrap();

        assert_eq!(
            state.myanimelist.access_token,
            Some("mal_access_token".to_string())
        );
        assert_eq!(
            state.myanimelist.refresh_token,
            Some("mal_refresh_token".to_string())
        );
        assert_eq!(
            state.anilist.access_token,
            Some("anilist_access_token".to_string())
        );
        assert_eq!(
            state.anilist.refresh_token,
            Some("anilist_refresh_token".to_string())
        );
    }

    #[test]
    fn load_nonexistant_state_returns_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_state_file = temp_dir.path().join("does_not_exist.json");

        let state = AppState::load_from(&temp_state_file).unwrap();

        assert_eq!(state, AppState::default())
    }
}
