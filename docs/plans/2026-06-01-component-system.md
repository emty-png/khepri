# Plan: Component System & Shape Colliders

**Date:** 2026-06-01  
**Status:** Planned  
**Scope:** Data model + editor UI only. No runtime physics simulation.

---

## Goal

Add a component system to SceneObject and implement shape colliders (Circle, Rectangle, Triangle) as the first components. Each object gets an "Add Components" button in the properties panel. Collider wireframes are rendered in the viewport. The system must be extensible for future components (sprite, rigidbody, audio, etc.) without restructuring.

---

## Current State

- `SceneObject` is a flat struct with 7 fields (id, name, shape, x, y, width, height, rotation).
- No component system exists. `crates/khepri-core/src/ecs.rs` is a 3-line stub.
- Properties panel renders 4 cards (Identity, Position, Size, Rotation) + Delete button.
- Viewport draws objects via `match obj.shape` with 3 arms. No collider visualization.
- Serialization via RON through `SceneData` (Vec<SceneObject> + metadata).

---

## Architecture Decision: Component Storage

**Approach: Enum-based component vec with typed accessor methods.**

```rust
// khepri-core/src/components/mod.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Component {
    Collider(Collider),
    // Future: Sprite(SpriteData), RigidBody(RigidBodyData), AudioSource(...), etc.
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset_x: f32,      // relative to object center
    pub offset_y: f32,
    pub rotation: f32,       // local rotation offset (degrees)
    pub is_trigger: bool,    // future: trigger vs solid
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ColliderShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Triangle { width: f32, height: f32 },
}
```

**Why enum vec, not direct fields:**
- Adding a new component type = add one enum variant + one data struct. No SceneObject field changes.
- `has_component::<T>()`, `get_component::<T>()`, `get_component_mut::<T>()` accessor methods give typed ergonomics.
- Serde-friendly (RON round-trips naturally).
- The vec is small (most objects will have 0-3 components). Linear scan is fine.
- Future migration to full ECS (Bevy, or custom) maps naturally: each variant becomes a component type.

**Why not `HashMap<TypeId, Box<dyn Any>>`:**
- Loses serde support. TypeId isn't serializable.
- Requires downcasting everywhere. No compile-time type safety.
- Debug/Clone are painful.

---

## Collider Design

### Shapes

| Shape | Fields | Auto-default from object |
|---|---|---|
| Circle | `radius: f32` | `obj.width / 2.0` |
| Rectangle | `width: f32, height: f32` | `obj.width, obj.height` |
| Triangle | `width: f32, height: f32` | `obj.width, obj.height` |

All shapes share offset (x, y) and rotation via the parent `Collider` struct.

### Collider defaults when added via "Add Components"
- `offset_x = 0.0, offset_y = 0.0` (centered on object)
- `rotation = 0.0` (aligned with object)
- `is_trigger = false`
- Shape dimensions auto-derived from the object's current width/height

### Viewport rendering
- Collider wireframe drawn as a **dashed stroke** in a distinct color (e.g. `#FF6B6B` red-orange)
- Rendered **after** the object fill/stroke, **before** the selection highlight
- Shape match: Circle -> dashed circle stroke, Rectangle -> dashed rotated rect, Triangle -> dashed rotated triangle
- Offset applied: wireframe center at `(obj.x + offset_x, obj.y + offset_y)`
- Rotation: `obj.rotation + collider.rotation` for the wireframe

---

## File Changes

### khepri-core (data model)

**New file: `crates/khepri-core/src/components/mod.rs`**
- `Component` enum (starts with `Collider` variant)
- `Collider` struct (shape, offset_x, offset_y, rotation, is_trigger)
- `ColliderShape` enum (Circle, Rectangle, Triangle)
- `impl Collider` with `new_for_object(shape_type, width, height) -> Collider` factory
- `impl ColliderShape` with `name() -> &'static str` and `default_from_object(shape_type, w, h)`

**Modify: `crates/khepri-core/src/scene.rs`**
- Add `pub components: Vec<Component>` to `SceneObject`
- Default to empty vec `vec![]` in `add_object()`
- Add typed accessor methods on `SceneObject`:
  - `has_collider() -> bool`
  - `collider() -> Option<&Collider>`
  - `collider_mut() -> Option<&mut Collider>`
  - `add_component(component: Component)`
  - `remove_component<T>(predicate)` — remove by variant match

**Modify: `crates/khepri-core/src/lib.rs`**
- Add `pub mod components;`

### khepri-storage (serialization)

**No changes needed.** `SceneData` wraps `Vec<SceneObject>`. Since `SceneObject` derives `Serialize, Deserialize` and `Component`/`Collider`/`ColliderShape` also derive them, RON serialization picks up the new field automatically. Existing scene files without `components` will deserialize with `vec![]` default (RON handles missing vec fields as empty if we use `#[serde(default)]`).

**One addition:** Add `#[serde(default)]` to the `components` field on `SceneObject` so old scene files (without the field) load cleanly.

### khepri-editor (UI + viewport)

**New file: `crates/khepri-editor/src/components_ui.rs`**
- `show_components(ui, scene)` — renders the "Components" section in the properties panel
- "Add Components" button at the bottom of the properties panel (after existing cards, before Delete)
- Clicking "Add Components" opens a dropdown/popup with available component types:
  - "Collider" (only shown if object doesn't already have one)
- When a component type is selected, it's added with defaults auto-derived from the object
- For each attached component, render an editable card:
  - **Collider card:** shape type dropdown (Circle/Rectangle/Triangle), dimension fields (DragValue), offset_x/offset_y (DragValue), rotation offset (DragValue), is_trigger (checkbox)
  - Each card has a remove (X) button

**Modify: `crates/khepri-editor/src/properties.rs`**
- Import and call `components_ui::show_components(ui, scene)` after the Rotation card, before the Delete button
- The "Add Components" popup follows the same visual pattern as the hierarchy "+" popup (manual state + egui::Area)

**Modify: `crates/khepri-editor/src/viewport.rs`**
- After drawing the object fill/stroke, check if the object has a collider
- If yes, draw the collider wireframe:
  - Set stroke to dashed 1.5px `#FF6B6B` (red-orange)
  - Match on `ColliderShape`:
    - Circle: `Shape::circle_stroke` at `(obj.x + offset_x, obj.y + offset_y)` with `radius`
    - Rectangle: `Shape::convex_polygon` from 4 corners, rotated by `obj.rotation + collider.rotation`
    - Triangle: `Shape::convex_polygon` from 3 vertices, rotated by `obj.rotation + collider.rotation`
  - Transform offsets through the camera world-to-screen projection

**Modify: `crates/khepri-editor/src/lib.rs`**
- Add `pub mod components_ui;`

---

## Implementation Order

1. **Component data model** — `components/mod.rs` with `Component`, `Collider`, `ColliderShape` enums/structs. Serde derives, accessor methods, factory method. Unit tests.
2. **SceneObject integration** — Add `components` field with `#[serde(default)]`. Add accessor methods. Update `Scene::add_object()` defaults.
3. **Serialization verification** — Build, save a scene with a collider, reload it. Confirm old scenes without colliders still load.
4. **Collider card in properties** — `components_ui.rs` with the "Add Components" button and collider card UI. Wire into `properties.rs`.
5. **Viewport wireframe** — Draw collider outlines in `viewport.rs` with dashed stroke and offset/rotation.
6. **Polish** — Hover states, spacing, edge cases (remove component, change shape type, verify dirty flag picks up component changes).

---

## "Add Components" UX Flow

```
┌─────────────────────────┐
│ [Identity card]         │
│ [Position card]         │
│ [Size card]             │
│ [Rotation card]         │
│                         │
│ ┌─ Components ────────┐ │
│ │ [Collider card]     │ │  ← if collider exists
│ │   Shape: [Circle ▾] │ │
│ │   Radius: [40.0]    │ │
│ │   Offset: [0, 0]    │ │
│ │   Rotation: [0°]    │ │
│ │   Trigger: [ ]      │ │
│ │              [×]    │ │  ← remove button
│ └─────────────────────┘ │
│                         │
│ [+ Add Component]       │  ← popup: [Collider]
│                         │
│ [Delete Object]         │
└─────────────────────────┘
```

---

## Edge Cases & Validation

- **No component, no UI:** If object has no components, the "Components" section shows only the "+ Add Component" button. No empty card.
- **One collider per object:** The "Collider" option is hidden from the dropdown if the object already has one. (Can be relaxed later for multi-collider support.)
- **Negative dimensions:** Clamp radius, width, height to >= 1.0 in DragValue range.
- **Old scene files:** `#[serde(default)]` on `components` field ensures vec is empty when loading scenes written before this feature.
- **Dirty flag:** Snapshot comparison (`to_scene_data()`) includes `components` because it serializes the full `Vec<SceneObject>`. No changes needed to dirty detection.
- **Collider rotation wrapping:** Clamp or allow free rotation (-360..=360 range on the DragValue, same as object rotation).

---

## What This Does NOT Include (Future Work)

- Runtime physics simulation (game view phase)
- Collision detection / resolution
- Rigid body dynamics (velocity, forces, mass)
- Joint/constraint system
- Physics material (friction, restitution)
- Multi-collider per object
- Convex polygon / capsule / polyline colliders
- Collider gizmo manipulation in viewport (drag to resize)

---

## Testing Plan

- Unit tests for `Collider::new_for_object()` factory — verify defaults match object dimensions for each shape type.
- Unit tests for `ColliderShape::name()` and accessor methods.
- Serialization round-trip: create SceneObject with collider, serialize to RON, deserialize, verify collider data matches.
- Backward compat: deserialize an old scene file (without `components` field), verify it loads with empty vec.
- Build: 0 errors, 0 warnings, all existing tests pass.
