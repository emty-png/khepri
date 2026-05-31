# Khepri Roadmap

## v0.1.0 -- Editor Shell (Current)

- [x] Workspace structure (core, editor, app)
- [x] Scene data model with unit tests
- [x] Editor UI: hierarchy, viewport, properties panels
- [x] Object creation, selection, deletion
- [x] Click-to-select, drag-to-move
- [x] Styled UI with custom theme
- [x] Editable object names

## v0.2.0 -- Save and Load

- [ ] Scene serialization (JSON or RON)
- [ ] Save/load from file
- [ ] Recent files

## v0.3.0 -- Undo/Redo

- [ ] Command pattern for undo/redo
- [ ] Keyboard shortcuts (Ctrl+Z, Ctrl+Y)

## v0.4.0 -- Renderer Backend

- [ ] Swap egui Shape API for wgpu renderer
- [ ] GPU-accelerated shape rendering
- [ ] Texture support

## v0.5.0 -- ECS

- [ ] Entity-component-system for scene objects
- [ ] Component system for transform, render, behavior

## v0.6.0 -- Asset Pipeline

- [ ] Texture loading
- [ ] Font loading
- [ ] Sound loading

## v1.0.0 -- Game Runtime

- [ ] Play mode (run scenes as games)
- [ ] Scripting (Lua or WASM)
- [ ] Physics (rapier or custom)
