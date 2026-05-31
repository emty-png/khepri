use eframe::egui;
use khepri_core::config;
use khepri_core::scene::Scene;
use khepri_editor::panels::{BentoLayout, draw_bento};
use khepri_editor::title_bar;
use khepri_editor::window_resize;

struct KhepriApp {
    bento: BentoLayout,
    scene: Scene,
}

impl KhepriApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            bento: BentoLayout::default(),
            scene: Scene::new(),
        }
    }
}

impl eframe::App for KhepriApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        title_bar::show(ui);
        let bg =
            egui::Color32::from_rgb(config::BG_COLOR_R, config::BG_COLOR_G, config::BG_COLOR_B);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg))
            .show_inside(ui, |ui| {
                draw_bento(ui, &mut self.bento, &mut self.scene);
            });
        window_resize::custom_window_resize(ui);
    }
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Khepri");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([400.0, 300.0])
            .with_decorations(false)
            .with_resizable(true),
        renderer: eframe::Renderer::Glow,
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native(
        "Khepri",
        native_options,
        Box::new(|cc| Ok(Box::new(KhepriApp::new(cc)))),
    )
}
