# Khepri Hub -- Design Spec

## Goal

Create a "Khepri Hub" window that opens when the application starts. For now, it's an empty window with the Khepri theme. Later it will host project management (create, open, recent projects).

## Architecture

### New crate: `crates/khepri-hub/`

A separate crate following the same pattern as `khepri-editor`. Contains the Hub UI.

```
crates/khepri-hub/
  Cargo.toml
  src/
    lib.rs        -- pub mod hub; pub use hub::HubApp;
    hub.rs        -- HubApp struct implementing eframe::App
```

### Changes to `khepri-app`

The binary now launches the Hub instead of the Editor.

```
crates/khepri-app/src/main.rs
  - Replace KhepriApp (editor) with HubApp (hub)
  - Window title: "Khepri Hub"
  - Window size: 900x600 (smaller than editor)
  - Same theme: BG_COLOR background, no decorations, resizable
```

### Workspace `Cargo.toml`

Add `khepri-hub` to workspace members.

### What the Hub shows (v0)

- Full-window BG_COLOR background
- Centered text: "Khepri Hub" in FG_COLOR
- Subtitle: "No projects yet" in muted text
- That's it. No buttons, no project list, no file dialogs.

### What stays untouched

- `khepri-core` -- no changes
- `khepri-editor` -- no changes (editor binary is preserved, just not launched)
- Bottom panel -- kept for future use

### Binary strategy

For now, `khepri-app` binary launches the Hub. Later we can:
- Add a `--editor` flag to skip the hub
- Or split into two binaries (`khepri-hub`, `khepri-editor-bin`)
- Or have the Hub spawn the editor in the same process

## Files to create/modify

| Action | File | What |
|--------|------|------|
| Create | `crates/khepri-hub/Cargo.toml` | Crate manifest |
| Create | `crates/khepri-hub/src/lib.rs` | Module declarations |
| Create | `crates/khepri-hub/src/hub.rs` | HubApp struct + UI |
| Modify | `Cargo.toml` (workspace) | Add khepri-hub to members |
| Modify | `crates/khepri-app/Cargo.toml` | Add khepri-hub dependency |
| Modify | `crates/khepri-app/src/main.rs` | Launch HubApp instead of editor |

## Verification

1. `cargo build` -- 0 errors, 0 warnings
2. `cargo test -p khepri-core` -- 6/6 pass (unchanged)
3. `cargo clippy --workspace -- -D warnings` -- 0 warnings
4. `cargo fmt --all -- --check` -- clean
5. `cargo run -p khepri-app` -- Hub window opens with "Khepri Hub" text
6. Editor still compiles (not launched, but builds clean)
