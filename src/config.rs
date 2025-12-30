//! Configuration management for the `cave` CLI.
//!
//! This module handles reading, writing, and updating the global
//! configuration file located at `~/.caveconfig.json`.
//!
//! # Adding a new configuration option
//! 1. **Add a field** to the [`Config`] struct (and update [`Default::default`])
//! 2. **Add a public setter function** following the pattern of [`set_auto_update`],
//! 3. **Add the option to the cli** (in ConfigAction in `cli.rs`)
//! 4. **Update the CLI command handler** in `main.rs`

use crate::manage::CaveError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Stores Docker registry credentials and repository information.
#[derive(Debug, Deserialize, Serialize)]
pub struct Registry {
    /// Name of the Docker repository.
    pub repo: String,
    /// Username for authentication.
    pub user: String,
    /// Access token or password.
    pub token: String,
}

/// Global configuration for the `cave` CLI.
///
/// The configuration is stored in `~/.caveconfig.json`
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Whether automatic update checks are enabled.
    pub auto_update: bool,
    /// Whether automatic new cave release checks are enabled.
    #[serde(default = "default_enable_auto_update")]
    pub auto_release_check: bool,
    /// Whether version tracking is enabled.
    pub version_tracking: bool,
    /// Optional registry configuration for private Docker images.
    pub registry: Option<Registry>,
    ///User_id used for telemetry, generated randomly
    pub user_id: String,
}

fn default_enable_auto_update() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_update: false,
            auto_release_check: true,
            version_tracking: true,
            registry: None,
            user_id: Uuid::new_v4().to_string(),
        }
    }
}

fn config_path() -> Result<PathBuf, CaveError> {
    let home = dirs::home_dir().ok_or(CaveError::HomeNotFound)?;
    Ok(home.join(".caveconfig.json"))
}

/// Reads the user configuration from `~/.caveconfig.json`.
///
/// If the file does not exist, a default configuration is returned.
///
/// # Example
/// ```
/// use cave::config::read_config;
///
/// let cfg = read_config().expect("Failed to read config");
/// println!("Auto update: {}", cfg.auto_update);
/// ```
pub fn read_config() -> Result<Config, CaveError> {
    let path = config_path()?;
    if !path.exists() {
        let config = Config::default();
        write_config(&config)?;
        return Ok(config);
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content).map_err(CaveError::SerdeError)?)
}

/// Writes the given configuration to `~/.caveconfig.json`.
///
/// # Example
/// ```
/// use cave::config::{write_config, Config};
///
/// let cfg = Config { auto_update: true, version_tracking: false, registry: None };
/// write_config(&cfg).expect("Failed to write config");
/// ```
pub fn write_config(config: &Config) -> Result<(), CaveError> {
    let path = config_path()?;
    let content = serde_json::to_string_pretty(config).map_err(CaveError::SerdeError)?;
    fs::write(path, content)?;
    Ok(())
}

/// Enables or disables automatic update checks globally.
///
/// # Example
/// ```
/// use cave::config::set_auto_update;
///
/// set_auto_update(true).expect("Failed to update setting");
/// ```
pub fn set_auto_update(value: bool) -> Result<(), CaveError> {
    let mut cfg = read_config()?;
    cfg.auto_update = value;
    write_config(&cfg)
}

/// Enables or disables automatic new cave release checks globally.
///
/// # Example
/// ```
/// use cave::config::set_auto_release_check;
///
/// set_auto_release_check(false).expect("Failed to update setting");
/// ```
pub fn set_auto_release_check(value: bool) -> Result<(), CaveError> {
    let mut cfg = read_config()?;
    cfg.auto_release_check = value;
    write_config(&cfg)
}

/// Enables or disables version tracking globally.
///
/// # Example
/// ```
/// use cave::config::set_version_tracking;
///
/// set_version_tracking(false).expect("Failed to update setting");
/// ```
pub fn set_version_tracking(value: bool) -> Result<(), CaveError> {
    let mut cfg = read_config()?;
    cfg.version_tracking = value;
    write_config(&cfg)
}

// TODO : uncomment to have registry option
//
// /// Sets the Docker registry configuration.
// ///
// /// Pass `None` to remove any existing registry settings.
// ///
// /// # Example
// /// ```
// /// use cave::config::{set_registry, Registry};
// ///
// /// let registry = Registry {
// ///     repo: "docker.io/myrepo".to_string(),
// ///     user: "username".to_string(),
// ///     token: "mytoken".to_string(),
// /// };
// /// set_registry(Some(registry)).expect("Failed to set registry");
// /// ```
// pub fn set_registry(registry: Option<Registry>) -> Result<(), CaveError> {
//     let mut cfg = read_config()?;
//     cfg.registry = registry;
//     write_config(&cfg)
// }

pub fn read_user_id() -> Result<String, CaveError> {
    let mut config = read_config()?;
    let user_id = config.user_id;
    if user_id.is_empty() {
        config.user_id = Uuid::new_v4().to_string();
        write_config(&config)?;
        return Ok(config.user_id);
    }
    Ok(user_id)
}
