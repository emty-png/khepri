use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// User-level preferences, stored in `~/.khepri/user.ron`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub recent_projects: Vec<PathBuf>,
}

/// Get the path to the global user config file.
fn user_config_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".khepri").join("user.ron")
}

/// Load the user config from disk. Returns default if file doesn't exist.
pub fn load_user_config() -> UserConfig {
    let path = user_config_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return UserConfig::default(),
    };
    ron::from_str(&content).unwrap_or_default()
}

/// Save the user config to disk.
pub fn save_user_config(config: &UserConfig) -> Result<(), std::io::Error> {
    let path = user_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pretty = ron::ser::to_string_pretty(config, ron::ser::PrettyConfig::default())
        .map_err(std::io::Error::other)?;
    std::fs::write(path, pretty)
}

/// Add a project path to the recent projects list.
/// Moves it to the front if already present. Keeps max 20 entries.
pub fn add_recent_project(project_root: &Path) -> Result<(), std::io::Error> {
    let mut config = load_user_config();
    let canonical =
        std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());

    // Remove if already present
    config.recent_projects.retain(|p| p != &canonical);
    // Insert at front
    config.recent_projects.insert(0, canonical);
    // Cap at 20
    config.recent_projects.truncate(20);

    save_user_config(&config)
}

/// Get the list of recent project paths.
pub fn get_recent_projects() -> Vec<PathBuf> {
    load_user_config().recent_projects
}

/// Remove a project from the recent projects list.
pub fn remove_recent_project(project_root: &Path) -> Result<(), std::io::Error> {
    let mut config = load_user_config();
    config.recent_projects.retain(|p| p != project_root);
    save_user_config(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests modify the real ~/.khepri/user.ron file.
    // We test only the serialization round-trip here.

    #[test]
    fn test_user_config_roundtrip() {
        let config = UserConfig {
            recent_projects: vec![PathBuf::from("/tmp/MyGame"), PathBuf::from("/tmp/Other")],
        };

        let ron_str = ron::to_string(&config).unwrap();
        let loaded: UserConfig = ron::from_str(&ron_str).unwrap();

        assert_eq!(loaded.recent_projects.len(), 2);
        assert_eq!(loaded.recent_projects[0], PathBuf::from("/tmp/MyGame"));
    }

    #[test]
    fn test_user_config_default() {
        let config = UserConfig::default();
        assert!(config.recent_projects.is_empty());
    }
}
