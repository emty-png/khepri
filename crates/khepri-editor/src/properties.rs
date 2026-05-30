use eframe::egui;
use khepri_core::config;
use khepri_core::scene::Scene;

pub fn show_properties(ui: &mut egui::Ui, scene: &mut Scene) {
    let fg = egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);

    ui.heading("Properties");
    ui.separator();

    let has_selection = scene.selected_id.is_some();

    if !has_selection {
        ui.label(egui::RichText::new("No object selected").color(fg));
        return;
    }

    // Scope the mutable borrow so we can call remove_selected later
    if let Some(obj) = scene.get_selected_mut() {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Type:").color(fg));
            ui.label(egui::RichText::new(obj.shape.name()).strong().color(fg));
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Name:").color(fg));
            ui.label(egui::RichText::new(&obj.name).color(fg));
        });

        ui.add_space(8.0);

        ui.label(egui::RichText::new("Position").strong().color(fg));
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("X:").color(fg));
            ui.add(egui::DragValue::new(&mut obj.x).speed(1.0));
            ui.label(egui::RichText::new("Y:").color(fg));
            ui.add(egui::DragValue::new(&mut obj.y).speed(1.0));
        });

        ui.add_space(8.0);

        ui.label(egui::RichText::new("Size").strong().color(fg));
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("W:").color(fg));
            ui.add(egui::DragValue::new(&mut obj.width).speed(1.0).range(1.0..=5000.0));
            ui.label(egui::RichText::new("H:").color(fg));
            ui.add(egui::DragValue::new(&mut obj.height).speed(1.0).range(1.0..=5000.0));
        });

        ui.add_space(8.0);

        ui.label(egui::RichText::new("Rotation").strong().color(fg));
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut obj.rotation).speed(1.0).range(-360.0..=360.0).suffix(" deg"));
        });
    }

    // Borrow is released here, so we can call remove_selected
    ui.add_space(16.0);
    if ui.button("Delete Object").clicked() {
        scene.remove_selected();
    }
}
