//! danji-ctrl - 真空管放大器 GUI 控制器。
//!
//! 本程序提供图形用户界面，用于控制实时仿真守护进程的参数。
//!
//! ---
//!
//! danji-ctrl - Vacuum tube amplifier GUI controller.
//!
//! This program provides a graphical user interface for controlling
//! real-time simulation daemon parameters.

mod daemon;

use daemon::{AppState, DaemonCtrl};
use eframe::egui;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn main() -> Result<(), eframe::Error> {
    let (state_tx, state_rx) = mpsc::channel::<AppState>();
    let ctrl = DaemonCtrl::connect(state_tx).unwrap_or_else(|e| {
        eprintln!("Warning: daemon not running ({e})");
        let (tx, _) = mpsc::channel();
        DaemonCtrl::from_fallback(tx)
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([380.0, 480.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "danji-ctrl",
        options,
        Box::new(|_cc| {
            Ok(Box::new(DaemonApp {
                ctrl,
                state_rx,
                state: AppState::default(),
                last_poll: Instant::now(),
            }))
        }),
    )
}

struct DaemonApp {
    ctrl: DaemonCtrl,
    state_rx: mpsc::Receiver<AppState>,
    state: AppState,
    last_poll: Instant,
}

impl eframe::App for DaemonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(new) = self.state_rx.try_recv() {
            self.state = new;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("danji-ctrl");

            if !self.state.connected {
                ui.colored_label(egui::Color32::RED, "⚠ Daemon not connected");
                ui.label("Start danji-realtime first, then relaunch this app.");
                return;
            }

            ui.horizontal(|ui| {
                ui.label("Model:");
                let models = ["single", "two-stage", "chain"];
                let mut cur = models
                    .iter()
                    .position(|m| *m == self.state.model)
                    .unwrap_or(0);
                egui::ComboBox::from_id_salt("model")
                    .selected_text(&self.state.model)
                    .show_ui(ui, |ui| {
                        for (i, m) in models.iter().enumerate() {
                            if ui.selectable_label(cur == i, *m).clicked() {
                                self.ctrl.send(format!("model {m}"));
                                cur = i;
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Tube:");
                let tubes = [
                    "12AX7", "12AU7", "12AT7", "6DJ8", "6L6GC", "6550", "EL34", "KT88",
                ];
                let mut cur = tubes
                    .iter()
                    .position(|t| *t == self.state.tube)
                    .unwrap_or(0);
                egui::ComboBox::from_id_salt("tube")
                    .selected_text(&self.state.tube)
                    .show_ui(ui, |ui| {
                        for (i, t) in tubes.iter().enumerate() {
                            if ui.selectable_label(cur == i, *t).clicked() {
                                self.ctrl.send(format!("tube {t}"));
                                cur = i;
                            }
                        }
                    });
            });

            ui.separator();

            let mut vol = self.state.volume;
            ui.label("Volume");
            if ui
                .add(egui::Slider::new(&mut vol, -60.0..=12.0).suffix(" dB"))
                .changed()
            {
                self.ctrl.send(format!("volume {vol:.1}"));
                self.state.volume = vol;
            }

            let mut gain = self.state.gain;
            ui.label("Gain");
            if ui
                .add(egui::Slider::new(&mut gain, -96.0..=10.0).suffix(" dB"))
                .changed()
            {
                self.ctrl.send(format!("gain {gain:.1}"));
                self.state.gain = gain;
            }

            let mut bp = self.state.bplus;
            ui.label("B+ Voltage");
            if ui
                .add(egui::Slider::new(&mut bp, 50.0..=600.0).suffix(" V"))
                .changed()
            {
                self.ctrl.send(format!("bplus {bp:.0}"));
                self.state.bplus = bp;
            }

            ui.separator();

            let label = if self.state.bypass {
                egui::RichText::new("Bypass: ON").color(egui::Color32::YELLOW)
            } else {
                egui::RichText::new("Bypass: OFF").color(egui::Color32::GREEN)
            };
            if ui.button(label).clicked() {
                let val = if self.state.bypass { "off" } else { "on" };
                self.ctrl.send(format!("bypass {val}"));
                self.state.bypass = !self.state.bypass;
            }

            if !self.state.status_msg.is_empty() {
                ui.label(format!("Last: {}", self.state.status_msg));
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                if ui.button("Stop Daemon").clicked() {
                    self.ctrl.stop();
                    self.state.connected = false;
                }
            });
        });

        if self.last_poll.elapsed() > Duration::from_millis(200) {
            ctx.request_repaint();
            self.last_poll = Instant::now();
        }
    }
}
