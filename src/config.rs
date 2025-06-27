use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "ticktick")]
    pub ticktick: TickTickConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickTickConfig {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default = "default_redirect_uri")]
    pub redirect_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

pub fn default_redirect_uri() -> String {
    "http://localhost:8080/callback".to_string()
}

impl Config {
    fn config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".ticktick.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Err(anyhow!(
                "Configuration file not found at: {}\n\nPlease create this file with your TickTick API credentials:\n\n[ticktick]\nclient_id = \"your_client_id_here\"\nclient_secret = \"your_client_secret_here\"\n# redirect_uri = \"http://localhost:8080/callback\"  # Optional, defaults to this value", 
                config_path.display()
            ));
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("Failed to read config file {}: {}", config_path.display(), e))?;
        
        let config: Config = toml::from_str(&config_content)
            .map_err(|e| anyhow!("Failed to parse config file {}: {}", config_path.display(), e))?;
        
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let config_content = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
        
        fs::write(&config_path, config_content)
            .map_err(|e| anyhow!("Failed to write config file {}: {}", config_path.display(), e))?;
        
        Ok(())
    }

    pub fn create_example() -> Result<()> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            return Err(anyhow!("Configuration file already exists at: {}", config_path.display()));
        }

        let example_config = r#"# TickTick API Configuration
# Get your client_id and client_secret from the TickTick Developer Center
# https://developer.ticktick.com/

[ticktick]
client_id = "your_client_id_here"
client_secret = "your_client_secret_here"

# Optional: Custom redirect URI (defaults to http://localhost:8080/callback)
# Make sure this matches what you configured in the TickTick Developer Center
# redirect_uri = "http://localhost:8080/callback"

# Note: access_token will be automatically added and managed by the application
# after the first successful OAuth authentication
"#;

        fs::write(&config_path, example_config)
            .map_err(|e| anyhow!("Failed to create config file {}: {}", config_path.display(), e))?;
        
        println!("✅ Created example configuration file at: {}", config_path.display());
        println!("📝 Please edit this file with your actual TickTick API credentials.");
        
        Ok(())
    }
}
