# Plan: Bento Panel Layout

## Decision

4 floating panels on cream background with 15px gaps, 20px corner radius, black fill, 12px inner padding. Resizable via splitter bars (dragging gaps between panels).

## Layout

```
+----------+---+-----------+---+----------+
|          |   |           |   |          |
|  Left    |   |  Center   |   |  Right   |  <- 70% height
| (slim)   |15 |  (wide)   |15 | (slim)   |
|          |px |           |px |          |
+----------+---+-----------+---+----------+
|          15px gap          15px          |
+-----------------------------------------+
|                                         |
|           Bottom (full width)           |  <- 30% height
|                                         |
+-----------------------------------------+
```

- Top row: 3 columns. Left/Right ~20% width each, Center ~60%. Split ratios stored in app state.
- Bottom row: full width. 30% of content height (after title bar).
- 15px gaps between all panels.
- Each panel: black fill, 20px corner radius, 12px inner padding.
- Splitter bars: dragging the 15px gap between panels adjusts the split ratios.

## Files

```
src/
  main.rs       — Add BentoLayout struct with split state, layout logic, splitter drag handling
```

One file modified. `title_bar.rs` unchanged.

## State (in KhepriApp)

```rust
struct BentoLayout {
    // Top row: left/center/right split (0.0-1.0 ratios)
    left_ratio: f32,    // default 0.20
    right_ratio: f32,   // default 0.20
    // Vertical split: top/bottom
    top_ratio: f32,     // default 0.70
}
```

## Layout Logic

1. After title bar, get remaining content rect.
2. Split vertically at `top_ratio` -> top_rect, bottom_rect with 15px gap.
3. Split top_rect horizontally:
   - left_width = top_width * left_ratio
   - right_width = top_width * right_ratio
   - center_width = top_width - left_width - right_width - 2*gap
4. Shrink each panel rect by 15px gap on all sides (floating effect).
5. Paint each panel: `rect_filled` with black, corner radius 20, then offset content by 12px padding.

## Splitter Bars

4 draggable zones (each 15px wide/tall):
1. **Vertical splitter** (gap between top and bottom rows): drag up/down changes `top_ratio`.
2. **Left/Center splitter**: drag left/right changes `left_ratio`.
3. **Center/Right splitter**: drag left/right changes `right_ratio`.

Each splitter: invisible rect in the gap, drag changes the corresponding ratio, clamped to min/max.

## Constants

```rust
const PANEL_RADIUS: egui::CornerRadius = egui::CornerRadius::same(20);
const PANEL_FILL: egui::Color32 = egui::Color32::BLACK;
const PANEL_GAP: f32 = 15.0;
const PANEL_PADDING: f32 = 12.0;
```

## Verification

```bash
cargo build   # 0 errors, 0 warnings
cargo run     # 4 floating black panels with gaps, resizable via drag
```
