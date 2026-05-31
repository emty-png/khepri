use eframe::egui;
use khepri_core::config;

pub fn show(ui: &mut egui::Ui) {
    let bg = egui::Color32::from_rgb(config::BG_COLOR_R, config::BG_COLOR_G, config::BG_COLOR_B);
    let fg = egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
    let hover = egui::Color32::from_rgb(
        config::HOVER_COLOR_R,
        config::HOVER_COLOR_G,
        config::HOVER_COLOR_B,
    );

    egui::Panel::top("title_bar")
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            ui.set_min_height(config::TITLE_BAR_HEIGHT);
            let rect = ui.available_rect_before_wrap();
            let title_bar_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(rect.width(), config::TITLE_BAR_HEIGHT),
            );
            let title_bar_response = ui.interact(
                title_bar_rect,
                egui::Id::new("title_bar"),
                egui::Sense::click_and_drag(),
            );

            if title_bar_response.drag_started_by(egui::PointerButton::Primary) {
                ui.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            if title_bar_response.double_clicked() {
                let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                ui.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
            }

            ui.painter().rect_filled(title_bar_rect, 0.0, bg);

            ui.painter().line_segment(
                [
                    egui::pos2(title_bar_rect.left(), title_bar_rect.bottom()),
                    egui::pos2(title_bar_rect.right(), title_bar_rect.bottom()),
                ],
                egui::Stroke::new(1.5, fg),
            );

            ui.painter().text(
                title_bar_rect.left_center() + egui::vec2(12.0, 0.0),
                egui::Align2::LEFT_CENTER,
                "Khepri",
                egui::FontId::proportional(13.0),
                fg,
            );

            let mut x = title_bar_rect.right();

            // --- Close ---
            x -= config::BUTTON_WIDTH;
            let close_rect = egui::Rect::from_min_size(
                egui::pos2(x, title_bar_rect.min.y),
                egui::vec2(config::BUTTON_WIDTH, config::BUTTON_HEIGHT),
            );
            let close_response =
                ui.interact(close_rect, egui::Id::new("btn_close"), egui::Sense::click());

            let close_hovered = close_response.hovered();
            let close_bg = if close_hovered {
                egui::Color32::from_rgb(0xC4, 0x2B, 0x1C)
            } else {
                bg
            };
            let close_fg = if close_hovered {
                egui::Color32::WHITE
            } else {
                fg
            };

            ui.painter().rect_filled(close_rect, 0.0, close_bg);
            paint_x_icon(ui.painter(), close_rect.center(), close_fg);

            if close_response.clicked() {
                ui.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            // --- Maximize / Restore ---
            x -= config::BUTTON_WIDTH;
            let max_rect = egui::Rect::from_min_size(
                egui::pos2(x, title_bar_rect.min.y),
                egui::vec2(config::BUTTON_WIDTH, config::BUTTON_HEIGHT),
            );
            let max_response =
                ui.interact(max_rect, egui::Id::new("btn_max"), egui::Sense::click());

            let max_hovered = max_response.hovered();
            let max_bg = if max_hovered { hover } else { bg };

            ui.painter().rect_filled(max_rect, 0.0, max_bg);
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            if is_maximized {
                paint_restore_icon(ui.painter(), max_rect.center(), fg, max_bg);
            } else {
                paint_maximize_icon(ui.painter(), max_rect.center(), fg);
            }

            if max_response.clicked() {
                ui.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
            }

            // --- Minimize ---
            x -= config::BUTTON_WIDTH;
            let min_rect = egui::Rect::from_min_size(
                egui::pos2(x, title_bar_rect.min.y),
                egui::vec2(config::BUTTON_WIDTH, config::BUTTON_HEIGHT),
            );
            let min_response =
                ui.interact(min_rect, egui::Id::new("btn_min"), egui::Sense::click());

            let min_hovered = min_response.hovered();
            let min_bg = if min_hovered { hover } else { bg };

            ui.painter().rect_filled(min_rect, 0.0, min_bg);
            paint_minimize_icon(ui.painter(), min_rect.center(), fg);

            if min_response.clicked() {
                ui.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
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

fn paint_maximize_icon(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let size = 10.0;
    let stroke = egui::Stroke::new(1.5, color);
    painter.rect_stroke(
        egui::Rect::from_center_size(center, egui::vec2(size, size)),
        egui::CornerRadius::same(0),
        stroke,
        egui::StrokeKind::Middle,
    );
}

fn paint_restore_icon(
    painter: &egui::Painter,
    center: egui::Pos2,
    color: egui::Color32,
    bg_color: egui::Color32,
) {
    let size = 8.0;
    let stroke = egui::Stroke::new(1.5, color);
    let corner = egui::CornerRadius::same(0);
    let kind = egui::StrokeKind::Middle;

    painter.rect_stroke(
        egui::Rect::from_center_size(center + egui::vec2(1.5, -1.5), egui::vec2(size, size)),
        corner,
        stroke,
        kind,
    );
    let front =
        egui::Rect::from_center_size(center + egui::vec2(-1.5, 1.5), egui::vec2(size, size));
    painter.rect_filled(front, corner, bg_color);
    painter.rect_stroke(front, corner, stroke, kind);
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
