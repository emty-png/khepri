# Plan: 2D Scene Editor with Hierarchy and Properties

## Goal

Add a working 2D scene in the center viewport with:
- Left panel: Hierarchy (object list + "+" button to add objects)
- Center panel: 2D viewport (render objects, click to select, drag to move)
- Right panel: Properties (position, size, rotation for selected object)
- Object types: Rectangle, Circle, Triangle
- Auto-generated names (e.g. "Rectangle 1", "Circle 2")

## Rendering Decision

Use egui's built-in `Shape` API for viewport rendering (rects, circles, triangles with fills, strokes, transforms, hit testing). Vello requires wgpu 29 but eframe 0.34.3 uses wgpu 25 -- incompatible. The scene data model is renderer-agnostic, so swapping to vello later is a rendering-layer change only.

## Architecture

```
khepri-core/src/scene.rs    <- Scene data model (SceneObject, Scene, ShapeType)
khepri-editor/src/
    hierarchy.rs             <- Left panel: object list + add popup
    viewport.rs              <- Center panel: 2D rendering + interaction
    properties.rs            <- Right panel: transform editing
    panels.rs                <- Updated to route panels to new modules
khepri-app/src/main.rs      <- Updated: KhepriApp holds Scene
```

## Data Model (`khepri-core/src/scene.rs`)

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Triangle,
}

#[derive(Clone)]
pub struct SceneObject {
    pub id: u64,
    pub name: String,
    pub shape: ShapeType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32, // degrees
}

pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub selected_id: Option<u64>,
    next_id: u64,
    name_counters: [u32; 3], // one per ShapeType
}
```

Scene methods:
- `new()` -> empty scene
- `add_object(shape: ShapeType)` -> auto-generates name, increments counter, assigns id
- `remove_selected()` -> removes selected object
- `get_selected()` -> Option<&SceneObject>
- `get_selected_mut()` -> Option<&mut SceneObject>
- `select(id: Option<u64>)` -> sets selected_id
- `object_count()` -> usize

Default object properties when added:
- position: (100.0, 100.0)
- size: 80.0 x 80.0
- rotation: 0.0

## Task 1: Scene Data Model + Tests

**Files:**
- Create: `crates/khepri-core/src/scene.rs`

**Steps:**

1. Replace stub `scene.rs` with full data model (ShapeType, SceneObject, Scene)
2. Implement all Scene methods
3. Write unit tests:
   - `test_add_object` -- adds Rectangle, verifies name is "Rectangle 1", id is 1
   - `test_add_multiple_objects` -- adds Rectangle, Circle, Triangle -- names are "Rectangle 1", "Circle 1", "Triangle 1"
   - `test_add_second_of_same_type` -- adds two Rectangles -- names are "Rectangle 1", "Rectangle 2"
   - `test_select_and_deselect` -- select by id, verify selected_id; select None
   - `test_remove_selected` -- add two objects, select one, remove it, verify only the other remains
   - `test_get_selected_mut` -- select an object, modify its position, verify change persists
4. Run: `cargo test -p khepri-core`
5. Expected: all 6 tests pass, 0 warnings

## Task 2: Hierarchy Panel (Left)

**Files:**
- Create: `crates/khepri-editor/src/hierarchy.rs`
- Modify: `crates/khepri-editor/src/lib.rs` (add `pub mod hierarchy;`)

**UI Layout:**
```
+----------------------------------+
| Hierarchy                 [+]   |  <- header with add button
|----------------------------------|
| > Rectangle 1                   |  <- selected (highlighted)
|   Circle 1                      |
|   Triangle 1                    |
|                                  |
+----------------------------------+
```

**Behavior:**
- Header row: "Hierarchy" label on left, "+" button on right
- Clicking "+" opens a popup below it with 3 options: Rectangle, Circle, Triangle
- Selecting an option calls `scene.add_object(shape)`
- Each object row shows its name, is clickable
- Clicking a row selects it (`scene.select(id)`)
- Selected row has a highlight background (use `HOVER_COLOR`)
- Non-selected rows have transparent background

**API:**
```rust
pub fn show_hierarchy(ui: &mut egui::Ui, scene: &mut Scene);
```

**Popup implementation:**
- Use a boolean `show_add_popup` stored in egui memory via `ui.ctx().data_mut(|d| d.get_temp_mut_or(...))` or pass it through the function
- Actually: store `show_add_popup: bool` in the function's egui Id-based memory. Or simpler: use `egui::popup_below_button` or manual `Area` rendering.
- Simplest approach: store the popup state in `egui::Id`-keyed data via `ui.memory(|m| ...)` or use `ui.ctx().input()`.

**Recommended approach:** Use a local `bool` passed by reference, or use egui's `Area` for the popup. Since the popup is per-frame, a simple approach:
```rust
// Use egui's popup_below_widget or manual Area
if ui.button("+").clicked() {
    // Toggle popup
}
// If popup open, show Area with options
```

Best approach: store `show_popup` in egui's `Id`-based data:
```rust
let popup_id = egui::Id::new("add_object_popup");
if add_button.clicked() {
    ui.memory_mut(|m| m.toggle_popup(popup_id));
}
if ui.memory(|m| m.is_popup_open(popup_id)) {
    egui::popup_below_widget(ui, popup_id, &add_button, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
        if ui.button("Rectangle").clicked() {
            scene.add_object(ShapeType::Rectangle);
            ui.memory_mut(|m| m.close_popup());
        }
        // ... Circle, Triangle
    });
}
```

## Task 3: Viewport Panel (Center)

**Files:**
- Create: `crates/khepri-editor/src/viewport.rs`
- Modify: `crates/khepri-editor/src/lib.rs` (add `pub mod viewport;`)

**Rendering:**
- Fill viewport rect with white/light background (use `BG_COLOR` or a lighter variant)
- For each object in scene, compute screen rect from object's (x, y, width, height)
- Object position (x, y) is the center of the shape in "world space"
- For now, world space = screen space (1 pixel = 1 unit). Camera/zoom later.
- Render using egui's `Shape` API:
  - Rectangle: `Shape::rect_filled(rect, corner_radius, color)` + `Shape::rect_stroke(rect, corner_radius, stroke, kind)`
  - Circle: `Shape::circle_filled(center, radius, color)` + `Shape::circle_stroke(center, radius, stroke)`
  - Triangle: `Shape::convex_polygon(points, color, stroke)` -- 3 vertices computed from center + width/height
- Objects are rendered in order (index 0 first = bottom layer)

**Interaction:**
- Use `ui.interact(viewport_rect, id, Sense::click_and_drag())` on the viewport
- On click: find which object was clicked (iterate objects in reverse for top-first hit test)
  - Hit test: check if click point is inside the object's bounding shape
  - For rectangles: simple AABB check
  - For circles: distance from center < radius
  - For triangles: point-in-triangle test
- On drag (after selecting): move the selected object by drag_delta()
- Cursor: `CursorIcon::Grabbing` while dragging, `CursorIcon::Grab` when hovering over an object

**Selection indicator:**
- Draw a dashed or colored border around the selected object (slightly larger than the object)
- Use `Shape::rect_stroke` with a distinct color (e.g., blue #0078D4)

**API:**
```rust
pub fn show_viewport(ui: &mut egui::Ui, scene: &mut Scene);
```

## Task 4: Properties Panel (Right)

**Files:**
- Create: `crates/khepri-editor/src/properties.rs`
- Modify: `crates/khepri-editor/src/lib.rs` (add `pub mod properties;`)

**UI Layout (when object selected):**
```
+----------------------------------+
| Properties                       |
|----------------------------------|
| Type:    Rectangle               |
| Name:    Rectangle 1             |
|                                  |
| Position                         |
|   X: [100.0]  Y: [100.0]        |
|                                  |
| Size                             |
|   W: [80.0]   H: [80.0]         |
|                                  |
| Rotation                         |
|   [0.0] deg                      |
|                                  |
| [Delete Object]                  |
+----------------------------------+
```

**UI Layout (when nothing selected):**
```
+----------------------------------+
| Properties                       |
|----------------------------------|
| No object selected               |
+----------------------------------+
```

**Behavior:**
- Show "Type" and "Name" as read-only labels
- Position X/Y: `ui.add(egui::DragValue::new(&mut obj.x).speed(1.0).prefix("X: "))`
- Size W/H: `ui.add(egui::DragValue::new(&mut obj.width).speed(1.0).prefix("W: "))`
- Rotation: `ui.add(egui::DragValue::new(&mut obj.rotation).speed(1.0).suffix(" deg"))`
- "Delete Object" button: calls `scene.remove_selected()`
- All DragValue inputs clamp to reasonable ranges (position: any, size: 1.0..=5000.0, rotation: -360..=360)

**API:**
```rust
pub fn show_properties(ui: &mut egui::Ui, scene: &mut Scene);
```

## Task 5: Wire Panels Together

**Files:**
- Modify: `crates/khepri-editor/src/panels.rs`
- Modify: `crates/khepri-app/src/main.rs`

**panels.rs changes:**
- `draw_bento` signature changes: add `scene: &mut Scene` parameter
- Left panel section: call `hierarchy::show_hierarchy(ui, scene)` inside a child UI clipped to `left_panel`
- Center panel section: call `viewport::show_viewport(ui, scene)` inside a child UI clipped to `center_panel`
- Right panel section: call `properties::show_properties(ui, scene)` inside a child UI clipped to `right_panel`
- Bottom panel: stays as-is (empty for now)
- Each panel: paint background, then create child UI with `ui.child_ui(panel_rect, Layout::top_down(LEFT), None)`

**main.rs changes:**
- `KhepriApp` gains `scene: Scene` field
- `KhepriApp::new()` creates `Scene::new()`
- `eframe::App::ui()` passes `&mut self.scene` to `draw_bento`

## Task 6: Visual Polish

- Add grid lines in viewport background (subtle, optional -- only if time permits)
- Ensure viewport has a distinct background from panels (slightly lighter or use white)
- Ensure selection highlight is visible against all shape colors
- Add a thin border around each panel for visual separation (use `Shape::rect_stroke`)

## Verification

```bash
cargo build                    # 0 errors, 0 warnings
cargo test -p khepri-core      # all scene tests pass
cargo run -p khepri-app        # window opens:
                                #   - left panel shows "Hierarchy" with "+" button
                                #   - clicking "+" shows popup with Rectangle/Circle/Triangle
                                #   - adding objects shows them in hierarchy list and viewport
                                #   - clicking object in viewport selects it (blue border)
                                #   - dragging object in viewport moves it
                                #   - right panel shows properties of selected object
                                #   - changing values in properties updates viewport in real-time
                                #   - delete button removes object
```
