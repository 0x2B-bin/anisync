use std::{
    fs,
    io::{self, ErrorKind, Write},
    path::{Path, PathBuf},
};

use color_eyre::eyre::{Result, WrapErr, eyre};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {
    pub myanimelist: Auth,
    pub anilist: Auth,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Auth {
    pub client_id: String,
    pub client_secret: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

fn prompt_input(label: &str) -> Result<String> {
    print!("{label}");
    let mut buffer = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

impl Config {
    fn default_paths() -> Result<(PathBuf, PathBuf)> {
        let mut config_dir =
            dirs::config_dir().ok_or(eyre!("Unable to locate config directory"))?;
        config_dir.push("anisync");
        let config_file_path = config_dir.join("config.toml");
        Ok((config_dir, config_file_path))
    }

    pub fn load() -> Result<Self> {
        let (config_dir, config_file_path) = Self::default_paths()?;
        Self::load_from(&config_dir, &config_file_path)
    }

    pub fn load_from(config_dir: &Path, config_file_path: &Path) -> Result<Self> {
        match fs::read_to_string(config_file_path) {
            Ok(str) => {
                let config: Config =
                    toml::from_str(&str).wrap_err("Failed to parse config.toml")?;
                Ok(config)
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                println!("Config does not exist, let's make one!");
                Self::setup_interactive_from(&config_dir, &config_file_path)
            }
            Err(err) => Err(err).wrap_err(format!(
                "Failed to read config file at {:?}",
                config_file_path
            )),
        }
    }

    pub fn setup_interactive_default() -> Result<Self> {
        let (config_dir, config_file_path) = Self::default_paths()?;
        Self::setup_interactive_from(&config_dir, &config_file_path)
    }

    pub fn setup_interactive_from(config_dir: &Path, config_file_path: &Path) -> Result<Self> {
        println!("--- AniSync Setup  ---");

        println!("[MyAnimeList]");
        let mal_client_id = prompt_input("  Client ID: ")?;
        let mal_client_secret = prompt_input("  Client Secret: ")?;

        println!("[AniList]");
        let anilist_client_id = prompt_input("  Client ID: ")?;
        let anilist_client_secret = prompt_input("  Client Secret: ")?;

        let config = Self {
            myanimelist: Auth {
                client_id: mal_client_id,
                client_secret: mal_client_secret,
                access_token: None,
                refresh_token: None,
            },
            anilist: Auth {
                client_id: anilist_client_id,
                client_secret: anilist_client_secret,
                access_token: None,
                refresh_token: None,
            },
        };

        fs::create_dir_all(config_dir)?;

        let config_text = toml::to_string_pretty(&config)?;

        fs::write(config_file_path, config_text)?;

        Ok(config)
    }

    pub fn serialize(&self) -> Result<()> {
        let (config_dir, config_file_path) = Self::default_paths()?;
        self.serialize_to(&config_dir, &config_file_path)?;
        Ok(())
    }

    pub fn serialize_to(&self, config_dir: &Path, config_file_path: &Path) -> Result<()> {
        let serialzed =
            toml::to_string_pretty(self).wrap_err("Failed to serialize config to string")?;
        fs::create_dir_all(config_dir)
            .wrap_err_with(|| format!("Failed to create config directory at {:?}", config_dir))?;
        fs::write(config_file_path, serialzed)
            .wrap_err_with(|| format!("Failed to write config file to {:?}", config_file_path))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_serialize_deserialize() {
        let config = Config {
            myanimelist: Auth {
                client_id: "mal_client_id_123".to_string(),
                client_secret: "mal_client_secret_456".to_string(),
                access_token: None,
                refresh_token: None,
            },
            anilist: Auth {
                client_id: "anilist_client_id_123".to_string(),
                client_secret: "anilist_client_secret_456".to_string(),
                access_token: None,
                refresh_token: None,
            },
        };
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("mal_client_id_123"));
        assert!(serialized.contains("mal_client_secret_456"));
        assert!(serialized.contains("anilist_client_id_123"));
        assert!(serialized.contains("anilist_client_secret_456"));

        let deserialzed: Config = toml::from_str(serialized.as_str()).unwrap();

        assert_eq!(
            deserialzed.myanimelist.client_id,
            "mal_client_id_123".to_string()
        );
        assert_eq!(
            deserialzed.myanimelist.client_secret,
            "mal_client_secret_456".to_string()
        );
        assert_eq!(
            deserialzed.anilist.client_id,
            "anilist_client_id_123".to_string()
        );
        assert_eq!(
            deserialzed.anilist.client_secret,
            "anilist_client_secret_456".to_string()
        );
    }

    #[test]
    fn load_from_existing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path();
        let config_file_path = config_dir.join("config.toml");

        let toml_str = r#"
            [myanimelist]
            client_id = "mock_client"
            client_secret = "mock_secret"

            [anilist]
            client_id = "mock_client"
            client_secret = "mock_secret"
        "#;

        fs::write(&config_file_path, toml_str).unwrap();

        let config = Config::load_from(config_dir, &config_file_path).unwrap();

        assert_eq!(config.myanimelist.client_id, "mock_client".to_string());
        assert_eq!(config.myanimelist.client_secret, "mock_secret".to_string());
        assert_eq!(config.anilist.client_id, "mock_client".to_string());
        assert_eq!(config.anilist.client_secret, "mock_secret".to_string());
    }

    #[test]
    fn serialize_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path();
        let config_file_path = config_dir.join("config.toml");

        let mut config = Config::default();
        config.myanimelist.client_id = "123".to_string();
        config.myanimelist.access_token = Some("mock_access_token".to_string());

        config.serialize_to(config_dir, &config_file_path).unwrap();

        let config_from_file = Config::load_from(config_dir, &config_file_path).unwrap();

        assert_eq!(config_from_file.myanimelist.client_id, "123");
        assert_eq!(
            config_from_file.myanimelist.access_token.as_deref(),
            Some("mock_access_token")
        );
    }
}
