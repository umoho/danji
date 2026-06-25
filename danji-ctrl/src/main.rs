use eframe::egui;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SOCKET_PATH: &str = "/tmp/danji.sock";

#[derive(Clone)]
struct AppState {
    connected: bool,
    bypass: bool,
    volume: f32,
    gain: f32,
    bplus: f32,
    tube: String,
    model: String,
    status_msg: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connected: false,
            bypass: false,
            volume: -12.0,
            gain: -38.0,
            bplus: 300.0,
            tube: "12AX7".into(),
            model: "single".into(),
            status_msg: String::new(),
        }
    }
}

struct DaemonCtrl {
    tx: mpsc::Sender<DaemonCmd>,
}

enum DaemonCmd {
    Set(String),
    Stop,
}

impl DaemonCtrl {
    fn connect() -> Result<(Self, mpsc::Receiver<AppState>), String> {
        let mut stream = UnixStream::connect(SOCKET_PATH).map_err(|e| format!("connect: {e}"))?;

        let (cmd_tx, cmd_rx) = mpsc::channel::<DaemonCmd>();
        let (state_tx, state_rx) = mpsc::channel::<AppState>();
        let state = Arc::new(Mutex::new(AppState::default()));

        // Initial status query
        writeln!(stream, "status").map_err(|e| format!("send: {e}"))?;
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut resp = String::new();
        reader
            .read_line(&mut resp)
            .map_err(|e| format!("read: {e}"))?;
        {
            let mut s = state.lock().unwrap();
            parse_status(&resp, &mut s);
            s.connected = true;
            state_tx.send(s.clone()).ok();
        }

        let state_clone = state.clone();
        std::thread::spawn(move || {
            let mut stream = stream;
            loop {
                match cmd_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(DaemonCmd::Set(cmd)) => {
                        if writeln!(stream, "{cmd}").is_err() {
                            break;
                        }
                        let mut resp = String::new();
                        let mut reader = BufReader::new(stream.try_clone().unwrap());
                        reader.read_line(&mut resp).ok();
                        let resp = resp.trim().to_string();
                        let mut s = state_clone.lock().unwrap();
                        s.status_msg = resp.clone();
                        if cmd == "stop" {
                            break;
                        }
                        drop(s);
                        // Refresh full status after command
                        writeln!(stream, "status").ok();
                        let mut resp2 = String::new();
                        reader.read_line(&mut resp2).ok();
                        let mut s = state_clone.lock().unwrap();
                        parse_status(&resp2, &mut s);
                        state_tx.send(s.clone()).ok();
                    }
                    Ok(DaemonCmd::Stop) => {
                        let _ = writeln!(stream, "stop");
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Periodic refresh
                        writeln!(stream, "status").ok();
                        let mut resp = String::new();
                        let mut reader = BufReader::new(stream.try_clone().unwrap());
                        reader.read_line(&mut resp).ok();
                        let mut s = state_clone.lock().unwrap();
                        parse_status(&resp, &mut s);
                        state_tx.send(s.clone()).ok();
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Ok((
            Self {
                tx: cmd_tx,
            },
            state_rx,
        ))
    }

    fn send(&self, cmd: String) {
        let _ = self.tx.send(DaemonCmd::Set(cmd));
    }

    fn stop(&self) {
        let _ = self.tx.send(DaemonCmd::Stop);
    }
}

fn parse_status(line: &str, state: &mut AppState) {
    for part in line.split_whitespace() {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().unwrap_or("");
        let val = kv.next().unwrap_or("");
        match key {
            "bypass" => state.bypass = val == "on",
            "volume" => state.volume = val.parse().unwrap_or(-12.0),
            "gain" => state.gain = val.parse().unwrap_or(-38.0),
            "bplus" => state.bplus = val.parse().unwrap_or(300.0),
            "tube" => state.tube = val.to_string(),
            "model" => state.model = val.to_string(),
            _ => {}
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let (ctrl, state_rx) = DaemonCtrl::connect().unwrap_or_else(|e| {
        eprintln!("Warning: daemon not running ({e})");
        let (tx, _rx) = mpsc::channel();
        let (state_tx, state_rx) = mpsc::channel();
        let state = Arc::new(Mutex::new(AppState::default()));
        let st = state.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            state_tx.send(st.lock().unwrap().clone()).ok();
        });
        (
            DaemonCtrl {
                tx,
            },
            state_rx,
        )
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
        // Drain pending state updates
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
                let mut current = models
                    .iter()
                    .position(|m| *m == self.state.model)
                    .unwrap_or(0);
                egui::ComboBox::from_id_salt("model")
                    .selected_text(&self.state.model)
                    .show_ui(ui, |ui| {
                        for (i, m) in models.iter().enumerate() {
                            let selected = current == i;
                            if ui.selectable_label(selected, *m).clicked() {
                                self.ctrl.send(format!("model {m}"));
                                current = i;
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Tube:");
                let tubes = [
                    "12AX7", "12AU7", "12AT7", "6DJ8", "6L6GC", "6550", "EL34", "KT88",
                ];
                let mut current = tubes
                    .iter()
                    .position(|t| *t == self.state.tube)
                    .unwrap_or(0);
                egui::ComboBox::from_id_salt("tube")
                    .selected_text(&self.state.tube)
                    .show_ui(ui, |ui| {
                        for (i, t) in tubes.iter().enumerate() {
                            let selected = current == i;
                            if ui.selectable_label(selected, *t).clicked() {
                                self.ctrl.send(format!("tube {t}"));
                                current = i;
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

            let bypass_text = if self.state.bypass {
                "Bypass: ON"
            } else {
                "Bypass: OFF"
            };
            if ui
                .button(if self.state.bypass {
                    egui::RichText::new(bypass_text).color(egui::Color32::YELLOW)
                } else {
                    egui::RichText::new(bypass_text).color(egui::Color32::GREEN)
                })
                .clicked()
            {
                let new = if self.state.bypass { "off" } else { "on" };
                self.ctrl.send(format!("bypass {new}"));
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

        // Continuous repaint for status updates
        if self.last_poll.elapsed() > Duration::from_millis(200) {
            ctx.request_repaint();
            self.last_poll = Instant::now();
        }
    }
}
