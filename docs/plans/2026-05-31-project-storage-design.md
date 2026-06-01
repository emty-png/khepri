# Project Storage System -- Design Spec

## Goal

Add project creation, storage, and persistence to Khepri. The Hub creates projects, the Editor saves/loads scene data, and everything persists between sessions.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Format | RON | Human-readable, git-diffable, Rust-native, used by Bevy |
| Structure | Folder-based with `.khepri/` metadata | Like Godot/Unity. Git-friendly. Extensible for assets. |
| Save behavior | Manual (Ctrl+S) + "save before close?" popup | Like Unity/Godot. User has control. |
| Hub-to-Editor | Same process, Hub closes, Editor opens | Single binary, no IPC. Clean separation. |
| File dialog | `rfd` crate | Native OS dialogs, cross-platform, no extra deps |

## Disk Layout

```
MyGame/                          <- User picks this folder
  .khepri/
    project.ron                  <- Project metadata
    user.ron                     <- User preferences (recent projects)
  scenes/
    main_scene.ron               <- Scene data (objects, transforms)
  assets/                        <- Empty for now (textures, fonts, sounds later)
```

### `project.ron`

```ron
ProjectConfig(
    name: "MyGame",
    version: "0.1.0",
    created_at: "2026-05-31T12:00:00Z",
)
```

### `scenes/main_scene.ron`

```ron
SceneData(
    objects: [
        SceneObjectData(
            id: 1,
            name: "Rectangle 1",
            shape: Rectangle,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 80.0,
            rotation: 0.0,
        ),
    ],
    next_id: 2,
    name_counters: [1, 0, 0],
)
```

### `.khepri/user.ron`

```ron
UserConfig(
    recent_projects: [
        "C:/Users/Empty/Projects/MyGame",
    ],
)
```

## Architecture

### New crate: `crates/khepri-storage/`

Pure Rust, no UI. Handles all file I/O. Depends on khepri-core, serde, ron.

```
crates/khepri-storage/
  Cargo.toml
  src/
    lib.rs              -- pub mod project; pub mod scene_io; pub mod recent;
    project.rs          -- ProjectConfig, create_project(), init_git()
    scene_io.rs         -- SceneData, save_scene(), load_scene()
    recent.rs           -- UserConfig, add_recent_project(), get_recent_projects()
```

### Changes to existing crates

**khepri-core/src/scene.rs:**
- Add `#[derive(Serialize, Deserialize)]` to `ShapeType`, `SceneObject`, `Scene`
- Add `SceneData` struct for serialization (separate from runtime `Scene`)
- Add `to_scene_data()` / `from_scene_data()` conversion methods

**khepri-hub/src/hub.rs:**
- Add project creation popup (triggered by "+" button)
- Add recent projects list on the hub main screen
- Add Hub-to-Editor transition logic

**khepri-hub/src/title_bar.rs:**
- Wire "+" button to open project creation popup

**khepri-app/src/main.rs:**
- Add logic to close Hub and open Editor after project creation/selection

**khepri-editor (future):**
- Load scene from project path on startup
- Save scene on Ctrl+S
- "Save before close?" popup on unsaved changes

## New Dependencies

```toml
# khepri-core
serde = { version = "1", features = ["derive"] }

# khepri-storage
serde = { version = "1", features = ["derive"] }
ron = "0.8"

# khepri-hub
rfd = "0.15"              # Native file dialogs
khepri-storage = { path = "../khepri-storage" }

# khepri-editor (future)
khepri-storage = { path = "../khepri-storage" }
```

## Implementation Tasks

### Task 1: Serialize Scene Data

**Files:**
- Modify: `crates/khepri-core/Cargo.toml` -- add serde dependency
- Modify: `crates/khepri-core/src/scene.rs` -- add Serialize/Deserialize derives

**What:**
- Add `serde::Serialize` and `serde::Deserialize` to `ShapeType`, `SceneObject`, `Scene`
- Add `SceneData` struct for serialization (flat, no selection state)
- Add `to_scene_data()` on `Scene` and `from_scene_data()` on `Scene`
- Add unit tests: round-trip serialize/deserialize

### Task 2: Create khepri-storage Crate

**Files:**
- Create: `crates/khepri-storage/Cargo.toml`
- Create: `crates/khepri-storage/src/lib.rs`
- Create: `crates/khepri-storage/src/project.rs`
- Create: `crates/khepri-storage/src/scene_io.rs`
- Create: `crates/khepri-storage/src/recent.rs`
- Modify: `Cargo.toml` (workspace) -- add khepri-storage

**What:**
- `project.rs`: `ProjectConfig` struct, `create_project()` (creates dir structure, writes project.ron, optionally inits git), `load_project()`
- `scene_io.rs`: `save_scene()` (SceneData -> RON file), `load_scene()` (RON file -> SceneData)
- `recent.rs`: `UserConfig` struct, `add_recent()`, `get_recent()`, stores in `~/.khepri/user.ron`
- Unit tests for all save/load/recent operations (use tempdir)

### Task 3: Hub Project Creation Popup

**Files:**
- Modify: `crates/khepri-hub/src/hub.rs` -- add popup state, UI rendering
- Modify: `crates/khepri-hub/src/title_bar.rs` -- wire "+" button click to popup state

**What:**
- When "+" is clicked, show a popup (like the editor's shape popup but bigger)
- Popup contains: project name input, folder picker (via rfd), "Initialize Git repo" checkbox
- "Create" button: calls khepri-storage to create project, then transitions to editor
- "Cancel" button: closes popup

### Task 4: Hub Recent Projects List

**Files:**
- Modify: `crates/khepri-hub/src/hub.rs` -- replace "No projects yet" with project list

**What:**
- On hub load, read recent projects from `~/.khepri/user.ron`
- Display each as a styled card (name + path)
- Click a project card: opens it in the editor
- If no recent projects: show "No projects yet" (current behavior)

### Task 5: Hub-to-Editor Transition

**Files:**
- Modify: `crates/khepri-app/src/main.rs` -- add editor launch function
- Modify: `crates/khepri-hub/src/hub.rs` -- signal to close hub and open editor

**What:**
- Hub returns a `HubResult` enum: `Quit` (close app) or `OpenProject(PathBuf)` (open editor)
- `main.rs` matches on result: if `OpenProject`, calls `run_editor(path)` which runs the editor with the project loaded
- Editor receives project path and loads scene from disk
- When editor closes, control returns to hub (or app exits, configurable)

### Task 6: Editor Save System

**Files:**
- Modify: `crates/khepri-editor/src/viewport.rs` -- add Ctrl+S detection
- Modify: `crates/khepri-app/src/main.rs` -- wire save/load to editor

**What:**
- Editor receives project path on startup
- Loads scene from `scenes/main_scene.ron` via khepri-storage
- Ctrl+S saves current scene to disk
- Dirty flag tracked on every scene mutation
- "Save before close?" popup on unsaved changes (like Unity/Godot)

## Verification

After all tasks:
1. `cargo build` -- 0 errors, 0 warnings
2. `cargo test` -- all tests pass (including new storage tests)
3. `cargo clippy --workspace -- -D warnings` -- 0 warnings
4. `cargo fmt --all -- --check` -- clean
5. Hub shows recent projects list
6. "+" creates a project with folder dialog
7. Project opens in editor, scene loads
8. Ctrl+S saves scene to disk
9. Closing editor with unsaved changes shows popup
10. Re-opening project loads saved scene
