use std::{
    fs,
    io::{self, ErrorKind, Write},
    path::Path,
};

use color_eyre::eyre::{Result, WrapErr, eyre};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub myanimelist: Auth,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Auth {
    pub client_id: String,
    pub client_secret: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config_dir = dirs::config_dir().ok_or(eyre!("Unable to locate config directory"))?;
        config_dir.push("anisync");
        let config_file_path = config_dir.join("config.toml");

        Self::load_from(&config_dir, &config_file_path)
    }

    pub fn load_from(config_dir: &Path, config_file_path: &Path) -> Result<Self> {
        match fs::read_to_string(config_file_path) {
            Ok(str) => {
                let config: Config = toml::from_str(&str).wrap_err("Failed to parse config.toml")?;
                Ok(config)
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {
                println!("Config does not exist, let's make one!");
                Self::setup_interactive(&config_dir, &config_file_path)
            }
            Err(err) => Err(err).wrap_err(format!(
                "Failed to read config file at {:?}",
                config_file_path
            )),
        }
    }

    pub fn setup_interactive(config_dir: &Path, config_file_path: &Path) -> Result<Self> {
        let mut client_id = String::new();
        print!("Enter Client ID ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut client_id)?;

        let mut client_secret = String::new();
        print!("Enter Client Secret ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut client_secret)?;

        let config = Self {
            myanimelist: Auth {
                client_id: client_id.trim().to_string(),
                client_secret: client_secret.trim().to_string(),
            },
        };

        fs::create_dir_all(config_dir)?;

        let config_text = toml::to_string_pretty(&config)?;

        fs::write(config_file_path, config_text)?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let config = Config {
            myanimelist: Auth {
                client_id: "client_id_123".to_string(),
                client_secret: "client_secret_456".to_string(),
            },
        };
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("client_id_123"));
        assert!(serialized.contains("client_secret_456"));

        let deserialzed: Config = toml::from_str(serialized.as_str()).unwrap();

        assert_eq!(
            deserialzed.myanimelist.client_id,
            "client_id_123".to_string()
        );
        assert_eq!(
            deserialzed.myanimelist.client_secret,
            "client_secret_456".to_string()
        );
    }

    #[test]
    fn test_load_from_existing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_dir = temp_dir.path();
        let config_file_path = config_dir.join("config.toml");

        let toml_str = r#"
            [myanimelist]
            client_id = "mock_client"
            client_secret = "mock_secret"
        "#;

        fs::write(&config_file_path, toml_str).unwrap();

        let config = Config::load_from(config_dir, &config_file_path).unwrap();

        assert_eq!(config.myanimelist.client_id, "mock_client".to_string());
        assert_eq!(config.myanimelist.client_secret, "mock_secret".to_string());
    }
}
