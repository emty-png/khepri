use eframe::egui;
use khepri_core::config;
use khepri_core::scene::{Scene, ShapeType};

pub fn show_hierarchy(ui: &mut egui::Ui, scene: &mut Scene) {
    let fg = egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
    let hover_bg = egui::Color32::from_rgb(
        config::HOVER_COLOR_R,
        config::HOVER_COLOR_G,
        config::HOVER_COLOR_B,
    );
    let normal_bg = egui::Color32::from_rgba_premultiplied(255, 255, 255, 15);
    let bg = egui::Color32::from_rgb(config::BG_COLOR_R, config::BG_COLOR_G, config::BG_COLOR_B);

    // ── Tab-style header ────────────────────────────────────────────────────
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let tab_text = egui::RichText::new("Hierarchy")
            .size(11.0)
            .strong()
            .color(fg);
        let galley = ui.painter().layout_no_wrap(
            tab_text.text().to_string(),
            egui::FontId::proportional(11.0),
            fg,
        );
        let text_size = galley.size();
        let padding = egui::vec2(10.0, 6.0);
        let tab_size = text_size + padding * 2.0;
        let tab_rect = egui::Rect::from_min_size(ui.cursor().min, tab_size);
        ui.painter()
            .rect_filled(tab_rect, egui::CornerRadius::same(4), normal_bg);
        ui.painter()
            .galley(tab_rect.left_top() + padding, galley, fg);
        ui.advance_cursor_after_rect(tab_rect);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Styled "+" button: 20x20, cream bg, black "+"
            let btn_size = egui::vec2(20.0, 20.0);
            let (btn_rect, btn_response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
            let btn_bg = if btn_response.hovered() { hover_bg } else { bg };
            ui.painter()
                .rect_filled(btn_rect, egui::CornerRadius::same(6), btn_bg);
            ui.painter().text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::proportional(14.0),
                fg,
            );

            // Manual popup state (avoid egui Popup system interference)
            let state_id = egui::Id::new("add_popup_open");
            let mut is_open: bool = ui.data(|d| d.get_temp(state_id).unwrap_or(false));

            if btn_response.clicked() {
                is_open = !is_open;
                ui.data_mut(|d| d.insert_temp(state_id, is_open));
            }

            if is_open {
                let popup_width = 124.0;
                let popup_height = 94.0;
                let margin = egui::vec2(8.0, 8.0);
                let popup_pos = egui::pos2(btn_rect.right() - popup_width, btn_rect.bottom() + 4.0);
                let outer_rect =
                    egui::Rect::from_min_size(popup_pos, egui::vec2(popup_width, popup_height));
                let inner_rect = outer_rect.shrink2(margin);

                // Everything rendered inside one Area at Foreground order
                let mut close_popup = false;
                egui::Area::new(egui::Id::new("add_popup_area"))
                    .fixed_pos(popup_pos)
                    .order(egui::Order::Foreground)
                    .interactable(true)
                    .show(ui.ctx(), |area_ui| {
                        area_ui.set_min_size(egui::vec2(popup_width, popup_height));

                        // Background + border
                        area_ui.painter().rect_filled(
                            outer_rect,
                            egui::CornerRadius::same(8),
                            egui::Color32::from_rgb(0x1A, 0x1A, 0x1A),
                        );
                        area_ui.painter().rect_stroke(
                            outer_rect,
                            egui::CornerRadius::same(8),
                            egui::Stroke::new(2.0, fg),
                            egui::StrokeKind::Outside,
                        );

                        // Buttons inside inner area
                        area_ui.scope_builder(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                            for shape in
                                [ShapeType::Rectangle, ShapeType::Circle, ShapeType::Triangle]
                            {
                                let item_text =
                                    egui::RichText::new(shape.name()).size(11.0).color(fg);
                                let item_btn = egui::Button::new(item_text)
                                    .fill(bg)
                                    .stroke(egui::Stroke::new(1.0, fg))
                                    .corner_radius(egui::CornerRadius::same(6));
                                if ui.add_sized([inner_rect.width(), 22.0], item_btn).clicked() {
                                    scene.add_object(shape);
                                    close_popup = true;
                                }
                            }
                        });
                    });

                if close_popup {
                    ui.data_mut(|d| d.insert_temp(state_id, false));
                }

                // Close on outside click
                let pointer = ui.input(|i| i.pointer.any_pressed());
                if pointer
                    && !outer_rect
                        .contains(ui.input(|i| i.pointer.interact_pos().unwrap_or_default()))
                    && !btn_rect
                        .contains(ui.input(|i| i.pointer.interact_pos().unwrap_or_default()))
                {
                    ui.data_mut(|d| d.insert_temp(state_id, false));
                }
            }
        });
    });

    ui.add_space(4.0);
    ui.separator();

    // Object list
    let mut clicked_id = None;
    for obj in &scene.objects {
        let is_selected = scene.selected_id == Some(obj.id);

        let item_bg = if is_selected { hover_bg } else { normal_bg };
        let text = egui::RichText::new(&obj.name).size(11.0).color(fg);
        let btn = egui::Button::new(text)
            .fill(item_bg)
            .corner_radius(egui::CornerRadius::same(6));

        if ui.add_sized([ui.available_width(), 22.0], btn).clicked() {
            clicked_id = Some(obj.id);
        }
    }

    if let Some(id) = clicked_id {
        scene.select(Some(id));
    }
}
