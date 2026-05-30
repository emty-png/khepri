/// Shape types available in the scene editor.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Triangle,
}

impl ShapeType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Rectangle => "Rectangle",
            Self::Circle => "Circle",
            Self::Triangle => "Triangle",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Rectangle => 0,
            Self::Circle => 1,
            Self::Triangle => 2,
        }
    }
}

/// A single object in the 2D scene.
#[derive(Clone)]
pub struct SceneObject {
    pub id: u64,
    pub name: String,
    pub shape: ShapeType,
    /// X position (center of the shape in world space).
    pub x: f32,
    /// Y position (center of the shape in world space).
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Rotation in degrees.
    pub rotation: f32,
}

/// The complete 2D scene containing all objects and selection state.
pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub selected_id: Option<u64>,
    next_id: u64,
    name_counters: [u32; 3],
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            selected_id: None,
            next_id: 1,
            name_counters: [0; 3],
        }
    }

    /// Add a new object with auto-generated name and default transform.
    pub fn add_object(&mut self, shape: ShapeType) -> u64 {
        let idx = shape.index();
        self.name_counters[idx] += 1;
        let id = self.next_id;
        self.next_id += 1;

        let name = format!("{} {}", shape.name(), self.name_counters[idx]);

        self.objects.push(SceneObject {
            id,
            name,
            shape,
            x: 100.0,
            y: 100.0,
            width: 80.0,
            height: 80.0,
            rotation: 0.0,
        });

        id
    }

    /// Remove the currently selected object, if any.
    pub fn remove_selected(&mut self) {
        if let Some(id) = self.selected_id {
            self.objects.retain(|o| o.id != id);
            self.selected_id = None;
        }
    }

    /// Get a reference to the selected object.
    pub fn get_selected(&self) -> Option<&SceneObject> {
        let id = self.selected_id?;
        self.objects.iter().find(|o| o.id == id)
    }

    /// Get a mutable reference to the selected object.
    pub fn get_selected_mut(&mut self) -> Option<&mut SceneObject> {
        let id = self.selected_id?;
        self.objects.iter_mut().find(|o| o.id == id)
    }

    /// Set or clear the selection.
    pub fn select(&mut self, id: Option<u64>) {
        self.selected_id = id;
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_object() {
        let mut scene = Scene::new();
        let id = scene.add_object(ShapeType::Rectangle);
        assert_eq!(id, 1);
        assert_eq!(scene.object_count(), 1);
        assert_eq!(scene.objects[0].name, "Rectangle 1");
        assert_eq!(scene.objects[0].shape, ShapeType::Rectangle);
    }

    #[test]
    fn test_add_multiple_objects() {
        let mut scene = Scene::new();
        let r_id = scene.add_object(ShapeType::Rectangle);
        let c_id = scene.add_object(ShapeType::Circle);
        let t_id = scene.add_object(ShapeType::Triangle);

        assert_eq!(scene.object_count(), 3);
        assert_eq!(scene.objects[0].name, "Rectangle 1");
        assert_eq!(scene.objects[0].id, r_id);
        assert_eq!(scene.objects[1].name, "Circle 1");
        assert_eq!(scene.objects[1].id, c_id);
        assert_eq!(scene.objects[2].name, "Triangle 1");
        assert_eq!(scene.objects[2].id, t_id);
    }

    #[test]
    fn test_add_second_of_same_type() {
        let mut scene = Scene::new();
        scene.add_object(ShapeType::Rectangle);
        scene.add_object(ShapeType::Rectangle);
        assert_eq!(scene.objects[0].name, "Rectangle 1");
        assert_eq!(scene.objects[1].name, "Rectangle 2");
    }

    #[test]
    fn test_select_and_deselect() {
        let mut scene = Scene::new();
        let id = scene.add_object(ShapeType::Circle);

        scene.select(Some(id));
        assert_eq!(scene.selected_id, Some(id));

        scene.select(None);
        assert_eq!(scene.selected_id, None);
    }

    #[test]
    fn test_remove_selected() {
        let mut scene = Scene::new();
        let id1 = scene.add_object(ShapeType::Rectangle);
        let _id2 = scene.add_object(ShapeType::Circle);

        scene.select(Some(id1));
        scene.remove_selected();

        assert_eq!(scene.object_count(), 1);
        assert_eq!(scene.objects[0].name, "Circle 1");
        assert_eq!(scene.selected_id, None);
    }

    #[test]
    fn test_get_selected_mut() {
        let mut scene = Scene::new();
        let id = scene.add_object(ShapeType::Triangle);
        scene.select(Some(id));

        scene.get_selected_mut().unwrap().x = 250.0;
        scene.get_selected_mut().unwrap().y = 300.0;

        let obj = scene.get_selected().unwrap();
        assert_eq!(obj.x, 250.0);
        assert_eq!(obj.y, 300.0);
    }
}
