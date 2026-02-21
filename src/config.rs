use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub display: DisplayConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub endpoint: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.collected.cards/graphql".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub image_mode: String,
    pub color: bool,
    pub language: String,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            image_mode: "auto".to_string(),
            color: true,
            language: "de".to_string(),
        }
    }
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("collected");
        Ok(dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn get_token(&self) -> Option<&str> {
        self.auth.token.as_deref()
    }

    pub fn set_token(&mut self, token: String) -> Result<()> {
        self.auth.token = Some(token);
        self.save()
    }

    pub fn clear_token(&mut self) -> Result<()> {
        self.auth.token = None;
        self.save()
    }
}
