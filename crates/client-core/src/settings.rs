//! Local, owner-only API configuration. Never include parsed values in errors.

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub api_id: i32,
    pub api_hash: String,
    #[serde(default)]
    pub use_test_dc: bool,
    #[serde(default)]
    pub tdlib_library: Option<PathBuf>,
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

    /// Write only application credentials and settings, never session data.
    pub fn save(&self) -> Result<(), SettingsError> {
        if !self.is_valid() {
            return Err(SettingsError::Invalid);
        }
        let directory = config_path();
        fs::create_dir_all(&directory).map_err(|_| SettingsError::Invalid)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
                .map_err(|_| SettingsError::Invalid)?;
        }
        let path = directory.join("config.toml");
        let contents = toml::to_string(self).map_err(|_| SettingsError::Invalid)?;
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|_| SettingsError::Invalid)?;
            file.write_all(contents.as_bytes())
                .map_err(|_| SettingsError::Invalid)?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .map_err(|_| SettingsError::Invalid)?;
        }
        #[cfg(not(unix))]
        fs::write(path, contents).map_err(|_| SettingsError::Invalid)?;
        Ok(())
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
