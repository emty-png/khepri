use eframe::egui;
use khepri_core::config;
use khepri_core::scene::{Scene, ShapeType};

const SELECTION_COLOR: egui::Color32 = egui::Color32::from_rgb(0x00, 0x78, 0xD4);
const GRID_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(255, 255, 255, 20);
const OBJECT_FILL: egui::Color32 = egui::Color32::from_rgb(0x50, 0x50, 0x50);
const OBJECT_STROKE: egui::Color32 = egui::Color32::from_rgb(0x80, 0x80, 0x80);
const VIEWPORT_BG: egui::Color32 = egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);

/// Camera stored in egui memory so it persists across frames.
#[derive(Clone, Copy)]
struct Camera {
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl Camera {
    fn id() -> egui::Id {
        egui::Id::new("viewport_camera")
    }

    fn load(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp(Self::id())).unwrap_or_default()
    }

    fn save(self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.insert_temp(Self::id(), self));
    }

    /// World position of the viewport center (screen center minus pan).
    fn world_origin(&self, viewport_center: egui::Pos2) -> egui::Pos2 {
        egui::pos2(
            viewport_center.x - self.pan_x * self.zoom,
            viewport_center.y - self.pan_y * self.zoom,
        )
    }

    /// Convert world coordinates to screen coordinates.
    fn world_to_screen(&self, world: egui::Pos2, viewport_center: egui::Pos2) -> egui::Pos2 {
        let origin = self.world_origin(viewport_center);
        egui::pos2(
            origin.x + world.x * self.zoom,
            origin.y + world.y * self.zoom,
        )
    }

    /// Convert screen coordinates to world coordinates.
    fn screen_to_world(&self, screen: egui::Pos2, viewport_center: egui::Pos2) -> egui::Pos2 {
        let origin = self.world_origin(viewport_center);
        egui::pos2(
            (screen.x - origin.x) / self.zoom,
            (screen.y - origin.y) / self.zoom,
        )
    }

    /// Scale a world-space length to screen-space.
    fn scale(&self, world_len: f32) -> f32 {
        world_len * self.zoom
    }
}

pub fn show_viewport(ui: &mut egui::Ui, scene: &mut Scene) {
    let rect = ui.available_rect_before_wrap();
    let center = rect.center();
    let mut cam = Camera::load(ui.ctx());
    let radius = egui::CornerRadius::same(config::PANEL_RADIUS);

    // ── Background with rounded corners ─────────────────────────────────────
    ui.painter().rect_filled(rect, radius, VIEWPORT_BG);

    // ── Clip all content inside the panel ────────────────────────────────────
    ui.set_clip_rect(rect);

    // ── Input handling (Unity/Godot style) ───────────────────────────────────

    // Scroll zoom (no modifier needed)
    let scroll = ui.input(|i| i.smooth_scroll_delta);
    if scroll.y != 0.0 {
        let zoom_factor = 1.0 + scroll.y * 0.001;
        cam.zoom = (cam.zoom * zoom_factor).clamp(0.1, 10.0);
    }

    // Middle-click pan (Unity/Godot style)
    let viewport_id = egui::Id::new("viewport_interaction");
    let response = ui.interact(rect, viewport_id, egui::Sense::click_and_drag());
    if response.dragged_by(egui::PointerButton::Middle) {
        let delta = response.drag_delta();
        cam.pan_x += delta.x / cam.zoom;
        cam.pan_y += delta.y / cam.zoom;
    }

    // Left-click select / drag-to-move
    if response.clicked()
        && let Some(pointer) = ui.input(|i| i.pointer.latest_pos())
    {
        let world_pos = cam.screen_to_world(pointer, center);
        let mut found = false;
        for obj in scene.objects.iter().rev() {
            if hit_test_world(world_pos, obj) {
                scene.select(Some(obj.id));
                found = true;
                break;
            }
        }
        if !found {
            scene.select(None);
        }
    }

    if response.dragged_by(egui::PointerButton::Primary)
        && let Some(obj) = scene.get_selected_mut()
    {
        let delta = response.drag_delta();
        obj.x += delta.x / cam.zoom;
        obj.y += delta.y / cam.zoom;
    }

    // Cursor
    if response.dragged_by(egui::PointerButton::Primary) && scene.selected_id.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    } else if response.dragged_by(egui::PointerButton::Middle) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::AllScroll);
    } else if let Some(pointer) = ui.input(|i| i.pointer.latest_pos())
        && rect.contains(pointer)
    {
        let world_pos = cam.screen_to_world(pointer, center);
        let hovering = scene
            .objects
            .iter()
            .rev()
            .any(|obj| hit_test_world(world_pos, obj));
        if hovering {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
    }

    // ── Grid (in world space) ───────────────────────────────────────────────
    draw_grid(ui, rect, &cam, center);

    // ── Origin crosshair ────────────────────────────────────────────────────
    let origin_screen = cam.world_to_screen(egui::pos2(0.0, 0.0), center);
    let crosshair_color = egui::Color32::from_rgba_premultiplied(255, 255, 255, 40);
    ui.painter().line_segment(
        [
            egui::pos2(origin_screen.x, rect.top()),
            egui::pos2(origin_screen.x, rect.bottom()),
        ],
        egui::Stroke::new(1.0, crosshair_color),
    );
    ui.painter().line_segment(
        [
            egui::pos2(rect.left(), origin_screen.y),
            egui::pos2(rect.right(), origin_screen.y),
        ],
        egui::Stroke::new(1.0, crosshair_color),
    );

    // ── Render objects (clipped to viewport) ────────────────────────────────
    let clip_rect = rect; // painter clip
    for obj in &scene.objects {
        let screen_center = cam.world_to_screen(egui::pos2(obj.x, obj.y), center);
        let sw = cam.scale(obj.width);
        let sh = cam.scale(obj.height);
        let half_w = sw / 2.0;
        let half_h = sh / 2.0;
        let stroke = egui::Stroke::new(1.0, OBJECT_STROKE);
        let rot = obj.rotation.to_radians();

        // Skip if entirely outside viewport
        let bounds = egui::Rect::from_center_size(screen_center, egui::vec2(sw + 4.0, sh + 4.0));
        if !clip_rect.intersects(bounds) {
            continue;
        }

        match obj.shape {
            ShapeType::Rectangle => {
                let corners = rotate_rect(screen_center, half_w, half_h, rot);
                ui.painter().add(egui::Shape::convex_polygon(
                    corners.clone(),
                    OBJECT_FILL,
                    stroke,
                ));
            }
            ShapeType::Circle => {
                let radius = half_w.min(half_h);
                ui.painter().add(egui::Shape::circle_filled(
                    screen_center,
                    radius,
                    OBJECT_FILL,
                ));
                ui.painter()
                    .add(egui::Shape::circle_stroke(screen_center, radius, stroke));
            }
            ShapeType::Triangle => {
                let verts = rotate_triangle(screen_center, half_w, half_h, rot);
                ui.painter()
                    .add(egui::Shape::convex_polygon(verts, OBJECT_FILL, stroke));
            }
        }
    }

    // ── Selection border ────────────────────────────────────────────────────
    if let Some(obj) = scene.get_selected() {
        let screen_center = cam.world_to_screen(egui::pos2(obj.x, obj.y), center);
        let sw = cam.scale(obj.width) + 4.0;
        let sh = cam.scale(obj.height) + 4.0;

        match obj.shape {
            ShapeType::Rectangle => {
                let rot = obj.rotation.to_radians();
                let corners = rotate_rect(screen_center, sw / 2.0, sh / 2.0, rot);
                // Draw selection as a closed polyline
                for i in 0..corners.len() {
                    let a = corners[i];
                    let b = corners[(i + 1) % corners.len()];
                    ui.painter()
                        .line_segment([a, b], egui::Stroke::new(2.0, SELECTION_COLOR));
                }
            }
            ShapeType::Circle => {
                let radius = sw.min(sh) / 2.0;
                ui.painter().add(egui::Shape::circle_stroke(
                    screen_center,
                    radius,
                    egui::Stroke::new(2.0, SELECTION_COLOR),
                ));
            }
            ShapeType::Triangle => {
                let rot = obj.rotation.to_radians();
                let verts = rotate_triangle(screen_center, sw / 2.0, sh / 2.0, rot);
                for i in 0..verts.len() {
                    let a = verts[i];
                    let b = verts[(i + 1) % verts.len()];
                    ui.painter()
                        .line_segment([a, b], egui::Stroke::new(2.0, SELECTION_COLOR));
                }
            }
        }
    }

    // ── Save camera state ───────────────────────────────────────────────────
    cam.save(ui.ctx());
}

// ── Geometry helpers ────────────────────────────────────────────────────────

fn rotate_around(point: egui::Pos2, center: egui::Pos2, cos_a: f32, sin_a: f32) -> egui::Pos2 {
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    egui::pos2(
        center.x + dx * cos_a - dy * sin_a,
        center.y + dx * sin_a + dy * cos_a,
    )
}

fn rotate_rect(center: egui::Pos2, half_w: f32, half_h: f32, radians: f32) -> Vec<egui::Pos2> {
    let cos_a = radians.cos();
    let sin_a = radians.sin();
    vec![
        rotate_around(
            egui::pos2(center.x - half_w, center.y - half_h),
            center,
            cos_a,
            sin_a,
        ),
        rotate_around(
            egui::pos2(center.x + half_w, center.y - half_h),
            center,
            cos_a,
            sin_a,
        ),
        rotate_around(
            egui::pos2(center.x + half_w, center.y + half_h),
            center,
            cos_a,
            sin_a,
        ),
        rotate_around(
            egui::pos2(center.x - half_w, center.y + half_h),
            center,
            cos_a,
            sin_a,
        ),
    ]
}

fn rotate_triangle(center: egui::Pos2, half_w: f32, half_h: f32, radians: f32) -> Vec<egui::Pos2> {
    let cos_a = radians.cos();
    let sin_a = radians.sin();
    let verts = [
        egui::pos2(center.x, center.y - half_h),          // top
        egui::pos2(center.x + half_w, center.y + half_h), // bottom-right
        egui::pos2(center.x - half_w, center.y + half_h), // bottom-left
    ];
    verts
        .iter()
        .map(|v| rotate_around(*v, center, cos_a, sin_a))
        .collect()
}

// ── Hit testing (in world space, no rotation for simplicity) ────────────────

fn hit_test_world(world_pos: egui::Pos2, obj: &khepri_core::scene::SceneObject) -> bool {
    let center = egui::pos2(obj.x, obj.y);
    let half_w = obj.width / 2.0;
    let half_h = obj.height / 2.0;

    match obj.shape {
        ShapeType::Rectangle => {
            let rect = egui::Rect::from_center_size(center, egui::vec2(obj.width, obj.height));
            rect.contains(world_pos)
        }
        ShapeType::Circle => {
            let radius = half_w.min(half_h);
            let dist = ((world_pos.x - center.x).powi(2) + (world_pos.y - center.y).powi(2)).sqrt();
            dist <= radius
        }
        ShapeType::Triangle => {
            let p0 = egui::pos2(center.x, center.y - half_h);
            let p1 = egui::pos2(center.x + half_w, center.y + half_h);
            let p2 = egui::pos2(center.x - half_w, center.y + half_h);
            point_in_triangle(world_pos, p0, p1, p2)
        }
    }
}

fn point_in_triangle(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
    !(has_neg && has_pos)
}

fn sign(p1: egui::Pos2, p2: egui::Pos2, p3: egui::Pos2) -> f32 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

// ── Grid drawing (world space, clipped to viewport) ─────────────────────────

fn draw_grid(ui: &mut egui::Ui, viewport: egui::Rect, cam: &Camera, center: egui::Pos2) {
    // Determine grid spacing: pick a world-space step that gives ~30-80px on screen
    let _target_px = 50.0;

    // Snap to nice numbers
    let nice = [10.0, 20.0, 25.0, 50.0, 100.0, 200.0, 500.0];
    let step = nice
        .iter()
        .copied()
        .find(|&s| s * cam.zoom >= 20.0)
        .unwrap_or(500.0);

    let stroke = egui::Stroke::new(0.5, GRID_COLOR);

    // Find visible world bounds
    let tl = cam.screen_to_world(viewport.left_top(), center);
    let br = cam.screen_to_world(viewport.right_bottom(), center);

    let min_x = (tl.x / step).floor() * step;
    let max_x = (br.x / step).ceil() * step;
    let min_y = (tl.y / step).floor() * step;
    let max_y = (br.y / step).ceil() * step;

    let mut x = min_x;
    while x <= max_x {
        let sx = cam.world_to_screen(egui::pos2(x, 0.0), center).x;
        ui.painter().line_segment(
            [
                egui::pos2(sx, viewport.top()),
                egui::pos2(sx, viewport.bottom()),
            ],
            stroke,
        );
        x += step;
    }

    let mut y = min_y;
    while y <= max_y {
        let sy = cam.world_to_screen(egui::pos2(0.0, y), center).y;
        ui.painter().line_segment(
            [
                egui::pos2(viewport.left(), sy),
                egui::pos2(viewport.right(), sy),
            ],
            stroke,
        );
        y += step;
    }
}
