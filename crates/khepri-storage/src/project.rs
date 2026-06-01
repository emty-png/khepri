use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::scene_io;

/// Project metadata stored in `.khepri/project.ron`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub created_at: String,
}

/// Create a new project on disk with the standard directory layout.
///
/// ```text
/// <project_root>/
///   .khepri/
///     project.ron
///   scenes/
///     main_scene.ron
///   assets/
/// ```
pub fn create_project(
    root: &Path,
    name: &str,
    git_init: bool,
) -> Result<ProjectConfig, std::io::Error> {
    let khepri_dir = root.join(".khepri");
    let scenes_dir = root.join("scenes");
    let assets_dir = root.join("assets");

    std::fs::create_dir_all(&khepri_dir)?;
    std::fs::create_dir_all(&scenes_dir)?;
    std::fs::create_dir_all(&assets_dir)?;

    let config = ProjectConfig {
        name: name.to_string(),
        version: "0.1.0".to_string(),
        created_at: chrono_stamp(),
    };

    let config_str = ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default())
        .map_err(std::io::Error::other)?;
    std::fs::write(khepri_dir.join("project.ron"), config_str)?;

    // Write an empty default scene
    scene_io::save_scene(
        &scenes_dir.join("main_scene.ron"),
        &khepri_core::scene::Scene::new(),
    )?;

    if git_init {
        init_git(root)?;
    }

    Ok(config)
}

/// Load an existing project config from disk.
pub fn load_project(root: &Path) -> Result<ProjectConfig, std::io::Error> {
    let path = root.join(".khepri").join("project.ron");
    let content = std::fs::read_to_string(&path)?;
    ron::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Run `git init` in the project root.
pub fn init_git(root: &Path) -> Result<(), std::io::Error> {
    std::process::Command::new("git")
        .arg("init")
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    Ok(())
}

/// Check if a path is a valid Khepri project (has .khepri/project.ron).
pub fn is_project(root: &Path) -> bool {
    root.join(".khepri").join("project.ron").exists()
}

fn chrono_stamp() -> String {
    // Simple timestamp without pulling in chrono crate
    // Uses system time as ISO-like string
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_load_project() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("MyGame");

        let config = create_project(&root, "MyGame", false).unwrap();
        assert_eq!(config.name, "MyGame");
        assert_eq!(config.version, "0.1.0");

        // Verify directory structure
        assert!(root.join(".khepri").join("project.ron").exists());
        assert!(root.join("scenes").join("main_scene.ron").exists());
        assert!(root.join("assets").is_dir());

        // Load it back
        let loaded = load_project(&root).unwrap();
        assert_eq!(loaded.name, "MyGame");
    }

    #[test]
    fn test_is_project() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!is_project(dir.path()));

        create_project(dir.path(), "Test", false).unwrap();
        assert!(is_project(dir.path()));
    }

    #[test]
    fn test_git_init() {
        let dir = tempfile::tempdir().unwrap();
        create_project(dir.path(), "GitTest", true).unwrap();
        assert!(dir.path().join(".git").exists());
    }
}
