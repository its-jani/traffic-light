use std::collections::HashMap;
use std::time::{Duration, Instant};
use crossbeam_channel::Receiver;
use eframe::egui::{self, Color32, Pos2, Rect, Rounding, Stroke, Vec2};

use crate::types::{IpcCommand, LightState, SessionInfo};

pub struct TrafficLightApp {
    rx: Receiver<IpcCommand>,
    sessions: HashMap<String, SessionInfo>,
    session_order: Vec<String>,
    active_session_id: Option<String>,
    pulse_phase: f32,
    last_frame_time: Instant,
    last_width: f32,
}

impl TrafficLightApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, rx: Receiver<IpcCommand>) -> Self {
        let mut sessions = HashMap::new();
        let default_id = "agent-1".to_string();
        sessions.insert(
            default_id.clone(),
            SessionInfo::new(default_id.clone(), Some("Session #1".to_string()), Some(LightState::Green)),
        );

        Self {
            rx,
            sessions,
            session_order: vec![default_id.clone()],
            active_session_id: Some(default_id),
            pulse_phase: 0.0,
            last_frame_time: Instant::now(),
            last_width: 88.0,
        }
    }

    fn process_ipc_events(&mut self) {
        while let Ok(cmd) = self.rx.try_recv() {
            match cmd {
                IpcCommand::SetState {
                    session_id,
                    state,
                    label,
                    message,
                } => {
                    // Replace placeholder session if this is the first real session connecting
                    if self.sessions.len() == 1 && self.sessions.contains_key("agent-1") && session_id != "agent-1" {
                        self.sessions.remove("agent-1");
                        self.session_order.retain(|id| id != "agent-1");
                    }

                    if let Some(session) = self.sessions.get_mut(&session_id) {
                        session.state = state;
                        if let Some(lbl) = label {
                            session.label = lbl;
                        }
                        if message.is_some() {
                            session.message = message;
                        }
                        session.last_updated = Instant::now();
                    } else {
                        let mut session = SessionInfo::new(session_id.clone(), label, Some(state));
                        session.message = message;
                        self.sessions.insert(session_id.clone(), session);
                        if !self.session_order.contains(&session_id) {
                            self.session_order.push(session_id.clone());
                        }
                    }
                    self.active_session_id = Some(session_id);
                }
                IpcCommand::SessionOn {
                    session_id,
                    label,
                    initial_state,
                } => {
                    // Replace placeholder session if this is the first real session connecting
                    if self.sessions.len() == 1 && self.sessions.contains_key("agent-1") && session_id != "agent-1" {
                        self.sessions.remove("agent-1");
                        self.session_order.retain(|id| id != "agent-1");
                    }

                    if let Some(session) = self.sessions.get_mut(&session_id) {
                        if let Some(lbl) = label {
                            session.label = lbl;
                        }
                        if let Some(st) = initial_state {
                            session.state = st;
                        }
                        session.last_updated = Instant::now();
                    } else {
                        let session = SessionInfo::new(session_id.clone(), label, initial_state);
                        self.sessions.insert(session_id.clone(), session);
                        if !self.session_order.contains(&session_id) {
                            self.session_order.push(session_id.clone());
                        }
                    }
                    self.active_session_id = Some(session_id);
                }
                IpcCommand::SessionOff { session_id } => {
                    self.sessions.remove(&session_id);
                    self.session_order.retain(|id| id != &session_id);
                    if self.active_session_id.as_deref() == Some(&session_id) {
                        self.active_session_id = self.session_order.last().cloned();
                    }
                }
                IpcCommand::ClearAll => {
                    self.sessions.clear();
                    self.session_order.clear();
                    self.active_session_id = None;
                }
            }
        }
    }

    fn has_active_animation(&self) -> bool {
        self.sessions.values().any(|s| s.state == LightState::Yellow)
    }
}

impl eframe::App for TrafficLightApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_ipc_events();

        // Dynamically adjust window size if multiple session tabs are present
        let needed_width = if self.session_order.len() > 1 { 120.0 } else { 88.0 };
        if (needed_width - self.last_width).abs() > 0.5 {
            self.last_width = needed_width;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(needed_width, 190.0)));
        }

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        if self.has_active_animation() {
            self.pulse_phase = (self.pulse_phase + dt * 4.2) % std::f32::consts::TAU;
            ctx.request_repaint_after(Duration::from_millis(33));
        } else {
            ctx.request_repaint_after(Duration::from_millis(1000));
        }

        // Keep active session valid
        if self.active_session_id.is_none() && !self.session_order.is_empty() {
            self.active_session_id = self.session_order.first().cloned();
        }

        let frame = egui::Frame::none()
            .fill(Color32::from_rgba_unmultiplied(14, 16, 22, 235))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 30)))
            .rounding(Rounding::same(16.0))
            .inner_margin(egui::Margin::symmetric(6.0, 6.0))
            .shadow(egui::epaint::Shadow {
                offset: Vec2::new(0.0, 6.0),
                blur: 14.0,
                spread: 0.0,
                color: Color32::from_black_alpha(160),
            });

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            // Drag entire widget anywhere
            let panel_rect = ui.max_rect();
            let drag_response = ui.interact(panel_rect, ui.id().with("window_drag"), egui::Sense::drag());
            if drag_response.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            // Top control header (Close & Minimize buttons)
            ui.horizontal(|ui| {
                // Drag handle dots
                ui.label(
                    egui::RichText::new("⠿")
                        .color(Color32::from_rgb(90, 95, 115))
                        .size(11.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Close button
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("✕")
                                    .size(10.0)
                                    .color(Color32::from_rgb(160, 165, 180)),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Close Traffic Light")
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    // Minimize button
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("—")
                                    .size(10.0)
                                    .color(Color32::from_rgb(160, 165, 180)),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Minimize Traffic Light")
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });

            ui.add_space(3.0);

            // Active session info
            let active_session = self
                .active_session_id
                .as_ref()
                .and_then(|id| self.sessions.get(id))
                .cloned();

            let has_multiple_sessions = self.session_order.len() > 1;

            ui.horizontal(|ui| {
                // Vertical Tabs column (when multiple sessions exist)
                if has_multiple_sessions {
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        for (idx, session_id) in self.session_order.iter().enumerate() {
                            if let Some(sess) = self.sessions.get(session_id) {
                                let is_selected = self.active_session_id.as_deref() == Some(session_id);
                                let tab_dot_color = match sess.state {
                                    LightState::Red => Color32::from_rgb(255, 60, 60),
                                    LightState::Yellow => Color32::from_rgb(255, 210, 40),
                                    LightState::Green => Color32::from_rgb(50, 230, 110),
                                    LightState::Off => Color32::from_rgb(80, 85, 100),
                                };

                                let tab_btn = egui::Button::new(
                                    egui::RichText::new(format!("{}", idx + 1))
                                        .size(10.0)
                                        .color(if is_selected {
                                            Color32::WHITE
                                        } else {
                                            Color32::from_rgb(140, 145, 165)
                                        })
                                        .strong(),
                                )
                                .fill(if is_selected {
                                    Color32::from_rgba_unmultiplied(45, 50, 70, 220)
                                } else {
                                    Color32::from_rgba_unmultiplied(20, 24, 34, 160)
                                })
                                .stroke(Stroke::new(
                                    1.0_f32,
                                    if is_selected {
                                        tab_dot_color
                                    } else {
                                        Color32::from_rgba_unmultiplied(255, 255, 255, 15)
                                    },
                                ))
                                .rounding(Rounding::same(6.0));

                                let hover_text = format!(
                                    "{}: {}\nStatus: {}",
                                    sess.label,
                                    sess.state.display_name(),
                                    sess.message.as_deref().unwrap_or("No message")
                                );

                                if ui.add(tab_btn).on_hover_text(hover_text).clicked() {
                                    self.active_session_id = Some(session_id.clone());
                                }

                                ui.add_space(2.0);
                            }
                        }
                    });

                    ui.add_space(2.0);
                }

                // Vertical Authentic Traffic Light
                ui.vertical_centered(|ui| {
                    if let Some(session) = &active_session {
                        render_vertical_traffic_light(ui, session.state, self.pulse_phase);

                        // Minimal tooltip info on hover
                        let elapsed = session.last_updated.elapsed().as_secs();
                        let time_str = if elapsed < 60 {
                            format!("{elapsed}s ago")
                        } else {
                            format!("{}m ago", elapsed / 60)
                        };

                        let status_text = session.message.as_deref().unwrap_or(session.state.display_name());
                        ui.label(
                            egui::RichText::new(format!("• {status_text}"))
                                .size(9.5)
                                .color(match session.state {
                                    LightState::Red => Color32::from_rgb(255, 120, 120),
                                    LightState::Yellow => Color32::from_rgb(255, 215, 80),
                                    LightState::Green => Color32::from_rgb(100, 230, 140),
                                    LightState::Off => Color32::from_rgb(110, 120, 135),
                                }),
                        )
                        .on_hover_text(format!("Session: {}\nUpdated: {}", session.label, time_str));
                    } else {
                        // Empty / standby state
                        render_vertical_traffic_light(ui, LightState::Off, 0.0);
                        ui.label(
                            egui::RichText::new("Standby")
                                .size(9.0)
                                .color(Color32::from_rgb(100, 110, 130)),
                        );
                    }
                });
            });
        });
    }
}

/// Renders the authentic vertical traffic light with hooded visors and rich glow
fn render_vertical_traffic_light(ui: &mut egui::Ui, state: LightState, pulse_phase: f32) {
    let bulb_radius = 9.0;
    let bulb_diameter = bulb_radius * 2.0;
    let spacing = 7.0;
    let padding_y = 6.0;
    let padding_x = 6.0;

    let housing_width = bulb_diameter + padding_x * 2.0 + 4.0;
    let housing_height = bulb_diameter * 3.0 + spacing * 2.0 + padding_y * 2.0;

    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(housing_width, housing_height),
        egui::Sense::hover(),
    );

    let painter = ui.painter_at(rect);

    // Physical Traffic Light Bezel Housing (Obsidian Dark with Beveled Edge)
    painter.rect_filled(
        rect,
        Rounding::same(bulb_radius + 4.0),
        Color32::from_rgba_unmultiplied(8, 10, 14, 245),
    );
    painter.rect_stroke(
        rect,
        Rounding::same(bulb_radius + 4.0),
        Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 30)),
    );

    let center_x = rect.center().x;
    let start_y = rect.min.y + padding_y + bulb_radius;

    let centers = [
        Pos2::new(center_x, start_y),                               // RED (Top)
        Pos2::new(center_x, start_y + bulb_diameter + spacing),     // YELLOW (Middle)
        Pos2::new(center_x, start_y + (bulb_diameter + spacing) * 2.0), // GREEN (Bottom)
    ];

    // Draw Hood Visors above each lamp for realistic traffic light look
    for &c in &centers {
        let hood_rect = Rect::from_min_max(
            Pos2::new(c.x - bulb_radius - 2.0, c.y - bulb_radius - 3.0),
            Pos2::new(c.x + bulb_radius + 2.0, c.y - bulb_radius + 1.0),
        );
        painter.rect_filled(
            hood_rect,
            Rounding::same(2.0),
            Color32::from_rgba_unmultiplied(20, 24, 32, 230),
        );
    }

    // 1. Draw RED bulb (Top)
    let is_red = state == LightState::Red;
    draw_traffic_bulb(
        &painter,
        centers[0],
        bulb_radius,
        is_red,
        Color32::from_rgb(255, 40, 40),
        Color32::from_rgb(38, 12, 12),
        Color32::from_rgba_unmultiplied(255, 45, 45, 120),
        0.0,
    );

    // 2. Draw YELLOW bulb (Middle) with breathing glow pulse
    let is_yellow = state == LightState::Yellow;
    let yellow_pulse = if is_yellow {
        ((pulse_phase.sin() + 1.0) / 2.0) * 0.45 + 0.55
    } else {
        0.0
    };
    draw_traffic_bulb(
        &painter,
        centers[1],
        bulb_radius,
        is_yellow,
        Color32::from_rgb(255, 205, 25),
        Color32::from_rgb(40, 32, 8),
        Color32::from_rgba_unmultiplied(255, 210, 35, (150.0 * yellow_pulse) as u8),
        yellow_pulse,
    );

    // 3. Draw GREEN bulb (Bottom)
    let is_green = state == LightState::Green;
    draw_traffic_bulb(
        &painter,
        centers[2],
        bulb_radius,
        is_green,
        Color32::from_rgb(40, 235, 100),
        Color32::from_rgb(10, 35, 18),
        Color32::from_rgba_unmultiplied(45, 240, 110, 120),
        0.0,
    );

    // Tooltip on hover
    response.on_hover_text(match state {
        LightState::Red => "🔴 Needs Input / Error / Halted",
        LightState::Yellow => "🟡 Thinking / Generating / Working",
        LightState::Green => "🟢 Ready / Done / Idle",
        LightState::Off => "⚪ Standby",
    });
}

/// Helper function to draw an individual traffic light bulb with glass lens highlights and glowing neon halos
#[allow(clippy::too_many_arguments)]
fn draw_traffic_bulb(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    is_active: bool,
    lit_color: Color32,
    unlit_color: Color32,
    glow_color: Color32,
    pulse_intensity: f32,
) {
    if is_active {
        // Outer diffuse bloom halo (Layer 1)
        let glow_rad_1 = radius * (2.1 + pulse_intensity * 0.4);
        painter.circle_filled(
            center,
            glow_rad_1,
            Color32::from_rgba_unmultiplied(
                glow_color.r(),
                glow_color.g(),
                glow_color.b(),
                (40.0 * (1.0 + pulse_intensity)) as u8,
            ),
        );

        // Mid glow halo (Layer 2)
        let glow_rad_2 = radius * (1.5 + pulse_intensity * 0.25);
        painter.circle_filled(
            center,
            glow_rad_2,
            Color32::from_rgba_unmultiplied(
                glow_color.r(),
                glow_color.g(),
                glow_color.b(),
                (85.0 * (1.0 + pulse_intensity)) as u8,
            ),
        );

        // Lit bulb body
        painter.circle_filled(center, radius, lit_color);

        // Inner bright white hot center
        painter.circle_filled(
            center,
            radius * 0.45,
            Color32::from_rgba_unmultiplied(255, 255, 255, 175),
        );

        // Top-left specular glint / glass shine
        let glint_pos = Pos2::new(center.x - radius * 0.28, center.y - radius * 0.30);
        painter.circle_filled(
            glint_pos,
            radius * 0.24,
            Color32::from_rgba_unmultiplied(255, 255, 255, 230),
        );

        // Outer lens rim
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 190)),
        );
    } else {
        // Unlit bulb - dark translucent glass with subtle rim
        painter.circle_filled(center, radius, unlit_color);
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 18)),
        );

        // Subtle specular highlight on dark glass
        let glint_pos = Pos2::new(center.x - radius * 0.25, center.y - radius * 0.28);
        painter.circle_filled(
            glint_pos,
            radius * 0.16,
            Color32::from_rgba_unmultiplied(255, 255, 255, 25),
        );
    }
}
