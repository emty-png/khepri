use eframe::egui;

pub fn custom_window_resize(ui: &mut egui::Ui) {
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if is_maximized {
        return;
    }

    let rect = ui.ctx().content_rect();
    let border = 6.0;

    let tl_rect = egui::Rect::from_min_max(rect.min, rect.min + egui::vec2(border, border));
    let tr_rect = egui::Rect::from_min_max(
        egui::pos2(rect.max.x - border, rect.min.y),
        egui::pos2(rect.max.x, rect.min.y + border),
    );
    let bl_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, rect.max.y - border),
        egui::pos2(rect.min.x + border, rect.max.y),
    );
    let br_rect = egui::Rect::from_min_max(rect.max - egui::vec2(border, border), rect.max);

    let l_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, rect.min.y + border),
        egui::pos2(rect.min.x + border, rect.max.y - border),
    );
    let r_rect = egui::Rect::from_min_max(
        egui::pos2(rect.max.x - border, rect.min.y + border),
        egui::pos2(rect.max.x, rect.max.y - border),
    );
    let t_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + border, rect.min.y),
        egui::pos2(rect.max.x - border, rect.min.y + border),
    );
    let b_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + border, rect.max.y - border),
        egui::pos2(rect.max.x - border, rect.max.y),
    );

    use egui::viewport::ResizeDirection;
    let resize_areas = [
        (
            tl_rect,
            ResizeDirection::NorthWest,
            egui::CursorIcon::ResizeNwSe,
        ),
        (
            tr_rect,
            ResizeDirection::NorthEast,
            egui::CursorIcon::ResizeNeSw,
        ),
        (
            bl_rect,
            ResizeDirection::SouthWest,
            egui::CursorIcon::ResizeNeSw,
        ),
        (
            br_rect,
            ResizeDirection::SouthEast,
            egui::CursorIcon::ResizeNwSe,
        ),
        (l_rect, ResizeDirection::West, egui::CursorIcon::ResizeWest),
        (r_rect, ResizeDirection::East, egui::CursorIcon::ResizeEast),
        (
            t_rect,
            ResizeDirection::North,
            egui::CursorIcon::ResizeNorth,
        ),
        (
            b_rect,
            ResizeDirection::South,
            egui::CursorIcon::ResizeSouth,
        ),
    ];

    for (r, dir, cursor) in resize_areas {
        let id = egui::Id::new(format!("resize_{:?}", dir));
        let response = ui.interact(r, id, egui::Sense::drag());

        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(cursor);
        }

        if response.dragged_by(egui::PointerButton::Primary) {
            ui.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
        }
    }
}
