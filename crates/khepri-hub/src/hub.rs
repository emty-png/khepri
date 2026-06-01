use eframe::egui;
use khepri_core::config;

use crate::title_bar;
use crate::window_resize;

/// What the Hub returns when it closes.
pub enum HubResult {
    Quit,
    OpenProject(std::path::PathBuf),
}

/// Shared result type for hub-to-main communication.
pub type SharedHubResult = std::sync::Arc<std::sync::Mutex<Option<HubResult>>>;

pub struct HubApp {
    show_popup: bool,
    popup_just_opened: bool,
    project_name: String,
    git_init: bool,
    chosen_folder: Option<std::path::PathBuf>,
    result: SharedHubResult,
    // Context menu (dots button)
    ctx_menu_project: Option<std::path::PathBuf>,
    ctx_menu_pos: egui::Pos2,
    ctx_menu_just_opened: bool,
    // Rename popup
    rename_project: Option<std::path::PathBuf>,
    rename_text: String,
    rename_just_opened: bool,
}

impl HubApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, result: SharedHubResult) -> Self {
        Self {
            show_popup: false,
            popup_just_opened: false,
            project_name: String::new(),
            git_init: false,
            chosen_folder: None,
            result,
            ctx_menu_project: None,
            ctx_menu_pos: egui::Pos2::ZERO,
            ctx_menu_just_opened: false,
            rename_project: None,
            rename_text: String::new(),
            rename_just_opened: false,
        }
    }

    fn render_popup(&mut self, ui: &mut egui::Ui) {
        if !self.show_popup {
            return;
        }

        // ── Colors (cream bg popup, black interactive elements) ──────────────
        let black = egui::Color32::from_rgb(0x00, 0x00, 0x00);
        let cream = egui::Color32::from_rgb(0xFF, 0xFD, 0xDD);
        let muted = egui::Color32::from_rgb(0x77, 0x77, 0x77);
        let popup_radius = egui::CornerRadius::same(10);
        let btn_r = egui::CornerRadius::same(6);

        // ── Popup rect ───────────────────────────────────────────────────────
        let screen = ui.ctx().content_rect();
        let popup_w = 300.0;
        let popup_h = 310.0;
        let popup_rect =
            egui::Rect::from_center_size(screen.center(), egui::vec2(popup_w, popup_h));

        egui::Area::new(egui::Id::new("new_project_popup"))
            .fixed_pos(popup_rect.min)
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                // Backdrop: paint + interactive absorb so background UI is blocked
                ui.painter().rect_filled(
                    screen,
                    0,
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 160),
                );
                // Invisible full-screen rect that absorbs all clicks behind the popup
                let _absorb = ui.interact(
                    screen,
                    egui::Id::new("popup_backdrop"),
                    egui::Sense::click(),
                );

                // Cream card background
                ui.painter().rect_filled(popup_rect, popup_radius, cream);

                // Black border (Middle so it stays inside the Area clip rect)
                ui.painter().rect_stroke(
                    popup_rect,
                    popup_radius,
                    egui::Stroke::new(2.0, black),
                    egui::StrokeKind::Middle,
                );

                // Clip to popup bounds (content inside, border painted above)
                ui.set_clip_rect(popup_rect);
                let inner = popup_rect.shrink2(egui::vec2(20.0, 20.0));
                let orig = ui.visuals().clone();

                // ── Widget overrides: black bg, cream text ────────────────────
                let stroke = egui::Stroke::new(1.0, cream);
                ui.visuals_mut().widgets.noninteractive.bg_fill = black;
                ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
                ui.visuals_mut().widgets.noninteractive.fg_stroke = stroke;
                ui.visuals_mut().widgets.noninteractive.corner_radius = btn_r;
                ui.visuals_mut().widgets.inactive.bg_fill = black;
                ui.visuals_mut().widgets.inactive.weak_bg_fill = black;
                ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                ui.visuals_mut().widgets.inactive.fg_stroke = stroke;
                ui.visuals_mut().widgets.inactive.corner_radius = btn_r;
                ui.visuals_mut().widgets.hovered.bg_fill =
                    egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);
                ui.visuals_mut().widgets.hovered.weak_bg_fill =
                    egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);
                ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::new(1.5, cream);
                ui.visuals_mut().widgets.hovered.fg_stroke = stroke;
                ui.visuals_mut().widgets.hovered.corner_radius = btn_r;
                ui.visuals_mut().widgets.active.bg_fill = egui::Color32::from_rgb(0x33, 0x33, 0x33);
                ui.visuals_mut().widgets.active.weak_bg_fill =
                    egui::Color32::from_rgb(0x33, 0x33, 0x33);
                ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::new(2.0, cream);
                ui.visuals_mut().widgets.active.fg_stroke = stroke;
                ui.visuals_mut().widgets.active.corner_radius = btn_r;
                ui.visuals_mut().selection.bg_fill = cream;
                ui.visuals_mut().selection.stroke = egui::Stroke::new(1.0, black);

                ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                    // ── Title ─────────────────────────────────────────────────
                    ui.label(
                        egui::RichText::new("New Project")
                            .size(24.0)
                            .strong()
                            .color(black),
                    );
                    ui.add_space(20.0);

                    // ── Name ──────────────────────────────────────────────────
                    ui.label(egui::RichText::new("Name").size(10.0).color(muted));
                    ui.add_space(4.0);
                    ui.add_sized(
                        [ui.available_width(), 28.0],
                        egui::TextEdit::singleline(&mut self.project_name)
                            .font(egui::FontId::proportional(12.0))
                            .text_color(cream)
                            .background_color(black)
                            .margin(egui::vec2(8.0, 6.0)),
                    );
                    ui.add_space(12.0);

                    // ── Location ──────────────────────────────────────────────
                    ui.label(egui::RichText::new("Location").size(10.0).color(muted));
                    ui.add_space(4.0);
                    let browse_label = match &self.chosen_folder {
                        Some(p) => p.display().to_string(),
                        None => "Browse...".to_string(),
                    };
                    let browse_btn = egui::Button::new(
                        egui::RichText::new(&browse_label).size(11.0).color(cream),
                    )
                    .fill(black)
                    .corner_radius(btn_r);

                    if ui
                        .add_sized([ui.available_width(), 28.0], browse_btn)
                        .clicked()
                        && let Some(folder) = rfd::FileDialog::new()
                            .set_title("Select Project Folder")
                            .pick_folder()
                    {
                        self.chosen_folder = Some(folder);
                    }
                    ui.add_space(12.0);

                    // ── Git checkbox ──────────────────────────────────────────
                    ui.checkbox(
                        &mut self.git_init,
                        egui::RichText::new("Initialize Git repository")
                            .size(11.0)
                            .color(black),
                    );
                    ui.add_space(20.0);

                    // ── Action buttons ────────────────────────────────────────
                    let can_create = !self.project_name.is_empty() && self.chosen_folder.is_some();

                    ui.horizontal(|ui| {
                        let item_gap = ui.spacing().item_spacing.x;
                        let cancel_w = 82.0;
                        // Use popup_w - 40.0 (the known inner width) for consistent sizing
                        let create_w = (popup_w - 40.0) - cancel_w - item_gap;

                        // Dim the Create button when inputs are incomplete
                        let dim = if can_create { 255u8 } else { 70u8 };
                        let c_bg = egui::Color32::from_rgba_unmultiplied(0x00, 0x00, 0x00, dim);
                        let c_fg = egui::Color32::from_rgba_unmultiplied(0xFF, 0xFD, 0xDD, dim);

                        let create_btn = egui::Button::new(
                            egui::RichText::new("Create")
                                .size(11.0)
                                .strong()
                                .color(c_fg),
                        )
                        .fill(c_bg)
                        .corner_radius(btn_r);

                        let create_resp = ui.add_sized([create_w, 30.0], create_btn);
                        // Hover overlay: cream bg + black text
                        if create_resp.hovered() && can_create {
                            let r = create_resp.rect;
                            ui.painter().rect_filled(r, btn_r, cream);
                            ui.painter().text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                "Create",
                                egui::FontId::proportional(11.0),
                                black,
                            );
                        }
                        if create_resp.clicked() && can_create {
                            let root = self
                                .chosen_folder
                                .as_ref()
                                .unwrap()
                                .join(&self.project_name);
                            match khepri_storage::project::create_project(
                                &root,
                                &self.project_name,
                                self.git_init,
                            ) {
                                Ok(_) => {
                                    khepri_storage::recent::add_recent_project(&root).ok();
                                    *self.result.lock().unwrap() =
                                        Some(HubResult::OpenProject(root));
                                    self.show_popup = false;
                                    ctx_close(ui);
                                }
                                Err(e) => {
                                    eprintln!("Failed to create project: {}", e);
                                }
                            }
                        }

                        let cancel_btn = egui::Button::new(
                            egui::RichText::new("Cancel").size(11.0).color(cream),
                        )
                        .fill(black)
                        .corner_radius(btn_r);

                        let cancel_resp = ui.add_sized([cancel_w, 30.0], cancel_btn);
                        // Hover overlay: cream bg + black text
                        if cancel_resp.hovered() {
                            let r = cancel_resp.rect;
                            ui.painter().rect_filled(r, btn_r, cream);
                            ui.painter().text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                "Cancel",
                                egui::FontId::proportional(11.0),
                                black,
                            );
                        }
                        if cancel_resp.clicked() {
                            self.show_popup = false;
                        }
                    });
                });

                // Close when backdrop absorbs a click (and not on the frame
                // we just opened the popup).
                if !self.popup_just_opened && _absorb.clicked() {
                    self.show_popup = false;
                }
                self.popup_just_opened = false;

                *ui.visuals_mut() = orig;
            });
    }

    fn render_context_menu(&mut self, ui: &mut egui::Ui) {
        let project = match &self.ctx_menu_project {
            Some(p) => p.clone(),
            None => return,
        };

        let black = egui::Color32::from_rgb(0x00, 0x00, 0x00);
        let cream = egui::Color32::from_rgb(0xFF, 0xFD, 0xDD);
        let btn_r = egui::CornerRadius::same(6);
        let popup_radius = egui::CornerRadius::same(10);

        let menu_w = 90.0;
        let menu_h = 68.0;
        let menu_pos = egui::pos2(
            self.ctx_menu_pos.x - menu_w / 2.0,
            self.ctx_menu_pos.y + 8.0,
        );
        let menu_rect = egui::Rect::from_min_size(menu_pos, egui::vec2(menu_w, menu_h));

        egui::Area::new(egui::Id::new("ctx_menu"))
            .fixed_pos(menu_pos)
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                ui.painter().rect_filled(menu_rect, popup_radius, cream);
                ui.painter().rect_stroke(
                    menu_rect,
                    popup_radius,
                    egui::Stroke::new(2.0, black),
                    egui::StrokeKind::Middle,
                );
                ui.set_clip_rect(menu_rect);
                let inner = menu_rect.shrink2(egui::vec2(8.0, 8.0));

                let mut close_menu = false;
                let mut do_rename = false;
                let mut do_delete = false;

                ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                    let rename_btn =
                        egui::Button::new(egui::RichText::new("Rename").size(11.0).color(black))
                            .fill(cream)
                            .corner_radius(btn_r);
                    let rename_resp = ui.add_sized([ui.available_width(), 24.0], rename_btn);
                    if rename_resp.hovered() {
                        let r = rename_resp.rect;
                        ui.painter().rect_filled(r, btn_r, black);
                        ui.painter().text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            "Rename",
                            egui::FontId::proportional(11.0),
                            cream,
                        );
                    }
                    if rename_resp.clicked() {
                        do_rename = true;
                        close_menu = true;
                    }

                    ui.add_space(2.0);

                    let delete_btn =
                        egui::Button::new(egui::RichText::new("Delete").size(11.0).color(black))
                            .fill(cream)
                            .corner_radius(btn_r);
                    let delete_resp = ui.add_sized([ui.available_width(), 24.0], delete_btn);
                    if delete_resp.hovered() {
                        let r = delete_resp.rect;
                        ui.painter().rect_filled(r, btn_r, black);
                        ui.painter().text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            "Delete",
                            egui::FontId::proportional(11.0),
                            cream,
                        );
                    }
                    if delete_resp.clicked() {
                        do_delete = true;
                        close_menu = true;
                    }
                });

                if do_rename {
                    self.rename_project = Some(project.clone());
                    self.rename_text = project
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    self.rename_just_opened = true;
                }

                if do_delete {
                    // Remove from recent projects
                    khepri_storage::recent::remove_recent_project(&project).ok();
                    // Delete folder from disk
                    let _ = std::fs::remove_dir_all(&project);
                    close_menu = true;
                }

                // Close on outside click (skip the frame we opened)
                if self.ctx_menu_just_opened {
                    self.ctx_menu_just_opened = false;
                } else if ui.input(|i| i.pointer.primary_clicked()) {
                    close_menu = true;
                }

                if close_menu {
                    self.ctx_menu_project = None;
                }
            });
    }

    fn render_rename_popup(&mut self, ui: &mut egui::Ui) {
        let project = match &self.rename_project {
            Some(p) => p.clone(),
            None => return,
        };

        let black = egui::Color32::from_rgb(0x00, 0x00, 0x00);
        let cream = egui::Color32::from_rgb(0xFF, 0xFD, 0xDD);
        let muted = egui::Color32::from_rgb(0x77, 0x77, 0x77);
        let btn_r = egui::CornerRadius::same(6);
        let popup_radius = egui::CornerRadius::same(10);

        let screen = ui.ctx().content_rect();
        let popup_w = 320.0;
        let popup_h = 180.0;
        let popup_rect =
            egui::Rect::from_center_size(screen.center(), egui::vec2(popup_w, popup_h));

        egui::Area::new(egui::Id::new("rename_popup"))
            .fixed_pos(popup_rect.min)
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ui.ctx(), |ui| {
                // Backdrop
                ui.painter().rect_filled(
                    screen,
                    0,
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 120),
                );
                let _absorb = ui.interact(
                    screen,
                    egui::Id::new("rename_backdrop"),
                    egui::Sense::click(),
                );

                // Cream card
                ui.painter().rect_filled(popup_rect, popup_radius, cream);
                ui.set_clip_rect(popup_rect);
                let inner = popup_rect.shrink2(egui::vec2(20.0, 20.0));
                let orig = ui.visuals().clone();

                // Widget overrides
                let stroke = egui::Stroke::new(1.0, cream);
                ui.visuals_mut().widgets.noninteractive.bg_fill = black;
                ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
                ui.visuals_mut().widgets.noninteractive.fg_stroke = stroke;
                ui.visuals_mut().widgets.noninteractive.corner_radius = btn_r;
                ui.visuals_mut().widgets.inactive.bg_fill = black;
                ui.visuals_mut().widgets.inactive.weak_bg_fill = black;
                ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                ui.visuals_mut().widgets.inactive.fg_stroke = stroke;
                ui.visuals_mut().widgets.inactive.corner_radius = btn_r;
                ui.visuals_mut().widgets.hovered.bg_fill =
                    egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);
                ui.visuals_mut().widgets.hovered.weak_bg_fill =
                    egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);
                ui.visuals_mut().widgets.hovered.bg_stroke = egui::Stroke::new(1.5, cream);
                ui.visuals_mut().widgets.hovered.fg_stroke = stroke;
                ui.visuals_mut().widgets.hovered.corner_radius = btn_r;
                ui.visuals_mut().widgets.active.bg_fill = egui::Color32::from_rgb(0x33, 0x33, 0x33);
                ui.visuals_mut().widgets.active.weak_bg_fill =
                    egui::Color32::from_rgb(0x33, 0x33, 0x33);
                ui.visuals_mut().widgets.active.bg_stroke = egui::Stroke::new(2.0, cream);
                ui.visuals_mut().widgets.active.fg_stroke = stroke;
                ui.visuals_mut().widgets.active.corner_radius = btn_r;
                ui.visuals_mut().selection.bg_fill = cream;
                ui.visuals_mut().selection.stroke = egui::Stroke::new(1.0, black);

                ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                    ui.label(
                        egui::RichText::new("Rename Project")
                            .size(20.0)
                            .strong()
                            .color(black),
                    );
                    ui.add_space(16.0);

                    ui.label(egui::RichText::new("New name").size(10.0).color(muted));
                    ui.add_space(4.0);
                    ui.add_sized(
                        [ui.available_width(), 28.0],
                        egui::TextEdit::singleline(&mut self.rename_text)
                            .font(egui::FontId::proportional(12.0))
                            .text_color(cream)
                            .background_color(black)
                            .margin(egui::vec2(8.0, 6.0)),
                    );
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        let item_gap = ui.spacing().item_spacing.x;
                        let cancel_w = 82.0;
                        let rename_w = (popup_w - 40.0) - cancel_w - item_gap;

                        let can_rename = !self.rename_text.is_empty();
                        let dim = if can_rename { 255u8 } else { 70u8 };
                        let r_bg = egui::Color32::from_rgba_unmultiplied(0x00, 0x00, 0x00, dim);
                        let r_fg = egui::Color32::from_rgba_unmultiplied(0xFF, 0xFD, 0xDD, dim);

                        let rename_btn = egui::Button::new(
                            egui::RichText::new("Rename")
                                .size(11.0)
                                .strong()
                                .color(r_fg),
                        )
                        .fill(r_bg)
                        .corner_radius(btn_r);

                        let rename_resp = ui.add_sized([rename_w, 30.0], rename_btn);
                        // Hover overlay: cream bg + black text
                        if rename_resp.hovered() && can_rename {
                            let r = rename_resp.rect;
                            ui.painter().rect_filled(r, btn_r, cream);
                            ui.painter().text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                "Rename",
                                egui::FontId::proportional(11.0),
                                black,
                            );
                        }
                        if rename_resp.clicked() && can_rename {
                            if let Some(parent) = project.parent() {
                                let new_path = parent.join(&self.rename_text);
                                if std::fs::rename(&project, &new_path).is_ok() {
                                    // Update recent projects
                                    khepri_storage::recent::remove_recent_project(&project).ok();
                                    khepri_storage::recent::add_recent_project(&new_path).ok();
                                }
                            }
                            self.rename_project = None;
                        }

                        let cancel_btn = egui::Button::new(
                            egui::RichText::new("Cancel").size(11.0).color(cream),
                        )
                        .fill(black)
                        .corner_radius(btn_r);

                        let cancel_resp = ui.add_sized([cancel_w, 30.0], cancel_btn);
                        // Hover overlay: cream bg + black text
                        if cancel_resp.hovered() {
                            let r = cancel_resp.rect;
                            ui.painter().rect_filled(r, btn_r, cream);
                            ui.painter().text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                "Cancel",
                                egui::FontId::proportional(11.0),
                                black,
                            );
                        }
                        if cancel_resp.clicked() {
                            self.rename_project = None;
                        }
                    });
                });

                // Close on backdrop click (skip the frame we opened)
                if self.rename_just_opened {
                    self.rename_just_opened = false;
                } else if _absorb.clicked() {
                    self.rename_project = None;
                }

                *ui.visuals_mut() = orig;
            });
    }
}

fn ctx_close(ui: &mut egui::Ui) {
    ui.send_viewport_cmd(egui::ViewportCommand::Close);
}

impl eframe::App for HubApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Title bar
        let plus_clicked = title_bar::show(ui, self.show_popup);
        if plus_clicked {
            self.show_popup = true;
            self.popup_just_opened = true;
            self.project_name.clear();
            self.chosen_folder = None;
            self.git_init = false;
        }

        // ── Theme colors ────────────────────────────────────────────────────
        let bg =
            egui::Color32::from_rgb(config::BG_COLOR_R, config::BG_COLOR_G, config::BG_COLOR_B);
        let fg =
            egui::Color32::from_rgb(config::FG_COLOR_R, config::FG_COLOR_G, config::FG_COLOR_B);
        let card_dark = egui::Color32::from_rgb(0x00, 0x00, 0x00);
        let hover_bg = egui::Color32::from_rgb(
            config::HOVER_COLOR_R,
            config::HOVER_COLOR_G,
            config::HOVER_COLOR_B,
        );
        let card_dark_radius = egui::CornerRadius::same(15);
        let btn_radius = egui::CornerRadius::same(6);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg).inner_margin(egui::Margin {
                left: 14,
                right: 14,
                top: 0,
                bottom: 0,
            }))
            .show_inside(ui, |ui| {
                let projects = khepri_storage::recent::get_recent_projects();
                if projects.is_empty() {
                    // ── Empty state ──────────────────────────────────────────
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 3.0);
                        ui.label(
                            egui::RichText::new("Khepri Hub")
                                .size(32.0)
                                .strong()
                                .color(fg),
                        );
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new("No projects yet — click + to create one")
                                .size(11.0)
                                .color(fg),
                        );
                    });
                } else {
                    // ── Project cards (2-column grid) ───────────────────────
                    ui.add_space(16.0);
                    let full_w = ui.available_width();
                    let gap = 8.0;
                    let card_w = (full_w - gap) / 2.0;

                    let mut open_path = None;

                    for row in projects.chunks(2) {
                        ui.columns(row.len(), |cols| {
                            for (i, project_path) in row.iter().enumerate() {
                                let name = project_path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "Unknown".to_string());

                                let mut clicked_open = false;
                                let mut clicked_more = false;
                                let mut dots_btn_rect = egui::Rect::NOTHING;

                                let card = egui::Frame::new()
                                    .fill(card_dark)
                                    .corner_radius(card_dark_radius)
                                    .inner_margin(egui::vec2(18.0, 16.0));

                                card.show(&mut cols[i], |ui| {
                                    ui.set_min_size(egui::vec2(card_w - 36.0, 90.0));

                                    // ── Name ─────────────────────────────
                                    ui.label(
                                        egui::RichText::new(&name)
                                            .size(28.0)
                                            .strong()
                                            .color(egui::Color32::WHITE),
                                    );
                                    ui.add_space(16.0);

                                    // ── Button row ──────────────────────
                                    ui.horizontal(|ui| {
                                        let item_gap = ui.spacing().item_spacing.x;
                                        let dots_w = 40.0;
                                        let open_w = ui.available_width() - dots_w - item_gap;

                                        let open_btn = egui::Button::new(
                                            egui::RichText::new("Open in Editor")
                                                .size(11.0)
                                                .strong()
                                                .color(fg),
                                        )
                                        .fill(bg)
                                        .corner_radius(btn_radius);

                                        if ui.add_sized([open_w, 30.0], open_btn).clicked() {
                                            clicked_open = true;
                                        }

                                        let (rect, resp) = ui.allocate_exact_size(
                                            egui::vec2(dots_w, 30.0),
                                            egui::Sense::click(),
                                        );
                                        dots_btn_rect = rect;
                                        let dots_bg = if resp.hovered() { hover_bg } else { bg };
                                        ui.painter().rect_filled(rect, btn_radius, dots_bg);
                                        ui.painter().text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "•••",
                                            egui::FontId::proportional(11.0),
                                            fg,
                                        );

                                        if resp.clicked() {
                                            clicked_more = true;
                                        }
                                    });
                                });

                                if clicked_open {
                                    open_path = Some(project_path.clone());
                                }
                                if clicked_more {
                                    self.ctx_menu_project = Some(project_path.clone());
                                    self.ctx_menu_pos = dots_btn_rect.center();
                                    self.ctx_menu_just_opened = true;
                                }
                            }
                        });
                        ui.add_space(gap);
                    }

                    if let Some(path) = open_path {
                        *self.result.lock().unwrap() = Some(HubResult::OpenProject(path));
                        ctx_close(ui);
                    }

                    // ── Context menu popup (rendered after cards so it's on top)
                    self.render_context_menu(ui);

                    // ── Rename popup (rendered after context menu so it's on top)
                    self.render_rename_popup(ui);
                }
            });

        // Popup (rendered on top)
        self.render_popup(ui);

        // Resize handles
        window_resize::custom_window_resize(ui);
    }
}
