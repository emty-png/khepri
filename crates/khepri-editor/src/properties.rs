use eframe::egui;
use khepri_core::config;
use khepri_core::scene::Scene;

pub fn show_properties(ui: &mut egui::Ui, scene: &mut Scene) {
    let fg = egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
    let normal_bg = egui::Color32::from_rgba_premultiplied(255, 255, 255, 15);
    let input_bg = egui::Color32::from_rgba_premultiplied(0, 0, 0, 30);
    let input_fg = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

    // ── Tab-style header ────────────────────────────────────────────────────
    ui.add_space(8.0);
    let tab_text = egui::RichText::new("Properties").size(11.0).strong().color(fg);
    let galley = ui.painter().layout_no_wrap(tab_text.text().to_string(), egui::FontId::proportional(11.0), fg);
    let text_size = galley.size();
    let padding = egui::vec2(10.0, 6.0);
    let tab_size = text_size + padding * 2.0;
    let tab_rect = egui::Rect::from_min_size(ui.cursor().min, tab_size);
    ui.painter().rect_filled(tab_rect, egui::CornerRadius::same(10), normal_bg);
    ui.painter().galley(tab_rect.left_top() + padding, galley, fg);
    ui.advance_cursor_after_rect(tab_rect);
    let _ = ui.available_width();

    ui.add_space(4.0);
    ui.separator();

    let has_selection = scene.selected_id.is_some();

    if !has_selection {
        ui.add_space(8.0);
        ui.label(egui::RichText::new("No object selected").size(11.0).color(fg));
        return;
    }

    if let Some(obj) = scene.get_selected_mut() {
        ui.add_space(8.0);

        // Override text style to small for all DragValue inputs
        let prev_override = ui.style().override_text_style;
        let prev_slider = ui.style().spacing.slider_width;
        ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
        ui.style_mut().spacing.slider_width = 80.0;

        // Type (read-only)
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Type").size(11.0).color(fg));
            let mut type_text = obj.shape.name().to_string();
            ui.add_sized(
                [ui.available_width(), 18.0],
                egui::TextEdit::singleline(&mut type_text)
                    .font(egui::FontId::proportional(11.0))
                    .text_color(input_fg)
                    .background_color(input_bg)
                    .margin(egui::vec2(4.0, 2.0)),
            );
        });

        // Name (read-only)
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Name").size(11.0).color(fg));
            let mut name_text = obj.name.clone();
            ui.add_sized(
                [ui.available_width(), 18.0],
                egui::TextEdit::singleline(&mut name_text)
                    .font(egui::FontId::proportional(11.0))
                    .text_color(input_fg)
                    .background_color(input_bg)
                    .margin(egui::vec2(4.0, 2.0)),
            );
        });

        ui.add_space(12.0);

        // Position
        ui.label(egui::RichText::new("Position").size(11.0).strong().color(fg));
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("X").size(11.0).color(fg));
            ui.add(egui::DragValue::new(&mut obj.x).speed(1.0));
            ui.label(egui::RichText::new("Y").size(11.0).color(fg));
            ui.add(egui::DragValue::new(&mut obj.y).speed(1.0));
        });

        ui.add_space(12.0);

        // Size
        ui.label(egui::RichText::new("Size").size(11.0).strong().color(fg));
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("W").size(11.0).color(fg));
            ui.add(egui::DragValue::new(&mut obj.width).speed(1.0).range(1.0..=5000.0));
            ui.label(egui::RichText::new("H").size(11.0).color(fg));
            ui.add(egui::DragValue::new(&mut obj.height).speed(1.0).range(1.0..=5000.0));
        });

        ui.add_space(12.0);

        // Rotation
        ui.label(egui::RichText::new("Rotation").size(11.0).strong().color(fg));
        ui.add_space(2.0);
        ui.add(egui::DragValue::new(&mut obj.rotation).speed(1.0).range(-360.0..=360.0).suffix(" deg"));

        // Restore style
        ui.style_mut().override_text_style = prev_override;
        ui.style_mut().spacing.slider_width = prev_slider;
    }

    // Borrow is released here, so we can call remove_selected
    ui.add_space(16.0);
    if ui.button(egui::RichText::new("Delete Object").size(11.0)).clicked() {
        scene.remove_selected();
    }
}
