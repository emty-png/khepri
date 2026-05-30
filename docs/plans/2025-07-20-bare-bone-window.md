# Plan: Bare-Bone Khepri Window

## Decision

**Stack:** `eframe` 0.34.3 with **wgpu** backend (default), custom title bar via egui `TopBottomPanel`.

**Window persistence:** eframe's built-in `persist_window: true` — saves/restores window position and size automatically to the platform config directory.

**Why eframe over raw winit+wgpu:**
- eframe wraps winit + wgpu + egui into a single `App` trait — eliminates ~200 lines of boilerplate (instance, adapter, device, surface, event loop).
- `persist_window` is a single boolean — no manual serde/config file needed.
- `frame.winit_window()?.drag_window()` gives us the winit Window handle for custom title bar drag.
- wgpu backend is the default — same GPU device will be available for the 2D engine later.

## Color Palette

| Role | Hex | Usage |
|------|-----|-------|
| Primary | `#FFFDDD` | Window background, title bar background |
| Secondary | `#000000` | Title bar text, button icons, borders |

## Files

```
src/
  main.rs          — App struct, eframe::App impl, entry point, color constants
  title_bar.rs     — Custom title bar widget (drag region, min/max/close buttons)
```

Two files only. No modules, no abstraction layers. The title bar is a pure function that takes `&egui::Context` and `&mut bool` (should_close flag).

## Implementation

### 1. Cargo.toml

```toml
[package]
name = "khepri"
version = "0.1.0"
edition = "2024"

[dependencies]
eframe = { version = "0.34", features = ["default"] }
```

eframe 0.34 defaults: wgpu renderer, winit, accesskit, wayland, x11, default_fonts, persistence.

### 2. src/main.rs

```rust
mod title_bar;

use eframe::egui;

const BG_COLOR: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFD, 0xDD); // #FFFDDD
const FG_COLOR: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00); // #000000

struct KhepriApp;

impl KhepriApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }
}

impl eframe::App for KhepriApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut should_close = false;
        title_bar::show(ctx, frame, &mut should_close);
        if should_close {
            frame.close();
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG_COLOR))
            .show(ctx, |ui| {
                // Empty for now — bare bone
            });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([400.0, 300.0])
            .with_decorations(false),   // hide OS title bar
        persist_window: true,           // auto-save/restore position & size
        ..Default::default()
    };

    eframe::run_native(
        "Khepri",
        native_options,
        Box::new(|cc| Ok(Box::new(KhepriApp::new(cc)))),
    )
}
```

**What happens on launch:**
1. eframe reads persisted window position/size from `{config_dir}/khepri/window.json` (RON format).
2. If found, window appears at saved position/size. If not, defaults to 1280x720.
3. On close, eframe writes current position/size back to that file.

### 3. src/title_bar.rs

```rust
use eframe::egui;
use crate::FG_COLOR;

const TITLE_BAR_HEIGHT: f32 = 32.0;
const BUTTON_SIZE: f32 = TITLE_BAR_HEIGHT;

/// Returns `true` if the window should close.
pub fn show(ctx: &egui::Context, frame: &mut eframe::Frame, should_close: &mut bool) {
    egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
        let rect = ui.available_rect_before_wrap();
        let title_bar_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(rect.width(), TITLE_BAR_HEIGHT),
        );

        let title_bar_response = ui.interact(
            title_bar_rect,
            egui::Id::new("title_bar"),
            egui::Sense::click_and_drag(),
        );

        // Drag window on title bar click-drag
        if title_bar_response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(window) = frame.winit_window() {
                let _ = window.drag_window();
            }
        }

        // Double-click to maximize/restore
        if title_bar_response.double_clicked() {
            if let Some(window) = frame.winit_window() {
                window.set_maximized(!window.is_maximized());
            }
        }

        // Paint title bar background
        ui.painter().rect_filled(title_bar_rect, 0.0, crate::BG_COLOR);

        // Title text (left)
        ui.painter().text(
            title_bar_rect.left_center() + egui::vec2(12.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "Khepri",
            egui::FontId::proportional(14.0),
            FG_COLOR,
        );

        // Window controls (right)
        let mut x = title_bar_rect.right();

        // Close button
        x -= BUTTON_SIZE;
        let close_rect = egui::Rect::from_min_size(
            egui::pos2(x, title_bar_rect.min.y),
            egui::vec2(BUTTON_SIZE, BUTTON_SIZE),
        );
        let close_response = ui.interact(
            close_rect,
            egui::Id::new("btn_close"),
            egui::Sense::click(),
        );
        let close_color = if close_response.hovered() {
            egui::Color32::from_rgb(0xC4, 0x2B, 0x1C) // Windows-style red hover
        } else {
            crate::BG_COLOR
        };
        ui.painter().rect_filled(close_rect, 0.0, close_color);
        paint_x_icon(ui, close_rect.center(), FG_COLOR);
        if close_response.clicked() {
            *should_close = true;
        }

        // Maximize button
        x -= BUTTON_SIZE;
        let max_rect = egui::Rect::from_min_size(
            egui::pos2(x, title_bar_rect.min.y),
            egui::vec2(BUTTON_SIZE, BUTTON_SIZE),
        );
        let max_response = ui.interact(
            max_rect,
            egui::Id::new("btn_max"),
            egui::Sense::click(),
        );
        let max_color = if max_response.hovered() {
            egui::Color32::from_rgb(0xE0, 0xE0, 0xE0)
        } else {
            crate::BG_COLOR
        };
        ui.painter().rect_filled(max_rect, 0.0, max_color);
        let is_maximized = frame
            .winit_window()
            .map(|w| w.is_maximized())
            .unwrap_or(false);
        paint_maximize_icon(ui, max_rect.center(), FG_COLOR, is_maximized);
        if max_response.clicked() {
            if let Some(window) = frame.winit_window() {
                window.set_maximized(!is_maximized);
            }
        }

        // Minimize button
        x -= BUTTON_SIZE;
        let min_rect = egui::Rect::from_min_size(
            egui::pos2(x, title_bar_rect.min.y),
            egui::vec2(BUTTON_SIZE, BUTTON_SIZE),
        );
        let min_response = ui.interact(
            min_rect,
            egui::Id::new("btn_min"),
            egui::Sense::click(),
        );
        let min_color = if min_response.hovered() {
            egui::Color32::from_rgb(0xE0, 0xE0, 0xE0)
        } else {
            crate::BG_COLOR
        };
        ui.painter().rect_filled(min_rect, 0.0, min_color);
        paint_minimize_icon(ui, min_rect.center(), FG_COLOR);
        if min_response.clicked() {
            if let Some(window) = frame.winit_window() {
                window.set_minimized(true);
            }
        }
    });
}

fn paint_x_icon(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let size = 10.0;
    let stroke = egui::Stroke::new(1.5, color);
    painter.line_segment(
        [
            center + egui::vec2(-size / 2.0, -size / 2.0),
            center + egui::vec2(size / 2.0, size / 2.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center + egui::vec2(size / 2.0, -size / 2.0),
            center + egui::vec2(-size / 2.0, size / 2.0),
        ],
        stroke,
    );
}

fn paint_maximize_icon(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32, is_maximized: bool) {
    let size = 10.0;
    let stroke = egui::Stroke::new(1.5, color);
    if is_maximized {
        // Overlapping rectangles (restore icon)
        painter.rect_stroke(
            egui::Rect::from_center_size(center + egui::vec2(-2.0, 2.0), egui::vec2(size, size)),
            0.0,
            stroke,
        );
        painter.rect_stroke(
            egui::Rect::from_center_size(center + egui::vec2(2.0, -2.0), egui::vec2(size, size)),
            0.0,
            stroke,
        );
    } else {
        painter.rect_stroke(
            egui::Rect::from_center_size(center, egui::vec2(size, size)),
            0.0,
            stroke,
        );
    }
}

fn paint_minimize_icon(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.5, color);
    painter.line_segment(
        [
            center + egui::vec2(-6.0, 0.0),
            center + egui::vec2(6.0, 0.0),
        ],
        stroke,
    );
}
```

## What You Get

1. **Frameless window** — no OS chrome, `with_decorations(false)`.
2. **Custom title bar** — 32px tall, `#FFFDDD` background, "Khepri" text in `#000000`, min/max/close buttons on the right with hover effects.
3. **Drag to move** — click-drag on the title bar moves the window via `drag_window()`.
4. **Double-click to maximize** — double-click the title bar to toggle maximize/restore.
5. **Window state persistence** — `persist_window: true` auto-saves position and size on close, restores on next launch.
6. **wgpu rendering** — GPU-accelerated, same device will be shared with the 2D engine later.

## Verification

```bash
cargo build          # Must compile with 0 errors
cargo run            # Window opens at 1280x720 with custom title bar
# Drag the title bar — window moves
# Click minimize — window minimizes to taskbar
# Click maximize — window fills screen, icon changes to restore
# Close the window
# cargo run again — window reopens at the same position/size
```

## What's NOT In This Plan (Future Work)

- Sidebar / scene hierarchy
- Game viewport / renderer integration
- Custom window resize handles (OS handles resize for decorated windows; frameless windows need manual resize — eframe handles this via `with_resizable(true)` in ViewportBuilder)
- Theming system beyond the two-color palette
