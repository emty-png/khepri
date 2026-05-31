use eframe::egui;
use khepri_core::config;
use khepri_core::scene::Scene;

pub fn show_properties(ui: &mut egui::Ui, scene: &mut Scene) {
    let fg = egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
    let normal_bg = egui::Color32::from_rgba_premultiplied(255, 255, 255, 15);

    // ── Tab-style header ────────────────────────────────────────────────────
    ui.add_space(8.0);
    let tab_text = egui::RichText::new("Properties")
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

    ui.add_space(4.0);
    ui.separator();

    let has_selection = scene.selected_id.is_some();

    if !has_selection {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("No object selected")
                .size(11.0)
                .color(fg),
        );
        return;
    }

    if let Some(obj) = scene.get_selected_mut() {
        ui.add_space(8.0);

        // ── Color palette ────────────────────────────────────────────────────
        let card_bg = egui::Color32::from_rgb(0xFF, 0xFD, 0xDD);
        let card_stroke = egui::Stroke::new(1.0, fg);
        let card_radius = egui::CornerRadius::same(6);
        let card_margin = egui::vec2(8.0, 8.0);
        let input_bg = fg;
        let input_text = card_bg;

        // ── Widget visuals override ──────────────────────────────────────────
        let orig_visuals = ui.visuals().clone();
        ui.visuals_mut().widgets.noninteractive.bg_fill = input_bg;
        ui.visuals_mut().widgets.noninteractive.bg_stroke = card_stroke;
        ui.visuals_mut().widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, input_text);
        ui.visuals_mut().widgets.noninteractive.corner_radius = card_radius;
        ui.visuals_mut().widgets.inactive.bg_fill = input_bg;
        ui.visuals_mut().widgets.inactive.bg_stroke = card_stroke;
        ui.visuals_mut().widgets.inactive.fg_stroke = egui::Stroke::new(1.0, input_text);
        ui.visuals_mut().widgets.inactive.corner_radius = card_radius;
        ui.visuals_mut().widgets.hovered.bg_fill = input_bg;
        ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::new(1.5, input_text);
        ui.visuals_mut().widgets.hovered.fg_stroke = egui::Stroke::new(1.0, input_text);
        ui.visuals_mut().widgets.hovered.corner_radius = card_radius;
        ui.visuals_mut().widgets.active.bg_fill = input_bg;
        ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::new(2.0, input_text);
        ui.visuals_mut().widgets.active.fg_stroke = egui::Stroke::new(1.0, input_text);
        ui.visuals_mut().widgets.active.corner_radius = card_radius;
        ui.visuals_mut().extreme_bg_color = input_bg;
        ui.visuals_mut().selection.bg_fill = input_bg;
        ui.visuals_mut().selection.stroke = egui::Stroke::new(1.0, input_text);

        // ── Identity card ─────────────────────────────────────────────────
        let card = egui::Frame::new()
            .fill(card_bg)
            .stroke(card_stroke)
            .corner_radius(card_radius)
            .inner_margin(card_margin);
        card.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Type").size(11.0).color(fg));
                let mut type_text = obj.shape.name().to_string();
                ui.add_sized(
                    [ui.available_width(), 18.0],
                    egui::TextEdit::singleline(&mut type_text)
                        .font(egui::FontId::proportional(11.0))
                        .text_color(input_text)
                        .background_color(input_bg)
                        .margin(egui::vec2(4.0, 2.0)),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Name").size(11.0).color(fg));
                ui.add_sized(
                    [ui.available_width(), 18.0],
                    egui::TextEdit::singleline(&mut obj.name)
                        .font(egui::FontId::proportional(11.0))
                        .text_color(input_text)
                        .background_color(input_bg)
                        .margin(egui::vec2(4.0, 2.0)),
                );
            });
        });

        ui.add_space(8.0);

        // ── Position card ─────────────────────────────────────────────────
        let card = egui::Frame::new()
            .fill(card_bg)
            .stroke(card_stroke)
            .corner_radius(card_radius)
            .inner_margin(card_margin);
        card.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new("Position")
                    .size(11.0)
                    .strong()
                    .color(fg),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("X").size(11.0).color(fg));
                ui.add(egui::DragValue::new(&mut obj.x).speed(1.0));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Y").size(11.0).color(fg));
                ui.add(egui::DragValue::new(&mut obj.y).speed(1.0));
            });
        });

        ui.add_space(8.0);

        // ── Size card ─────────────────────────────────────────────────────
        let card = egui::Frame::new()
            .fill(card_bg)
            .stroke(card_stroke)
            .corner_radius(card_radius)
            .inner_margin(card_margin);
        card.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("Size").size(11.0).strong().color(fg));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("W").size(11.0).color(fg));
                ui.add(
                    egui::DragValue::new(&mut obj.width)
                        .speed(1.0)
                        .range(1.0..=5000.0),
                );
                ui.add_space(8.0);
                ui.label(egui::RichText::new("H").size(11.0).color(fg));
                ui.add(
                    egui::DragValue::new(&mut obj.height)
                        .speed(1.0)
                        .range(1.0..=5000.0),
                );
            });
        });

        ui.add_space(8.0);

        // ── Rotation card ─────────────────────────────────────────────────
        let card = egui::Frame::new()
            .fill(card_bg)
            .stroke(card_stroke)
            .corner_radius(card_radius)
            .inner_margin(card_margin);
        card.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(
                egui::RichText::new("Rotation")
                    .size(11.0)
                    .strong()
                    .color(fg),
            );
            ui.add_space(4.0);
            ui.add(
                egui::DragValue::new(&mut obj.rotation)
                    .speed(1.0)
                    .range(-360.0..=360.0)
                    .suffix(" deg"),
            );
        });

        // Restore original visuals
        *ui.visuals_mut() = egui::Visuals::clone(&orig_visuals);
    }

    // Borrow released
    ui.add_space(16.0);
    let del_btn = egui::Button::new(egui::RichText::new("Delete Object").size(11.0).color(fg))
        .fill(egui::Color32::from_rgb(0xFF, 0xFD, 0xDD))
        .stroke(egui::Stroke::new(1.0, fg))
        .corner_radius(egui::CornerRadius::same(6));
    if ui
        .add_sized([ui.available_width(), 20.0], del_btn)
        .clicked()
    {
        scene.remove_selected();
    }
}
