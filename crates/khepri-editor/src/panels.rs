use crate::{hierarchy, properties, viewport};
use eframe::egui;
use khepri_core::config;
use khepri_core::scene::Scene;

pub struct BentoLayout {
    pub left_ratio: f32,
    pub right_ratio: f32,
    pub top_ratio: f32,
}

impl Default for BentoLayout {
    fn default() -> Self {
        Self {
            left_ratio: 0.20,
            right_ratio: 0.20,
            top_ratio: 0.70,
        }
    }
}

pub fn draw_bento(ui: &mut egui::Ui, bento: &mut BentoLayout, scene: &mut Scene) {
    let panel_bg =
        egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
    let rect = ui.available_rect_before_wrap();
    let split_y = rect.top() + rect.height() * bento.top_ratio;
    let total_w = rect.width();
    let left_w = total_w * bento.left_ratio;
    let right_w = total_w * bento.right_ratio;
    let gap = config::PANEL_GAP;
    let radius = egui::CornerRadius::same(config::PANEL_RADIUS);

    let top_y_min = rect.min.y + gap / 2.0;
    let top_y_max = split_y - gap / 2.0;

    let left_panel = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + gap / 2.0, top_y_min),
        egui::pos2(rect.min.x + left_w - gap / 2.0, top_y_max),
    );
    let center_panel = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + left_w + gap / 2.0, top_y_min),
        egui::pos2(rect.max.x - right_w - gap / 2.0, top_y_max),
    );
    let right_panel = egui::Rect::from_min_max(
        egui::pos2(rect.max.x - right_w + gap / 2.0, top_y_min),
        egui::pos2(rect.max.x - gap / 2.0, top_y_max),
    );
    let bottom_panel = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + gap / 2.0, split_y + gap / 2.0),
        egui::pos2(rect.max.x - gap / 2.0, rect.max.y - gap / 2.0),
    );

    // Paint panel backgrounds
    let painter = ui.painter();
    for panel in [&left_panel, &center_panel, &right_panel, &bottom_panel] {
        painter.rect_filled(*panel, radius, panel_bg);
    }

    // --- Render panel content ---

    // Left: Hierarchy
    {
        let builder = egui::UiBuilder::new()
            .max_rect(left_panel)
            .layout(egui::Layout::top_down(egui::Align::LEFT));
        let mut child_ui = ui.new_child(builder);
        egui::Frame::new()
            .inner_margin(config::PANEL_PADDING)
            .show(&mut child_ui, |ui| {
                hierarchy::show_hierarchy(ui, scene);
            });
    }

    // Center: Viewport
    {
        let builder = egui::UiBuilder::new()
            .max_rect(center_panel)
            .layout(egui::Layout::top_down(egui::Align::LEFT));
        let mut child_ui = ui.new_child(builder);
        viewport::show_viewport(&mut child_ui, scene);
    }

    // Right: Properties
    {
        let builder = egui::UiBuilder::new()
            .max_rect(right_panel)
            .layout(egui::Layout::top_down(egui::Align::LEFT));
        let mut child_ui = ui.new_child(builder);
        egui::Frame::new()
            .inner_margin(config::PANEL_PADDING)
            .show(&mut child_ui, |ui| {
                properties::show_properties(ui, scene);
            });
    }

    // Bottom panel: empty for now

    // --- Splitter bars ---

    // Vertical splitter
    let vert_split = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, split_y - gap / 2.0),
        egui::pos2(rect.max.x, split_y + gap / 2.0),
    );
    let resp = ui.interact(vert_split, egui::Id::new("split_vert"), egui::Sense::drag());
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeSouth);
    }
    if resp.dragged() {
        if let Some(pointer) = ui.input(|i| i.pointer.latest_pos()) {
            bento.top_ratio = ((pointer.y - rect.top()) / rect.height()).clamp(0.2, 0.8);
        }
    }

    // Left/Center splitter
    let lc_x = rect.min.x + left_w;
    let lc_split = egui::Rect::from_min_max(
        egui::pos2(lc_x - gap / 2.0, top_y_min),
        egui::pos2(lc_x + gap / 2.0, top_y_max),
    );
    let resp = ui.interact(lc_split, egui::Id::new("split_lc"), egui::Sense::drag());
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeEast);
    }
    if resp.dragged() {
        if let Some(pointer) = ui.input(|i| i.pointer.latest_pos()) {
            bento.left_ratio = ((pointer.x - rect.min.x) / rect.width()).clamp(0.1, 0.7);
        }
    }

    // Center/Right splitter
    let cr_x = rect.max.x - right_w;
    let cr_split = egui::Rect::from_min_max(
        egui::pos2(cr_x - gap / 2.0, top_y_min),
        egui::pos2(cr_x + gap / 2.0, top_y_max),
    );
    let resp = ui.interact(cr_split, egui::Id::new("split_cr"), egui::Sense::drag());
    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeEast);
    }
    if resp.dragged() {
        if let Some(pointer) = ui.input(|i| i.pointer.latest_pos()) {
            bento.right_ratio = ((rect.max.x - pointer.x) / rect.width()).clamp(0.1, 0.7);
        }
    }
}
