# Khepri

> **Maintained by Emty** -- a coding agent building a pure Rust 2D game engine with minimal human supervision.

Khepri is a 2D scene editor and game engine written in Rust. The goal is to create a fast, clean, and extensible engine that developers can use to build 2D games without the bloat of traditional engines.

---

## Project Status

**Early development.** The editor is functional but not feature-complete. The scene data model, editor UI (hierarchy, viewport, properties panels), and basic object manipulation are working. Rendering is currently handled via egui's Shape API.

---

## Architecture

Khepri is organized as a Cargo workspace with three crates:

```
crates/
  khepri-core/      -- Pure Rust, no UI. Scene data model, config, stub modules for renderer/ecs/assets.
  khepri-editor/    -- Editor UI panels (hierarchy, viewport, properties) using egui.
  khepri-app/       -- Binary entry point. Window creation, tracing, app loop.
```

### Why a workspace?

Separation of concerns. The core engine logic (`khepri-core`) has zero UI dependencies. The editor (`khepri-editor`) depends on egui but not on core internals. The app crate (`khepri-app`) wires everything together. This makes it possible to swap out the renderer, editor framework, or platform layer independently.

---

## Getting Started

### Prerequisites

- Rust 1.85+ (edition 2024)
- No external dependencies -- egui and eframe handle everything via pure Rust.

### Build and Run

```bash
# Build the workspace
cargo build

# Run the application
cargo run -p khepri-app

# Run core tests
cargo test -p khepri-core
```

### Verified Targets

- Windows 10/11 (x86_64)
- Linux (Wayland, X11)

---

## Contributing

Contributions are welcome. The codebase is small and readable. If you want to help:

1. **Fork and branch.** Create a feature branch from `master`.
2. **Keep it clean.** `cargo build` must produce zero errors and zero warnings. `cargo test -p khepri-core` must pass.
3. **Write tests.** If you change core logic, update or add tests in `khepri-core`.
4. **Match the style.** Look at existing code. Same naming, same patterns, same error handling.
5. **Open a PR.** Describe what you changed and why. Screenshots are welcome for UI changes.

### What needs work

- Renderer backend (swap egui Shape API for a proper GPU renderer)
- ECS (entity-component-system) for scene objects
- Asset loading (textures, fonts, sounds)
- Serialization (save/load scenes)
- Undo/redo
- Keyboard shortcuts
- Multi-select and transform gizmos

---

## Roadmap

See [`docs/roadmap.md`](docs/roadmap.md) for the planned feature timeline.

---

## License

MIT

---

## About Emty

Emty is a coding agent that maintains and builds Khepri. Every line of code in this repository was written, reviewed, and verified by Emty before being committed. The goal is to demonstrate that a coding agent can build and maintain a real, production-quality Rust project with minimal human supervision.

Human involvement is limited to feature requests, visual direction, and high-level decisions. Everything else -- architecture, implementation, testing, debugging, documentation -- is handled by Emty.
