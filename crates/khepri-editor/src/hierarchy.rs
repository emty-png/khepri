use eframe::egui;
use khepri_core::config;
use khepri_core::scene::{Scene, ShapeType};

pub fn show_hierarchy(ui: &mut egui::Ui, scene: &mut Scene) {
    let fg = egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
    let hover_bg = egui::Color32::from_rgb(config::HOVER_COLOR_R, config::HOVER_COLOR_G, config::HOVER_COLOR_B);
    let normal_bg = egui::Color32::from_rgba_premultiplied(255, 255, 255, 15);
    let bg = egui::Color32::from_rgb(config::BG_COLOR_R, config::BG_COLOR_G, config::BG_COLOR_B);

    // ── Tab-style header ────────────────────────────────────────────────────
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        // Tab label with rounded background
        let tab_text = egui::RichText::new("Hierarchy").size(11.0).strong().color(fg);
        let galley = ui.painter().layout_no_wrap(tab_text.text().to_string(), egui::FontId::proportional(11.0), fg);
        let text_size = galley.size();
        let padding = egui::vec2(10.0, 6.0);
        let tab_size = text_size + padding * 2.0;
        let tab_rect = egui::Rect::from_min_size(ui.cursor().min, tab_size);
        ui.painter().rect_filled(tab_rect, egui::CornerRadius::same(10), normal_bg);
        ui.painter().galley(tab_rect.left_top() + padding, galley, fg);
        ui.advance_cursor_after_rect(tab_rect);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Styled "+" button: 20x20, cream bg, black "+"
            let btn_size = egui::vec2(20.0, 20.0);
            let (btn_rect, btn_response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
            let btn_bg = if btn_response.hovered() {
                hover_bg
            } else {
                bg
            };
            ui.painter().rect_filled(btn_rect, egui::CornerRadius::same(6), btn_bg);
            ui.painter().text(
                btn_rect.center(),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::proportional(14.0),
                fg,
            );

            egui::Popup::from_toggle_button_response(&btn_response)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    for shape in [ShapeType::Rectangle, ShapeType::Circle, ShapeType::Triangle] {
                        if ui.button(shape.name()).clicked() {
                            scene.add_object(shape);
                        }
                    }
                });
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
        let btn = egui::Button::new(text).fill(item_bg).corner_radius(egui::CornerRadius::same(6));

        if ui.add_sized([ui.available_width(), 22.0], btn).clicked() {
            clicked_id = Some(obj.id);
        }
    }

    if let Some(id) = clicked_id {
        scene.select(Some(id));
    }
}
