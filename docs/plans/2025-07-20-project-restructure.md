# Plan: Production-Ready Project Restructure

## Problem

Everything is in `main.rs` (226 lines): colors, layout state, bento logic, window resize, app entry. `title_bar.rs` has its own color constant. No module boundaries. `src/ui/` exists but is empty. No separation between editor UI, engine core, and app shell.

When you add a 2D renderer, ECS, scene graph, or asset system, you'll be threading new code through a flat module structure with no ownership boundaries. This becomes a headache fast.

## Decision

**Cargo workspace with 3 crates:**

```
khepri/                          <- workspace root
├── Cargo.toml                   <- workspace manifest (no [package], only [workspace])
├── crates/
│   ├── khepri-core/             <- pure Rust library: engine core (no UI deps)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs        <- colors, constants, engine settings
│   │       ├── renderer.rs      <- stub: future 2D rendering (wgpu)
│   │       ├── scene.rs         <- stub: future scene graph
│   │       ├── ecs.rs           <- stub: future entity-component-system
│   │       └── assets.rs        <- stub: future asset loading
│   ├── khepri-editor/           <- library: editor UI (depends on khepri-core + egui)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── title_bar.rs     <- moved from src/title_bar.rs
│   │       ├── panels.rs        <- bento layout + splitters (extracted from main.rs)
│   │       └── window_resize.rs <- frameless window resize (extracted from main.rs)
│   └── khepri-app/              <- binary: entry point (depends on khepri-core + khepri-editor)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs          <- just entry point + eframe setup
├── docs/
│   └── plans/
└── .gitignore
```

## Why Workspace, Not Single Crate

1. **Compilation boundaries**: Changing editor UI doesn't recompile engine core. Engine compiles once, editor compiles once, app links them.
2. **Dependency isolation**: `khepri-core` has zero UI dependencies. It doesn't pull in egui/eframe. When you ship a standalone game (no editor), you depend only on `khepri-core`.
3. **Testing in isolation**: Engine core can be tested with `cargo test -p khepri-core` — no window, no GPU, no UI.
4. **Natural growth path**: Adding a scripting runtime? New crate. Adding a standalone player? New binary crate that depends on `khepri-core` only. No refactoring needed.

## Why Scaffold Empty Modules

`renderer.rs`, `scene.rs`, `ecs.rs`, `assets.rs` will be empty stubs with a single `pub struct` and a doc comment. This costs nothing but gives you:
- A clear place to put code when you start building those systems
- `cargo doc` shows the full architecture from day one
- Prevents "where does this go?" decisions later

## What Moves Where

| Current Location | Destination | Notes |
|---|---|---|
| `BG_COLOR`, `FG_COLOR` constants | `khepri-core/src/config.rs` | Shared by all crates |
| `BentoLayout` + `draw_bento()` | `khepri-editor/src/panels.rs` | Extracted from main.rs |
| `PANEL_RADIUS`, `PANEL_GAP`, `PANEL_PADDING` | `khepri-editor/src/panels.rs` | Moved with the layout code |
| `custom_window_resize()` | `khepri-editor/src/window_resize.rs` | Extracted from main.rs |
| `title_bar.rs` (full file) | `khepri-editor/src/title_bar.rs` | `HOVER_COLOR` moves to config.rs |
| `KhepriApp` + `main()` | `khepri-app/src/main.rs` | Just entry point |
| `src/ui/` (empty dir) | deleted | Unused |

## Dependencies

```toml
# khepri-core/Cargo.toml
[package]
name = "khepri-core"
version = "0.1.0"
edition = "2024"

[dependencies]
# None yet. Future: wgpu, bytemuck, glam, etc.
```

```toml
# khepri-editor/Cargo.toml
[package]
name = "khepri-editor"
version = "0.1.0"
edition = "2024"

[dependencies]
khepri-core = { path = "../khepri-core" }
eframe = { version = "0.34", default-features = false, features = ["glow", "accesskit", "default_fonts", "wayland", "x11", "persistence"] }
```

```toml
# khepri-app/Cargo.toml
[package]
name = "khepri-app"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "khepri"
path = "src/main.rs"

[dependencies]
khepri-core = { path = "../khepri-core" }
khepri-editor = { path = "../khepri-editor" }
eframe = { version = "0.34", default-features = false, features = ["glow", "accesskit", "default_fonts", "wayland", "x11", "persistence"] }
```

```toml
# Root Cargo.toml (workspace)
[workspace]
members = [
    "crates/khepri-core",
    "crates/khepri-editor",
    "crates/khepri-app",
]
resolver = "2"
```

## Config Module (`khepri-core/src/config.rs`)

All shared constants live here:
- `BG_COLOR`, `FG_COLOR` (moved from main.rs)
- `HOVER_COLOR` (moved from title_bar.rs)
- `PANEL_RADIUS`, `PANEL_GAP`, `PANEL_PADDING` (moved from main.rs)
- `TITLE_BAR_HEIGHT`, `BUTTON_WIDTH`, `BUTTON_HEIGHT` (moved from title_bar.rs)

This is the single source of truth for visual constants. Every crate that needs them depends on `khepri-core`.

## Stub Modules

Each stub is minimal — just enough to establish the module:

```rust
// khepri-core/src/renderer.rs
//! 2D rendering backend. Future: wgpu-based sprite/shape rendering.

pub struct Renderer;
```

```rust
// khepri-core/src/scene.rs
//! Scene graph for managing game objects and their transforms.

pub struct Scene;
```

```rust
// khepri-core/src/ecs.rs
//! Entity-Component-System for game object management.

pub struct World;
```

```rust
// khepri-core/src/assets.rs
//! Asset loading and management.

pub struct AssetManager;
```

## Logging

Add `tracing` to `khepri-app`:
```toml
tracing = "0.1"
tracing-subscriber = "0.3"
```

Initialize in `main()` before `eframe::run_native`:
```rust
tracing_subscriber::fmt::init();
```

Future crates add `tracing` when they need logging.

## Verification

```bash
cargo build                    # workspace builds all 3 crates, 0 errors
cargo run -p khepri-app        # window opens, everything works as before
cargo test -p khepri-core      # engine core compiles independently
cargo doc --workspace --open   # docs show full architecture
```

## Task Breakdown

### Task 1: Create workspace structure
- Create `crates/khepri-core/`, `crates/khepri-editor/`, `crates/khepri-app/` directories
- Write root `Cargo.toml` (workspace only)
- Write each crate's `Cargo.toml`

### Task 2: Move config to `khepri-core`
- Create `khepri-core/src/lib.rs` with `pub mod config;` and re-export stubs
- Create `khepri-core/src/config.rs` with all shared constants
- Create stub modules (renderer, scene, ecs, assets)

### Task 3: Move editor code to `khepri-editor`
- Create `khepri-editor/src/lib.rs`
- Move `title_bar.rs` (update imports to `khepri_core::config::*`)
- Create `panels.rs` (extract `BentoLayout` + `draw_bento()` from main.rs)
- Create `window_resize.rs` (extract `custom_window_resize()` from main.rs)

### Task 4: Create app entry point
- Create `khepri-app/src/main.rs` (just `KhepriApp` struct + `main()`)
- Add `tracing` + `tracing-subscriber` dependencies
- Add logging initialization

### Task 5: Clean up and verify
- Delete old `src/` directory
- Delete empty `src/ui/` directory
- `cargo build` — 0 errors, 0 warnings
- `cargo run -p khepri-app` — window works, panels work, title bar works, resize works
- `cargo doc --workspace --open` — architecture visible
