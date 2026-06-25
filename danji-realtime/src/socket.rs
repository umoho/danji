use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc;
use std::sync::Arc;

use crate::params::{MainCommand, SharedParams};

const SOCKET_PATH: &str = "/tmp/danji.sock";

pub fn run_socket_server(params: Arc<SharedParams>, cmd_tx: mpsc::Sender<MainCommand>) {
    let _ = std::fs::remove_file(SOCKET_PATH);
    match UnixListener::bind(SOCKET_PATH) {
        Ok(listener) => {
            log::info!("Listening on {SOCKET_PATH}");
            for stream in listener.incoming() {
                match stream {
                    Ok(s) => handle_client(s, &params, &cmd_tx),
                    Err(e) => log::error!("accept error: {e}"),
                }
            }
        }
        Err(e) => log::error!("socket bind: {e}"),
    }
}

fn handle_client(stream: UnixStream, params: &SharedParams, cmd_tx: &mpsc::Sender<MainCommand>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let line = line.trim();
    let resp = match parse_command(line, params, cmd_tx) {
        Ok(msg) => format!("{msg}\n"),
        Err(msg) => format!("ERROR {msg}\n"),
    };
    let _ = writer.write_all(resp.as_bytes());
}

fn parse_command(
    line: &str,
    params: &SharedParams,
    cmd_tx: &mpsc::Sender<MainCommand>,
) -> Result<String, String> {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let cmd = parts[0];
    let val = parts.get(1).copied().unwrap_or("");

    match cmd {
        "bypass" => match val {
            "on" => {
                params
                    .bypass
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                Ok("OK".into())
            }
            "off" => {
                params
                    .bypass
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                Ok("OK".into())
            }
            _ => Err("usage: bypass on|off".into()),
        },
        "volume" => {
            let db: f64 = val.parse().map_err(|_| "invalid volume".to_string())?;
            if !(-60.0..=12.0).contains(&db) {
                return Err("volume range: -60..+12 dB".into());
            }
            params.set_volume_db(db);
            Ok("OK".into())
        }
        "gain" => {
            let db: f64 = val.parse().map_err(|_| "invalid gain".to_string())?;
            if !(-96.0..=10.0).contains(&db) {
                return Err("gain range: -96..+10 dB".into());
            }
            params.set_gain_db(db);
            Ok("OK".into())
        }
        "bplus" => {
            let v: f64 = val.parse().map_err(|_| "invalid voltage".to_string())?;
            if !(50.0..=600.0).contains(&v) {
                return Err("bplus range: 50..600 V".into());
            }
            params.set_bplus(v);
            Ok("OK".into())
        }
        "tube" => {
            let tube = val.to_string();
            let (resp, rx) = mpsc::channel();
            cmd_tx
                .send(MainCommand::SetTube { tube, resp })
                .map_err(|_| "daemon busy".to_string())?;
            Ok(rx
                .recv()
                .unwrap_or_else(|_| "ERROR daemon disconnected".into()))
        }
        "model" => {
            let model = val.to_string();
            if !["single", "two-stage", "chain"].contains(&model.as_str()) {
                return Err("model: single, two-stage, chain".into());
            }
            let (resp, rx) = mpsc::channel();
            cmd_tx
                .send(MainCommand::SetModel { model, resp })
                .map_err(|_| "daemon busy".to_string())?;
            Ok(rx
                .recv()
                .unwrap_or_else(|_| "ERROR daemon disconnected".into()))
        }
        "status" => {
            let tube = params.tube.read().unwrap();
            let model = params.model.read().unwrap();
            Ok(format!(
                "bypass={} volume={:.1} gain={:.1} bplus={:.0} tube={} model={}",
                if params.bypass.load(std::sync::atomic::Ordering::Relaxed) {
                    "on"
                } else {
                    "off"
                },
                (params.volume_linear() as f64).max(1e-9).log10() * 20.0,
                (params.gain_linear() as f64).max(1e-9).log10() * 20.0,
                params.bplus_voltage(),
                *tube,
                *model,
            ))
        }
        "stop" => {
            cmd_tx.send(MainCommand::Stop).ok();
            Ok("OK".into())
        }
        _ => Err(format!("unknown command: {cmd}")),
    }
}
