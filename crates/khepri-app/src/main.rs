use eframe::egui;
use khepri_core::config;
use khepri_core::scene::Scene;
use khepri_editor::panels::{BentoLayout, draw_bento};
use khepri_editor::title_bar;
use khepri_editor::window_resize;
use khepri_hub::{HubResult, SharedHubResult};

// ── Editor App ──────────────────────────────────────────────────────────────

struct EditorApp {
    bento: BentoLayout,
    scene: Scene,
    project_path: std::path::PathBuf,
    dirty: bool,
    show_save_prompt: bool,
    wants_close: bool,
}

impl EditorApp {
    fn new(_cc: &eframe::CreationContext<'_>, project_path: std::path::PathBuf) -> Self {
        let scene_path = khepri_storage::scene_io::default_scene_path(&project_path);
        let scene = if scene_path.exists() {
            match khepri_storage::scene_io::load_scene(&scene_path) {
                Ok(s) => {
                    tracing::info!("Loaded scene from {}", scene_path.display());
                    s
                }
                Err(e) => {
                    tracing::warn!("Failed to load scene: {}", e);
                    Scene::new()
                }
            }
        } else {
            Scene::new()
        };

        Self {
            bento: BentoLayout::default(),
            scene,
            project_path,
            dirty: false,
            show_save_prompt: false,
            wants_close: false,
        }
    }

    fn save_scene(&mut self) {
        let scene_path = khepri_storage::scene_io::default_scene_path(&self.project_path);
        match khepri_storage::scene_io::save_scene(&scene_path, &self.scene) {
            Ok(()) => {
                self.dirty = false;
                tracing::info!("Saved scene");
            }
            Err(e) => {
                tracing::error!("Failed to save: {}", e);
            }
        }
    }

    fn snapshot(&self) -> SceneSnapshot {
        SceneSnapshot {
            object_count: self.scene.objects.len(),
            selected_id: self.scene.selected_id,
            objects: self.scene.objects.clone(),
        }
    }
}

struct SceneSnapshot {
    object_count: usize,
    selected_id: Option<u64>,
    objects: Vec<khepri_core::scene::SceneObject>,
}

fn render_save_prompt(ui: &mut egui::Ui) -> SavePromptAction {
    // ── Theme colors ────────────────────────────────────────────────────────
    let bg = egui::Color32::from_rgb(config::BG_COLOR_R, config::BG_COLOR_G, config::BG_COLOR_B);
    let fg = egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
    let card_bg = egui::Color32::from_rgb(0xFF, 0xFD, 0xDD);
    let card_stroke = egui::Stroke::new(1.0, fg);
    let card_radius = egui::CornerRadius::same(6);
    let card_margin = egui::vec2(16.0, 16.0);

    // ── Modal backdrop ──────────────────────────────────────────────────────
    let screen = ui.ctx().content_rect();
    ui.painter().rect_filled(
        screen,
        0,
        egui::Color32::from_rgba_premultiplied(0, 0, 0, 120),
    );

    // ── Centered card ───────────────────────────────────────────────────────
    let popup_w = 320.0;
    let popup_h = 160.0;
    let popup_rect = egui::Rect::from_center_size(screen.center(), egui::vec2(popup_w, popup_h));

    let mut action = SavePromptAction::None;

    egui::Area::new(egui::Id::new("save_prompt"))
        .fixed_pos(popup_rect.min)
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ui.ctx(), |ui| {
            ui.set_min_size(egui::vec2(popup_w, popup_h));

            // Background card
            ui.painter().rect_filled(popup_rect, card_radius, card_bg);
            ui.painter().rect_stroke(
                popup_rect,
                card_radius,
                card_stroke,
                egui::StrokeKind::Outside,
            );

            let inner = popup_rect.shrink2(card_margin);
            ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                // ── Header ───────────────────────────────────────────────────
                ui.label(
                    egui::RichText::new("Save changes?")
                        .size(14.0)
                        .strong()
                        .color(fg),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("You have unsaved changes.")
                        .size(11.0)
                        .color(fg),
                );
                ui.add_space(16.0);

                // ── Action buttons ───────────────────────────────────────────
                ui.horizontal(|ui| {
                    let save_btn = egui::Button::new(
                        egui::RichText::new("Save & Close")
                            .size(11.0)
                            .strong()
                            .color(fg),
                    )
                    .fill(card_bg)
                    .stroke(egui::Stroke::new(1.0, fg))
                    .corner_radius(card_radius);

                    if ui.add(save_btn).clicked() {
                        action = SavePromptAction::SaveAndClose;
                    }

                    let dont_save_btn =
                        egui::Button::new(egui::RichText::new("Don't Save").size(11.0).color(fg))
                            .fill(card_bg)
                            .stroke(egui::Stroke::new(1.0, fg))
                            .corner_radius(card_radius);

                    if ui.add(dont_save_btn).clicked() {
                        action = SavePromptAction::Close;
                    }

                    let cancel_btn =
                        egui::Button::new(egui::RichText::new("Cancel").size(11.0).color(fg))
                            .fill(bg)
                            .stroke(egui::Stroke::new(1.0, fg))
                            .corner_radius(card_radius);

                    if ui.add(cancel_btn).clicked() {
                        action = SavePromptAction::Cancel;
                    }
                });
            });
        });

    action
}

enum SavePromptAction {
    None,
    SaveAndClose,
    Close,
    Cancel,
}

impl eframe::App for EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Ctrl+S save
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_scene();
        }

        // Detect close request (from title bar X button)
        let close_requested = ui.input(|i| i.viewport().close_requested());
        if close_requested && self.dirty && !self.show_save_prompt {
            ui.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_save_prompt = true;
        }

        // Snapshot before frame for dirty detection
        let before = self.snapshot();

        // Title bar
        title_bar::show(ui);

        // Central panel with bento layout
        let bg =
            egui::Color32::from_rgb(config::BG_COLOR_R, config::BG_COLOR_G, config::BG_COLOR_B);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg))
            .show_inside(ui, |ui| {
                draw_bento(ui, &mut self.bento, &mut self.scene);
            });
        window_resize::custom_window_resize(ui);

        // Detect mutations
        if self.scene.objects.len() != before.object_count
            || self.scene.selected_id != before.selected_id
        {
            self.dirty = true;
        } else {
            for (a, b) in self.scene.objects.iter().zip(before.objects.iter()) {
                if a.x != b.x
                    || a.y != b.y
                    || a.width != b.width
                    || a.height != b.height
                    || a.rotation != b.rotation
                    || a.name != b.name
                {
                    self.dirty = true;
                    break;
                }
            }
        }

        // Save prompt overlay
        if self.show_save_prompt {
            match render_save_prompt(ui) {
                SavePromptAction::SaveAndClose => {
                    self.save_scene();
                    self.wants_close = true;
                    self.show_save_prompt = false;
                }
                SavePromptAction::Close => {
                    self.wants_close = true;
                    self.show_save_prompt = false;
                }
                SavePromptAction::Cancel => {
                    self.show_save_prompt = false;
                }
                SavePromptAction::None => {}
            }
        }

        // Actually close after save prompt decision
        if self.wants_close {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
            self.wants_close = false;
        }
    }
}

// ── Hub / Editor lifecycle ──────────────────────────────────────────────────

fn run_hub(result: SharedHubResult) {
    let shared = result;
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_decorations(false)
            .with_resizable(true),
        renderer: eframe::Renderer::Glow,
        persist_window: true,
        ..Default::default()
    };

    let shared_clone = shared.clone();
    let _ = eframe::run_native(
        "Khepri Hub",
        native_options,
        Box::new(|cc| Ok(Box::new(khepri_hub::HubApp::new(cc, shared_clone)))),
    );
}

fn run_editor(project_path: std::path::PathBuf) {
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

    let path = project_path;
    let _ = eframe::run_native(
        "Khepri",
        native_options,
        Box::new(|cc| Ok(Box::new(EditorApp::new(cc, path)))),
    );
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Khepri");

    loop {
        let result: SharedHubResult = std::sync::Arc::new(std::sync::Mutex::new(None));
        run_hub(result.clone());

        let hub_result = result.lock().unwrap().take().unwrap_or(HubResult::Quit);
        match hub_result {
            HubResult::Quit => break,
            HubResult::OpenProject(path) => {
                tracing::info!("Opening project: {}", path.display());
                run_editor(path);
            }
        }
    }

    Ok(())
}
