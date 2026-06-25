use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/danji.sock";

#[derive(Clone)]
pub struct AppState {
    pub connected: bool,
    pub bypass: bool,
    pub volume: f32,
    pub gain: f32,
    pub bplus: f32,
    pub tube: String,
    pub model: String,
    pub status_msg: String,
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

enum DaemonCmd {
    Set(String),
    Stop,
}

pub struct DaemonCtrl {
    tx: mpsc::Sender<DaemonCmd>,
    _state: Arc<Mutex<AppState>>,
}

impl DaemonCtrl {
    pub fn connect(state_rx: mpsc::Sender<AppState>) -> Result<Self, String> {
        let stream = UnixStream::connect(SOCKET_PATH).map_err(|e| format!("connect: {e}"))?;

        let (cmd_tx, cmd_rx) = mpsc::channel::<DaemonCmd>();
        let state = Arc::new(Mutex::new(AppState::default()));

        // Initial status query
        let mut s = stream.try_clone().unwrap();
        let mut r = BufReader::new(s.try_clone().unwrap());
        writeln!(s, "status").map_err(|e| format!("send: {e}"))?;
        let mut resp = String::new();
        r.read_line(&mut resp).map_err(|e| format!("read: {e}"))?;
        {
            let mut st = state.lock().unwrap();
            parse_status(&resp, &mut st);
            st.connected = true;
            state_rx.send(st.clone()).ok();
        }

        let st = state.clone();
        std::thread::spawn(move || {
            let mut w = s;
            loop {
                match cmd_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(DaemonCmd::Set(cmd)) => {
                        if writeln!(w, "{cmd}").is_err() {
                            break;
                        }
                        let mut resp = String::new();
                        let mut reader = BufReader::new(w.try_clone().unwrap());
                        reader.read_line(&mut resp).ok();
                        let resp = resp.trim().to_string();
                        let mut s = st.lock().unwrap();
                        s.status_msg = resp.clone();
                        if cmd == "stop" {
                            break;
                        }
                        drop(s);
                        writeln!(w, "status").ok();
                        let mut resp2 = String::new();
                        reader.read_line(&mut resp2).ok();
                        let mut s = st.lock().unwrap();
                        parse_status(&resp2, &mut s);
                        state_rx.send(s.clone()).ok();
                    }
                    Ok(DaemonCmd::Stop) => {
                        let _ = writeln!(w, "stop");
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        writeln!(w, "status").ok();
                        let mut resp = String::new();
                        let mut reader = BufReader::new(w.try_clone().unwrap());
                        reader.read_line(&mut resp).ok();
                        let mut s = st.lock().unwrap();
                        parse_status(&resp, &mut s);
                        state_rx.send(s.clone()).ok();
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Ok(Self {
            tx: cmd_tx,
            _state: state,
        })
    }

    pub fn send(&self, cmd: String) {
        let _ = self.tx.send(DaemonCmd::Set(cmd));
    }

    pub fn stop(&self) {
        let _ = self.tx.send(DaemonCmd::Stop);
    }

    pub fn from_fallback(state_rx: mpsc::Sender<AppState>) -> Self {
        let (tx, _) = mpsc::channel();
        let state = Arc::new(Mutex::new(AppState::default()));
        let st = state.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            state_rx.send(st.lock().unwrap().clone()).ok();
        });
        Self { tx, _state: state }
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
