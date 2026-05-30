use eframe::egui;
use khepri_core::config;
use khepri_core::scene::{Scene, ShapeType};

pub fn show_hierarchy(ui: &mut egui::Ui, scene: &mut Scene) {
    let fg = egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
    let hover_bg = egui::Color32::from_rgb(config::HOVER_COLOR_R, config::HOVER_COLOR_G, config::HOVER_COLOR_B);

    // Header row: "Hierarchy" on left, "+" on right
    ui.horizontal(|ui| {
        ui.heading("Hierarchy");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let add_button = ui.button("+");
            egui::Popup::from_toggle_button_response(&add_button)
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

    ui.separator();

    // Object list
    let mut clicked_id = None;
    for obj in &scene.objects {
        let is_selected = scene.selected_id == Some(obj.id);

        let normal_bg = egui::Color32::from_rgba_premultiplied(255, 255, 255, 15);
        let bg = if is_selected { hover_bg } else { normal_bg };
        let text = egui::RichText::new(&obj.name).color(fg);
        let btn = egui::Button::new(text).fill(bg);

        if ui.add_sized([ui.available_width(), 20.0], btn).clicked() {
            clicked_id = Some(obj.id);
        }
    }

    if let Some(id) = clicked_id {
        scene.select(Some(id));
    }
}
