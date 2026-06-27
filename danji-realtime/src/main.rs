//! danji-realtime - 真空管放大器实时音频守护进程。
//!
//! 本程序通过 BlackHole 虚拟音频设备捕获系统音频，
//! 实时处理后输出到物理音频设备。
//!
//! ---
//!
//! danji-realtime - Vacuum tube amplifier real-time audio daemon.
//!
//! This program captures system audio through BlackHole virtual audio device,
//! processes it in real-time, and outputs to physical audio device.

mod engine;
mod monitor;
mod params;
mod socket;

use clap::Parser;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::StreamConfig;
use hound::{WavSpec, WavWriter};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use engine::{blackhole_device, output_device, run_engine};
use params::SharedParams;

#[derive(Parser)]
#[command(name = "danji-realtime", about = "Real-time tube amplifier daemon")]
struct Args {
    #[arg(long, value_name = "PATH")]
    capture: Option<String>,

    #[arg(long, default_value = "3.0")]
    duration: f64,
}

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

    // 先克隆，再分别移入两个线程
    let cmd_tx_socket = cmd_tx.clone();
    let sp = params.clone();
    thread::spawn(move || {
        socket::run_socket_server(sp, cmd_tx_socket);
    });

    // 启动输出设备监听线程
    let host_for_monitor = cpal::default_host();
    thread::spawn(move || {
        monitor::run_device_monitor(host_for_monitor, cmd_tx);
    });

    run_engine(&blackhole, &output, &input_cfg, &output_cfg, params, cmd_rx)?;

    let _ = std::fs::remove_file("/tmp/danji.sock");
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
