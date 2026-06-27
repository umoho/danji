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
mod params;
mod socket;

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use hound::{WavSpec, WavWriter};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use engine::{blackhole_device, output_device, run_engine};
use params::{MainCommand, SharedParams};

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

/// 监听系统默认输出设备变化，检测到切换时通知 engine 重建输出流。
///
/// Polls the CoreAudio system default output device every 200 ms. When a change
/// is detected, sends a `SwitchOutput` command to the engine.
#[cfg(target_os = "macos")]
fn monitor_default_output(host: cpal::Host, cmd_tx: mpsc::Sender<MainCommand>) {
    use coreaudio_sys::{
        kAudioHardwarePropertyDefaultOutputDevice, kAudioObjectPropertyScopeGlobal,
        kAudioObjectSystemObject, AudioObjectGetPropertyData, AudioObjectID,
        AudioObjectPropertyAddress,
    };
    use std::mem;

    /// 从 CoreAudio 获取当前系统默认输出设备 ID。
    ///
    /// Get the current system default output device ID from CoreAudio.
    /// Returns 0 on failure.
    unsafe fn get_default_output_id() -> AudioObjectID {
        let address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultOutputDevice,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: 0,
        };
        let mut device_id: AudioObjectID = 0;
        let mut data_size = mem::size_of::<AudioObjectID>() as u32;
        let status = AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &address,
            0,
            std::ptr::null(),
            &mut data_size,
            &mut device_id as *mut _ as *mut std::ffi::c_void,
        );
        if status == 0 {
            device_id
        } else {
            0
        }
    }

    let mut last_id = unsafe { get_default_output_id() };
    log::info!("Output device monitor started (polling)");

    loop {
        thread::sleep(Duration::from_millis(200));

        let current_id = unsafe { get_default_output_id() };
        if current_id != 0 && current_id != last_id {
            last_id = current_id;
            if let Some(new_device) = host.default_output_device() {
                let name = new_device
                    .description()
                    .map(|d| d.name().to_string())
                    .unwrap_or_default();
                log::info!("System default output changed to: {name}");
                let _ = cmd_tx.send(MainCommand::SwitchOutput { device: new_device });
            }
        }
    }
}

/// 非 macOS 平台的空实现。
///
/// No-op implementation for non-macOS platforms.
#[cfg(not(target_os = "macos"))]
fn monitor_default_output(_host: cpal::Host, _cmd_tx: mpsc::Sender<MainCommand>) {}

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
        monitor_default_output(host_for_monitor, cmd_tx);
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
