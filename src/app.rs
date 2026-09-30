use std::collections::HashMap;
use std::time::{Duration, Instant};
use crossbeam_channel::{Receiver, Sender};
use eframe::egui::{self, Color32, Pos2, Rounding, Stroke, Vec2};

use crate::types::{IpcCommand, LightState, SessionInfo, WidgetOrientation};

pub struct TrafficLightApp {
    rx: Receiver<IpcCommand>,
    #[allow(dead_code)]
    tx: Sender<IpcCommand>,
    sessions: HashMap<String, SessionInfo>,
    session_order: Vec<String>,
    orientation: WidgetOrientation,
    pulse_phase: f32,
    last_frame_time: Instant,
    compact_mode: bool,
}

impl TrafficLightApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, rx: Receiver<IpcCommand>, tx: Sender<IpcCommand>) -> Self {
        // Start with an initial standby demo session so the widget is immediately visible
        let mut sessions = HashMap::new();
        let default_id = "agent-1".to_string();
        sessions.insert(
            default_id.clone(),
            SessionInfo::new(default_id.clone(), Some("Agent (Ready)".to_string()), Some(LightState::Green)),
        );

        Self {
            rx,
            tx,
            sessions,
            session_order: vec![default_id],
            orientation: WidgetOrientation::Horizontal,
            pulse_phase: 0.0,
            last_frame_time: Instant::now(),
            compact_mode: false,
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
                        self.session_order.push(session_id);
                    }
                }
                IpcCommand::SessionOn {
                    session_id,
                    label,
                    initial_state,
                } => {
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
                        self.session_order.push(session_id);
                    }
                }
                IpcCommand::SessionOff { session_id } => {
                    self.sessions.remove(&session_id);
                    self.session_order.retain(|id| id != &session_id);
                }
                IpcCommand::ClearAll => {
                    self.sessions.clear();
                    self.session_order.clear();
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
        // Transparent frame background
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_ipc_events();

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        if self.has_active_animation() {
            self.pulse_phase = (self.pulse_phase + dt * 4.0) % (std::f32::consts::TAU);
            // Repaint at ~30 FPS for smooth pulsing glow when yellow (working)
            ctx.request_repaint_after(Duration::from_millis(33));
        } else {
            // Idle near-zero CPU: schedule repaint every 1s for clock updates, or wait for IPC events
            ctx.request_repaint_after(Duration::from_millis(1000));
        }

        // Custom styling for sleek dark glassmorphism
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = Color32::from_rgba_unmultiplied(16, 18, 24, 230);
        style.visuals.window_stroke = Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 35));
        style.visuals.window_rounding = Rounding::same(12.0);
        ctx.set_style(style);

        // Frame wrapper
        let frame = egui::Frame::none()
            .fill(Color32::from_rgba_unmultiplied(16, 18, 24, 230))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 40)))
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .shadow(egui::epaint::Shadow {
                offset: Vec2::new(0.0, 4.0),
                blur: 8.0,
                spread: 0.0,
                color: Color32::from_black_alpha(120),
            });

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            // Drag handle & Header Bar
            ui.horizontal(|ui| {
                // Drag handle area
                let drag_response = ui.add(
                    egui::Label::new(
                        egui::RichText::new("⠿")
                            .color(Color32::from_rgb(140, 145, 165))
                            .size(14.0)
                            .strong(),
                    )
                    .sense(egui::Sense::drag()),
                );

                if drag_response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.heading(
                    egui::RichText::new("Traffic Light")
                        .color(Color32::from_rgb(235, 240, 255))
                        .size(13.0)
                        .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Close application button
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("✕")
                                    .size(11.0)
                                    .color(Color32::from_rgb(180, 180, 190)),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Quit Traffic Light")
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    // Compact / Expanded toggle
                    let mode_icon = if self.compact_mode { "⇲" } else { "⇱" };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(mode_icon)
                                    .size(11.0)
                                    .color(Color32::from_rgb(180, 180, 190)),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Toggle Compact Mode")
                        .clicked()
                    {
                        self.compact_mode = !self.compact_mode;
                    }

                    // Orientation toggle button
                    let orient_icon = match self.orientation {
                        WidgetOrientation::Horizontal => "↔",
                        WidgetOrientation::Vertical => "↕",
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(orient_icon)
                                    .size(12.0)
                                    .color(Color32::from_rgb(180, 180, 190)),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Toggle Light Orientation")
                        .clicked()
                    {
                        self.orientation = match self.orientation {
                            WidgetOrientation::Horizontal => WidgetOrientation::Vertical,
                            WidgetOrientation::Vertical => WidgetOrientation::Horizontal,
                        };
                    }

                    // Active session count badge
                    let count = self.sessions.len();
                    ui.label(
                        egui::RichText::new(format!("{count} active"))
                            .color(Color32::from_rgb(130, 140, 160))
                            .size(10.0),
                    );
                });
            });

            ui.add_space(6.0);

            // If no sessions, show clean empty/standby message
            if self.sessions.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("No active agent sessions")
                            .color(Color32::from_rgb(130, 140, 160))
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new("Listening on port 8765")
                            .color(Color32::from_rgb(80, 90, 110))
                            .size(10.0),
                    );
                    ui.add_space(4.0);
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("+ Add Test Session")
                                    .size(11.0)
                                    .color(Color32::from_rgb(100, 200, 255)),
                            )
                            .fill(Color32::from_rgba_unmultiplied(40, 50, 70, 180))
                            .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        let id = format!("agent-{}", self.session_order.len() + 1);
                        self.sessions.insert(
                            id.clone(),
                            SessionInfo::new(id.clone(), Some("Agent #1".to_string()), Some(LightState::Green)),
                        );
                        self.session_order.push(id);
                    }
                    ui.add_space(10.0);
                });
                return;
            }

            // Render all active sessions
            let mut session_to_remove: Option<String> = None;
            let current_pulse = self.pulse_phase;
            let is_compact = self.compact_mode;
            let orientation = self.orientation;

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(400.0)
                .show(ui, |ui| {
                    for session_id in &self.session_order {
                        if let Some(session) = self.sessions.get(session_id) {
                            let card_frame = egui::Frame::none()
                                .fill(Color32::from_rgba_unmultiplied(26, 30, 42, 220))
                                .stroke(Stroke::new(
                                    1.0_f32,
                                    match session.state {
                                        LightState::Red => Color32::from_rgba_unmultiplied(255, 70, 70, 80),
                                        LightState::Yellow => Color32::from_rgba_unmultiplied(255, 200, 50, 80),
                                        LightState::Green => Color32::from_rgba_unmultiplied(60, 220, 100, 60),
                                        LightState::Off => Color32::from_rgba_unmultiplied(255, 255, 255, 20),
                                    },
                                ))
                                .rounding(Rounding::same(8.0))
                                .inner_margin(egui::Margin::symmetric(8.0, 6.0));

                            card_frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Session info column
                                    if !is_compact {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&session.label)
                                                        .color(Color32::from_rgb(240, 245, 255))
                                                        .size(12.0)
                                                        .strong(),
                                                );

                                                // Quick dismiss 'x' button for session
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new("×")
                                                                .size(11.0)
                                                                .color(Color32::from_rgb(130, 135, 150)),
                                                        )
                                                        .frame(false),
                                                    )
                                                    .on_hover_text("Dismiss session")
                                                    .clicked()
                                                {
                                                    session_to_remove = Some(session.id.clone());
                                                }
                                            });

                                            // Subtitle status text & elapsed time
                                            let elapsed = session.last_updated.elapsed().as_secs();
                                            let time_str = if elapsed < 60 {
                                                format!("{elapsed}s ago")
                                            } else {
                                                format!("{}m ago", elapsed / 60)
                                            };

                                            let status_desc = if let Some(msg) = &session.message {
                                                msg.as_str()
                                            } else {
                                                session.state.display_name()
                                            };

                                            ui.label(
                                                egui::RichText::new(format!("{status_desc} • {time_str}"))
                                                    .color(match session.state {
                                                        LightState::Red => Color32::from_rgb(255, 120, 120),
                                                        LightState::Yellow => Color32::from_rgb(255, 215, 80),
                                                        LightState::Green => Color32::from_rgb(100, 230, 140),
                                                        LightState::Off => Color32::from_rgb(120, 130, 145),
                                                    })
                                                    .size(10.0),
                                            );
                                        });

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            render_traffic_light(ui, session.state, orientation, current_pulse);
                                        });
                                    } else {
                                        // Compact mode: label + light side-by-side
                                        ui.label(
                                            egui::RichText::new(&session.label)
                                                .color(Color32::from_rgb(220, 225, 240))
                                                .size(11.0),
                                        );
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            render_traffic_light(ui, session.state, orientation, current_pulse);
                                        });
                                    }
                                });
                            });

                            ui.add_space(4.0);
                        }
                    }
                });

            if let Some(id) = session_to_remove {
                self.sessions.remove(&id);
                self.session_order.retain(|s| s != &id);
            }
        });
    }
}

/// Renders the 3 physical traffic light circles (Red, Yellow, Green)
fn render_traffic_light(
    ui: &mut egui::Ui,
    state: LightState,
    orientation: WidgetOrientation,
    pulse_phase: f32,
) {
    let bulb_radius = 6.5;
    let bulb_diameter = bulb_radius * 2.0;
    let spacing = 6.0;

    let total_size = match orientation {
        WidgetOrientation::Horizontal => Vec2::new(bulb_diameter * 3.0 + spacing * 2.0 + 8.0, bulb_diameter + 8.0),
        WidgetOrientation::Vertical => Vec2::new(bulb_diameter + 8.0, bulb_diameter * 3.0 + spacing * 2.0 + 8.0),
    };

    let (rect, _response) = ui.allocate_exact_size(total_size, egui::Sense::hover());

    let painter = ui.painter_at(rect);

    // Dark bezel housing background
    painter.rect_filled(
        rect,
        Rounding::same(bulb_radius + 4.0),
        Color32::from_rgba_unmultiplied(10, 12, 16, 220),
    );
    painter.rect_stroke(
        rect,
        Rounding::same(bulb_radius + 4.0),
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 25)),
    );

    // Calculate centers for Red, Yellow, Green
    let center = rect.center();
    let centers = match orientation {
        WidgetOrientation::Horizontal => [
            Pos2::new(center.x - (bulb_diameter + spacing), center.y), // Red
            Pos2::new(center.x, center.y),                             // Yellow
            Pos2::new(center.x + (bulb_diameter + spacing), center.y), // Green
        ],
        WidgetOrientation::Vertical => [
            Pos2::new(center.x, center.y - (bulb_diameter + spacing)), // Red
            Pos2::new(center.x, center.y),                             // Yellow
            Pos2::new(center.x, center.y + (bulb_diameter + spacing)), // Green
        ],
    };

    // Draw RED light
    let is_red = state == LightState::Red;
    draw_bulb(
        &painter,
        centers[0],
        bulb_radius,
        is_red,
        Color32::from_rgb(255, 45, 45),
        Color32::from_rgb(50, 15, 15),
        Color32::from_rgba_unmultiplied(255, 50, 50, 110),
        0.0,
    );

    // Draw YELLOW light
    let is_yellow = state == LightState::Yellow;
    let yellow_pulse = if is_yellow {
        ((pulse_phase.sin() + 1.0) / 2.0) * 0.4 + 0.6 // 0.6 to 1.0 multiplier
    } else {
        0.0
    };
    draw_bulb(
        &painter,
        centers[1],
        bulb_radius,
        is_yellow,
        Color32::from_rgb(255, 205, 30),
        Color32::from_rgb(50, 42, 10),
        Color32::from_rgba_unmultiplied(255, 210, 40, (140.0 * yellow_pulse) as u8),
        yellow_pulse,
    );

    // Draw GREEN light
    let is_green = state == LightState::Green;
    draw_bulb(
        &painter,
        centers[2],
        bulb_radius,
        is_green,
        Color32::from_rgb(45, 235, 105),
        Color32::from_rgb(12, 45, 22),
        Color32::from_rgba_unmultiplied(50, 240, 115, 110),
        0.0,
    );
}

/// Helper function to draw an individual traffic light bulb with glowing halos and glass lens highlights
#[allow(clippy::too_many_arguments)]
fn draw_bulb(
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
        // Outer glow halo (layer 1)
        let glow_rad_1 = radius * (1.9 + pulse_intensity * 0.4);
        painter.circle_filled(center, glow_rad_1, Color32::from_rgba_unmultiplied(glow_color.r(), glow_color.g(), glow_color.b(), (35.0 * (1.0 + pulse_intensity)) as u8));

        // Outer glow halo (layer 2)
        let glow_rad_2 = radius * (1.4 + pulse_intensity * 0.2);
        painter.circle_filled(center, glow_rad_2, Color32::from_rgba_unmultiplied(glow_color.r(), glow_color.g(), glow_color.b(), (70.0 * (1.0 + pulse_intensity)) as u8));

        // Main lit bulb body
        painter.circle_filled(center, radius, lit_color);

        // Inner bright core
        painter.circle_filled(center, radius * 0.5, Color32::from_rgba_unmultiplied(255, 255, 255, 160));

        // Top specular glint / glass shine
        let glint_pos = Pos2::new(center.x - radius * 0.25, center.y - radius * 0.28);
        painter.circle_filled(glint_pos, radius * 0.22, Color32::from_rgba_unmultiplied(255, 255, 255, 220));

        // Bulb border rim
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 180)),
        );
    } else {
        // Unlit bulb - dark translucent bezel with subtle rim
        painter.circle_filled(center, radius, unlit_color);
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 20)),
        );

        // Subtle specular highlight on dark glass
        let glint_pos = Pos2::new(center.x - radius * 0.25, center.y - radius * 0.28);
        painter.circle_filled(glint_pos, radius * 0.18, Color32::from_rgba_unmultiplied(255, 255, 255, 30));
    }
}
