use tracing::{debug, warn};
use super::{user_config_path, UserConfig, UserConfigError};
/// Load the user configuration from `~/.centy/config.toml`.
///
/// Returns `Ok(UserConfig::default())` if the file does not exist.
///
/// # Errors
///
/// Returns [`UserConfigError`] if the file exists but cannot be read or parsed.
pub fn load_user_config() -> Result<UserConfig, UserConfigError> {
    let path = match user_config_path() {
        Some(p) => p,
        None => {
            warn!("Could not determine user config directory; using defaults");
            return Ok(UserConfig::default());
        }
    };
    if !path.exists() {
        debug!("User config not found at {}; using defaults", path.display());
        return Ok(UserConfig::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let config: UserConfig = toml::from_str(&content)?;
    debug!("Loaded user config from {}", path.display());
    Ok(config)
}
