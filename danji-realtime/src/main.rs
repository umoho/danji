use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use danji::{single_triode_config, Simulator, TriodeParams};
use hound::{WavSpec, WavWriter};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

const SOCKET_PATH: &str = "/tmp/danji.sock";

#[derive(Parser)]
#[command(name = "danji-realtime", about = "Real-time tube amplifier daemon")]
struct Args {
    /// Run in capture mode instead of daemon mode
    #[arg(long, value_name = "PATH")]
    capture: Option<String>,

    /// Duration in seconds for capture mode
    #[arg(long, default_value = "3.0")]
    duration: f64,
}

// ── Shared runtime parameters (hot-swappable, read by audio callback) ──

struct SharedParams {
    bypass: AtomicBool,
    gain: AtomicU32,
    volume: AtomicU32,
    bplus: AtomicU32,
    tube: RwLock<String>,
    model: RwLock<String>,
}

impl SharedParams {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            bypass: AtomicBool::new(false),
            gain: AtomicU32::new(f32::to_bits(10.0_f64.powf(-38.0 / 20.0) as f32)),
            volume: AtomicU32::new(f32::to_bits(10.0_f64.powf(-12.0 / 20.0) as f32)),
            bplus: AtomicU32::new(f32::to_bits(300.0)),
            tube: RwLock::new("12AX7".into()),
            model: RwLock::new("single".into()),
        })
    }

    fn gain_linear(&self) -> f32 {
        f32::from_bits(self.gain.load(Ordering::Relaxed))
    }
    fn volume_linear(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }
    fn bypass_enabled(&self) -> bool {
        self.bypass.load(Ordering::Relaxed)
    }
    fn bplus_voltage(&self) -> f64 {
        f32::from_bits(self.bplus.load(Ordering::Relaxed)) as f64
    }

    fn set_gain_db(&self, db: f64) {
        self.gain.store(
            f32::to_bits(10.0_f64.powf(db / 20.0) as f32),
            Ordering::Relaxed,
        );
    }
    fn set_volume_db(&self, db: f64) {
        self.volume.store(
            f32::to_bits(10.0_f64.powf(db / 20.0) as f32),
            Ordering::Relaxed,
        );
    }
    fn set_bplus(&self, v: f64) {
        self.bplus.store(f32::to_bits(v as f32), Ordering::Relaxed);
    }
}

// ── Commands that require main-thread action ──

enum MainCommand {
    SetModel {
        model: String,
        resp: mpsc::Sender<String>,
    },
    SetTube {
        tube: String,
        resp: mpsc::Sender<String>,
    },
    Stop,
}

// ── DC blocker ──

struct DcBlocker {
    x_prev: f32,
    y_prev: f32,
    alpha: f32,
}

impl DcBlocker {
    fn new(sample_rate: u32) -> Self {
        Self {
            x_prev: 0.0,
            y_prev: 0.0,
            alpha: (-2.0 * std::f32::consts::PI * 10.0 / sample_rate as f32).exp(),
        }
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x_prev + self.alpha * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }
}

// ── Device helpers ──

fn find_device<F>(host: &cpal::Host, predicate: F) -> Option<cpal::Device>
where
    F: Fn(&cpal::Device) -> bool,
{
    host.devices().ok()?.into_iter().find(predicate)
}

fn blackhole_device(host: &cpal::Host) -> Result<cpal::Device, String> {
    find_device(host, |d| {
        d.description()
            .map(|desc| desc.name().contains("BlackHole"))
            .unwrap_or(false)
    })
    .ok_or_else(|| "BlackHole device not found".into())
}

fn output_device(host: &cpal::Host) -> Result<cpal::Device, String> {
    find_device(host, |d| {
        d.description()
            .map(|desc| {
                !desc.name().contains("BlackHole")
                    && !desc.name().contains("多输出")
                    && !desc.name().contains("Aggregate")
            })
            .unwrap_or(false)
            && d.supported_output_configs()
                .ok()
                .is_some_and(|mut c| c.next().is_some())
    })
    .ok_or_else(|| "No physical output device found".into())
}

fn tube_params(name: &str) -> Result<Vec<TriodeParams>, String> {
    Ok(vec![match name {
        "12AX7" => TriodeParams::new_12ax7(),
        "12AU7" => TriodeParams::new_12au7(),
        "12AT7" => TriodeParams::new_12at7(),
        "6DJ8" => TriodeParams::new_6dj8(),
        "6L6GC" => TriodeParams::new_6l6gc(),
        "6550" => TriodeParams::new_6550(),
        "EL34" => TriodeParams::new_el34(),
        "KT88" => TriodeParams::new_kt88(),
        _ => return Err(format!("unknown tube: {name}")),
    }])
}

// ── Audio engine: build model, run loop ──

struct Engine {
    sim_l: Simulator,
    sim_r: Simulator,
    dc_l: DcBlocker,
    dc_r: DcBlocker,
}

fn build_engine(
    sample_rate: u32,
    params: &SharedParams,
) -> Result<Engine, Box<dyn std::error::Error>> {
    let tube = params.tube.read().unwrap().clone();
    let tube_p = tube_params(&tube)?;
    let bplus = params.bplus_voltage();
    let mut sim_l = Simulator::new(
        single_triode_config(sample_rate, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, bplus),
        tube_p.clone(),
        vec![],
        vec![],
    );
    let mut sim_r = Simulator::new(
        single_triode_config(sample_rate, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, bplus),
        tube_p,
        vec![],
        vec![],
    );
    for _ in 0..3000 {
        sim_l.process_sample(0.0).ok();
        sim_r.process_sample(0.0).ok();
    }
    Ok(Engine {
        sim_l,
        sim_r,
        dc_l: DcBlocker::new(sample_rate),
        dc_r: DcBlocker::new(sample_rate),
    })
}

fn run_engine(
    blackhole: &cpal::Device,
    output: &cpal::Device,
    input_cfg: &StreamConfig,
    output_cfg: &StreamConfig,
    params: Arc<SharedParams>,
    cmd_rx: mpsc::Receiver<MainCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sr = input_cfg.sample_rate;

    loop {
        let engine = Arc::new(std::sync::Mutex::new(build_engine(sr, &params)?));
        let (tx, rx) = mpsc::sync_channel::<f32>(65536);

        let p = params.clone();
        let eng = engine.clone();
        let input_stream = blackhole.build_input_stream(
            input_cfg,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let vol = p.volume_linear();
                if p.bypass_enabled() {
                    for &s in data {
                        if tx.try_send(s * vol).is_err() {
                            break;
                        }
                    }
                    return;
                }
                let mut e = eng.lock().unwrap();
                for frame in data.chunks(2) {
                    let l_in = frame[0];
                    let r_in = frame.get(1).copied().unwrap_or(l_in);
                    let l_raw = e.sim_l.process_sample(l_in).unwrap_or(0.0);
                    let r_raw = e.sim_r.process_sample(r_in).unwrap_or(0.0);
                    let l_out = (e.dc_l.process(l_raw) * vol).clamp(-1.0, 1.0);
                    let r_out = (e.dc_r.process(r_raw) * vol).clamp(-1.0, 1.0);
                    if tx.try_send(l_out).is_err() || tx.try_send(r_out).is_err() {
                        break;
                    }
                }
            },
            move |err| log::error!("input error: {err}"),
            None,
        )?;

        let output_stream = output.build_output_stream(
            output_cfg,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    *sample = rx.try_recv().unwrap_or(0.0);
                }
            },
            move |err| log::error!("output error: {err}"),
            None,
        )?;

        input_stream.play()?;
        output_stream.play()?;
        log::info!("Engine started");

        loop {
            match cmd_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(MainCommand::SetModel { model, resp }) => {
                    drop(input_stream);
                    drop(output_stream);
                    *params.model.write().unwrap() = model;
                    let msg = match build_engine(sr, &params) {
                        Ok(_) => "OK".into(),
                        Err(e) => format!("ERROR {e}"),
                    };
                    let _ = resp.send(msg);
                    break;
                }
                Ok(MainCommand::SetTube { tube, resp }) => {
                    let msg = match tube_params(&tube) {
                        Ok(tp) => {
                            let bplus = params.bplus_voltage();
                            let cfg = single_triode_config(
                                sr,
                                100_000.0,
                                1_500.0,
                                22e-6,
                                1_000_000.0,
                                bplus,
                            );
                            let mut new_l = Simulator::new(cfg.clone(), tp.clone(), vec![], vec![]);
                            let mut new_r = Simulator::new(cfg, tp, vec![], vec![]);
                            for _ in 0..3000 {
                                new_l.process_sample(0.0).ok();
                            }
                            for _ in 0..3000 {
                                new_r.process_sample(0.0).ok();
                            }
                            let mut e = engine.lock().unwrap();
                            e.sim_l = new_l;
                            e.sim_r = new_r;
                            e.dc_l = DcBlocker::new(sr);
                            e.dc_r = DcBlocker::new(sr);
                            *params.tube.write().unwrap() = tube;
                            "OK".into()
                        }
                        Err(e) => format!("ERROR {e}"),
                    };
                    let _ = resp.send(msg);
                }
                Ok(MainCommand::Stop) => {
                    drop(input_stream);
                    drop(output_stream);
                    return Ok(());
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return Ok(()),
            }
        }
    }
}

// ── Socket server ──

fn socket_server(
    params: Arc<SharedParams>,
    cmd_tx: mpsc::Sender<MainCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH)?;
    log::info!("Listening on {SOCKET_PATH}");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                handle_client(s, &params, &cmd_tx);
            }
            Err(e) => log::error!("accept error: {e}"),
        }
    }
    Ok(())
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
                params.bypass.store(true, Ordering::Relaxed);
                Ok("OK".into())
            }
            "off" => {
                params.bypass.store(false, Ordering::Relaxed);
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
                if params.bypass.load(Ordering::Relaxed) {
                    "on"
                } else {
                    "off"
                },
                (params.volume_linear() as f64).max(1e-9).log10() * 20.0,
                (params.gain_linear() as f64).max(1e-9).log10() * 20.0,
                params.bplus_voltage(),
                tube,
                model,
            ))
        }
        "stop" => {
            cmd_tx.send(MainCommand::Stop).ok();
            Ok("OK".into())
        }
        _ => Err(format!("unknown command: {cmd}")),
    }
}

// ── Capture mode ──

fn capture_to_file(path: &str, duration_secs: f64) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let blackhole = blackhole_device(&host)?;
    let config: StreamConfig = blackhole
        .default_input_config()
        .map_err(|_| "BlackHole has no input config")?
        .into();
    log::info!(
        "BlackHole: {} Hz, {} ch",
        config.sample_rate,
        config.channels
    );

    let spec = WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let writer = WavWriter::create(path, spec)?;
    let writer = Arc::new(std::sync::Mutex::new(Some(writer)));
    let w = writer.clone();
    let _stream = blackhole.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if let Some(ref mut w) = *w.lock().unwrap() {
                for &s in data {
                    w.write_sample((s * 32767.0).clamp(-32768.0, 32767.0) as i16)
                        .ok();
                }
            }
        },
        move |err| log::error!("capture error: {err}"),
        None,
    )?;
    _stream.play()?;
    log::info!("Capturing {} s to {}...", duration_secs, path);
    thread::sleep(Duration::from_secs_f64(duration_secs));
    writer.lock().unwrap().take();
    log::info!("Capture complete.");
    Ok(())
}

// ── Daemon entry ──

fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let blackhole = blackhole_device(&host)?;
    let output = output_device(&host)?;

    let input_cfg: StreamConfig = blackhole
        .default_input_config()
        .map_err(|_| "BlackHole has no input config")?
        .into();
    let output_cfg = StreamConfig {
        sample_rate: input_cfg.sample_rate,
        channels: input_cfg.channels,
        buffer_size: cpal::BufferSize::Default,
    };

    log::info!(
        "Input:  {} ({} Hz, {} ch)",
        blackhole.description()?.name(),
        input_cfg.sample_rate,
        input_cfg.channels
    );
    log::info!(
        "Output: {} ({} Hz, {} ch)",
        output.description()?.name(),
        output_cfg.sample_rate,
        output_cfg.channels
    );

    let params = SharedParams::new();
    let (cmd_tx, cmd_rx) = mpsc::channel();

    let sp = params.clone();
    let ct = cmd_tx.clone();
    thread::spawn(move || {
        if let Err(e) = socket_server(sp, ct) {
            log::error!("socket server: {e}");
        }
    });

    run_engine(
        &blackhole,
        &output,
        &input_cfg,
        &output_cfg,
        params.clone(),
        cmd_rx,
    )?;

    let _ = std::fs::remove_file(SOCKET_PATH);
    Ok(())
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    let args = Args::parse();

    if let Some(path) = &args.capture {
        if let Err(e) = capture_to_file(path, args.duration) {
            log::error!("{e}");
            std::process::exit(1);
        }
    } else if let Err(e) = run_daemon() {
        log::error!("{e}");
        std::process::exit(1);
    }
}
