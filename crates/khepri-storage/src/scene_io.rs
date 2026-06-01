use khepri_core::scene::{Scene, SceneData};
use std::path::Path;

/// Save a scene to a RON file.
pub fn save_scene(path: &Path, scene: &Scene) -> Result<(), std::io::Error> {
    let data = scene.to_scene_data();
    let pretty = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default())
        .map_err(std::io::Error::other)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, pretty)
}

/// Load a scene from a RON file.
pub fn load_scene(path: &Path) -> Result<Scene, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let data: SceneData = ron::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Scene::from_scene_data(data))
}

/// Get the default scene path for a project.
pub fn default_scene_path(project_root: &Path) -> std::path::PathBuf {
    project_root.join("scenes").join("main_scene.ron")
}

#[cfg(test)]
mod tests {
    use super::*;
    use khepri_core::scene::ShapeType;

    #[test]
    fn test_save_and_load_scene() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_scene.ron");

        let mut scene = Scene::new();
        scene.add_object(ShapeType::Rectangle);
        scene.add_object(ShapeType::Circle);
        scene.select(Some(1));

        save_scene(&path, &scene).unwrap();
        assert!(path.exists());

        let loaded = load_scene(&path).unwrap();
        assert_eq!(loaded.object_count(), 2);
        assert_eq!(loaded.objects[0].name, "Rectangle 1");
        assert_eq!(loaded.objects[1].name, "Circle 1");
        assert_eq!(loaded.selected_id, None);
    }

    #[test]
    fn test_save_empty_scene() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.ron");

        let scene = Scene::new();
        save_scene(&path, &scene).unwrap();

        let loaded = load_scene(&path).unwrap();
        assert_eq!(loaded.object_count(), 0);
    }

    #[test]
    fn test_default_scene_path() {
        let root = Path::new("/tmp/MyGame");
        let p = default_scene_path(root);
        assert_eq!(p, PathBuf::from("/tmp/MyGame/scenes/main_scene.ron"));
    }

    use std::path::PathBuf;
}
