//! Local, owner-only API configuration. Never include parsed values in errors.

use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub api_id: i32,
    pub api_hash: String,
    #[serde(default = "default_test_dc")]
    pub use_test_dc: bool,
    #[serde(default)]
    pub tdlib_library: Option<PathBuf>,
}

fn default_test_dc() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    Missing,
    InsecurePermissions,
    Invalid,
}

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        let path = config_path().join("config.toml");
        let metadata = fs::metadata(&path).map_err(|_| SettingsError::Missing)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(SettingsError::InsecurePermissions);
            }
        }
        let contents = fs::read_to_string(path).map_err(|_| SettingsError::Invalid)?;
        let settings: Self = toml::from_str(&contents).map_err(|_| SettingsError::Invalid)?;
        if !settings.is_valid() {
            return Err(SettingsError::Invalid);
        }
        Ok(settings)
    }

    pub fn is_valid(&self) -> bool {
        self.api_id > 0
            && self.api_hash.len() == 32
            && self.api_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
    }
}

pub fn config_path() -> PathBuf {
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(path).join("telerust");
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/telerust")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_ends_in_app_directory() {
        assert!(config_path().ends_with("telerust"));
    }

    #[test]
    fn malformed_configuration_fails_without_echoing_values() {
        let result = Settings {
            api_id: 0,
            api_hash: "0123456789abcdef0123456789abcdef".into(),
            use_test_dc: true,
            tdlib_library: None,
        };
        assert!(!result.is_valid());
    }
}
