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

/// 监听内置音频设备的 data source 变化，检测耳机插拔并切换 danji 输出。
///
/// Polls the built-in output device's data source every 200 ms. When the source
/// changes (e.g. internal speakers ↔ headphones), sends a `SwitchOutput` command.
#[cfg(target_os = "macos")]
fn monitor_default_output(host: cpal::Host, cmd_tx: mpsc::Sender<MainCommand>) {
    use coreaudio_sys::{
        kAudioDevicePropertyDataSource, kAudioDevicePropertyTransportType,
        kAudioDeviceTransportTypeBuiltIn, kAudioHardwarePropertyDevices,
        kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, AudioObjectGetPropertyData,
        AudioObjectGetPropertyDataSize, AudioObjectID, AudioObjectPropertyAddress,
    };
    use std::mem;

    /// 获取所有音频设备 ID 列表。
    ///
    /// Get all audio device IDs from the system.
    unsafe fn get_all_device_ids() -> Vec<AudioObjectID> {
        let address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: 0,
        };
        let mut data_size: u32 = 0;
        AudioObjectGetPropertyDataSize(
            kAudioObjectSystemObject,
            &address,
            0,
            std::ptr::null(),
            &mut data_size,
        );
        let count = data_size as usize / mem::size_of::<AudioObjectID>();
        let mut devices = vec![0u32; count];
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &address,
            0,
            std::ptr::null(),
            &mut data_size,
            devices.as_mut_ptr() as *mut std::ffi::c_void,
        );
        devices
    }

    /// 获取设备的 transport type。
    ///
    /// Get the transport type of an audio device.
    unsafe fn get_transport_type(device_id: AudioObjectID) -> u32 {
        let address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyTransportType,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: 0,
        };
        let mut transport_type: u32 = 0;
        let mut data_size = mem::size_of::<u32>() as u32;
        AudioObjectGetPropertyData(
            device_id,
            &address,
            0,
            std::ptr::null(),
            &mut data_size,
            &mut transport_type as *mut _ as *mut std::ffi::c_void,
        );
        transport_type
    }

    /// 获取设备的 data source ID。
    ///
    /// Get the current data source ID of an audio device.
    unsafe fn get_data_source(device_id: AudioObjectID) -> u32 {
        let address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyDataSource,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: 0,
        };
        let mut source_id: u32 = 0;
        let mut data_size = mem::size_of::<u32>() as u32;
        AudioObjectGetPropertyData(
            device_id,
            &address,
            0,
            std::ptr::null(),
            &mut data_size,
            &mut source_id as *mut _ as *mut std::ffi::c_void,
        );
        source_id
    }

    /// 查找内置输出设备。
    ///
    /// Find the built-in output device (e.g. MacBook speakers).
    fn find_builtin_output_device(host: &cpal::Host) -> Option<cpal::Device> {
        host.devices().ok()?.find(|d| {
            // 检查是否为输出设备且非虚拟
            d.default_output_config().is_ok()
                && d.description()
                    .map(|desc| {
                        let name = desc.name().to_string();
                        !name.contains("BlackHole")
                            && !name.contains("多输出")
                            && !name.contains("Aggregate")
                    })
                    .unwrap_or(false)
        })
    }

    // 四个字符码
    const ISPK: u32 = u32::from_be_bytes(*b"ispk"); // 内置扬声器
    const HDPN: u32 = u32::from_be_bytes(*b"hdpn"); // 耳机

    let builtin_device = match find_builtin_output_device(&host) {
        Some(d) => d,
        None => {
            log::warn!("No built-in output device found, monitor disabled");
            return;
        }
    };

    // 获取内置设备对应的 CoreAudio device ID 用于读取 data source
    let builtin_name = builtin_device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_default();
    log::info!("Output device monitor started for: {builtin_name}");

    // 通过 cpal 获取内置设备的 ID（用 description 名字匹配）
    let mut last_source = unsafe {
        // 找到内置设备的 CoreAudio ID
        let devices = get_all_device_ids();
        let mut found_id: AudioObjectID = 0;
        for &dev_id in &devices {
            if get_transport_type(dev_id) == kAudioDeviceTransportTypeBuiltIn {
                found_id = dev_id;
                break;
            }
        }
        if found_id != 0 {
            get_data_source(found_id)
        } else {
            0
        }
    };

    loop {
        thread::sleep(Duration::from_millis(200));

        let current_source = unsafe {
            let devices = get_all_device_ids();
            let mut found_id: AudioObjectID = 0;
            for &dev_id in &devices {
                if get_transport_type(dev_id) == kAudioDeviceTransportTypeBuiltIn {
                    found_id = dev_id;
                    break;
                }
            }
            if found_id != 0 {
                get_data_source(found_id)
            } else {
                0
            }
        };

        if current_source != 0 && current_source != last_source {
            last_source = current_source;

            let target_device = if current_source == HDPN {
                // 耳机插入 → 找耳机设备
                log::info!("Headphones detected, switching output");
                host.devices().ok().and_then(|devices| {
                    devices.into_iter().find(|d| {
                        d.description()
                            .map(|desc| {
                                let name = desc.name().to_string();
                                !name.contains("BlackHole")
                                    && !name.contains("多输出")
                                    && !name.contains("Aggregate")
                                    && name != builtin_name
                            })
                            .unwrap_or(false)
                    })
                })
            } else if current_source == ISPK {
                // 耳机拔出 → 切回内置扬声器
                log::info!("Headphones removed, switching to built-in speakers");
                Some(builtin_device.clone())
            } else {
                log::debug!("Unknown data source: {current_source:#x}");
                None
            };

            if let Some(device) = target_device {
                let name = device
                    .description()
                    .map(|d| d.name().to_string())
                    .unwrap_or_default();
                log::info!("Switching output to: {name}");
                let _ = cmd_tx.send(MainCommand::SwitchOutput { device });
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
