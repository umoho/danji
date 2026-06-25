use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use danji::{single_triode_config, Simulator, TriodeParams};
use hound::{WavSpec, WavWriter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "danji-realtime", about = "Real-time tube amplifier simulation")]
struct Args {
    /// Output volume in dB
    #[arg(default_value = "-12")]
    volume: f64,

    /// Bypass tube processing (direct passthrough)
    #[arg(long)]
    bypass: bool,

    /// Output 1 kHz test tone instead of processing audio
    #[arg(long)]
    test_tone: bool,

    /// Capture BlackHole input to WAV file
    #[arg(long, value_name = "PATH")]
    capture: Option<String>,

    /// Duration in seconds for capture mode
    #[arg(long, default_value = "3.0")]
    duration: f64,
}

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

fn find_device<F>(host: &cpal::Host, predicate: F) -> Option<cpal::Device>
where
    F: Fn(&cpal::Device) -> bool,
{
    host.devices().ok()?.into_iter().find(predicate)
}

fn capture_to_file(
    path: &str,
    duration_secs: f64,
    volume_db: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    let blackhole = find_device(&host, |d| {
        d.description()
            .map(|desc| desc.name().contains("BlackHole"))
            .unwrap_or(false)
    })
    .ok_or("BlackHole device not found")?;

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
    let vol = 10.0_f64.powf(volume_db / 20.0) as f32;

    let writer = WavWriter::create(path, spec)?;
    let writer = Arc::new(std::sync::Mutex::new(Some(writer)));

    let w = writer.clone();
    let _stream = blackhole.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if let Some(ref mut w) = *w.lock().unwrap() {
                for &s in data {
                    let sample = (s * vol * 32767.0).clamp(-32768.0, 32767.0) as i16;
                    w.write_sample(sample).ok();
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

fn start_realtime(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    let blackhole = find_device(&host, |d| {
        d.description()
            .map(|desc| desc.name().contains("BlackHole"))
            .unwrap_or(false)
    })
    .ok_or("BlackHole device not found")?;

    let output_device = find_device(&host, |d| {
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
    .ok_or("No physical output device found")?;

    let input_config: StreamConfig = blackhole
        .default_input_config()
        .map_err(|_| "BlackHole has no input config")?
        .into();

    let output_config: StreamConfig = StreamConfig {
        sample_rate: input_config.sample_rate,
        channels: input_config.channels,
        buffer_size: cpal::BufferSize::Default,
    };

    log::info!(
        "Input:  {} ({} Hz, {} ch)",
        blackhole.description()?.name(),
        input_config.sample_rate,
        input_config.channels
    );
    log::info!(
        "Output: {} ({} Hz, {} ch)",
        output_device.description()?.name(),
        output_config.sample_rate,
        output_config.channels
    );

    let bypass = args.bypass;
    let test_tone = args.test_tone;
    let vol = 10.0_f64.powf(args.volume / 20.0) as f32;
    let running = Arc::new(AtomicBool::new(true));
    let sr = input_config.sample_rate;

    let mut sim_l = Simulator::new(
        single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0),
        vec![TriodeParams::new_12ax7()],
        vec![],
        vec![],
    );
    let mut sim_r = Simulator::new(
        single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0),
        vec![TriodeParams::new_12ax7()],
        vec![],
        vec![],
    );
    let mut dc_l = DcBlocker::new(sr);
    let mut dc_r = DcBlocker::new(sr);
    for _ in 0..3000 {
        sim_l.process_sample(0.0).ok();
        sim_r.process_sample(0.0).ok();
    }

    let (tx, rx) = mpsc::sync_channel::<f32>(65536);

    let output_stream = output_device.build_output_stream(
        &output_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            if test_tone {
                let sr = output_config.sample_rate as f32;
                let mut phase = 0.0f32;
                let delta = 2.0 * std::f32::consts::PI * 1000.0 / sr;
                for sample in data.iter_mut() {
                    *sample = (phase.sin() * vol).clamp(-1.0, 1.0);
                    phase = (phase + delta) % (2.0 * std::f32::consts::PI);
                }
                return;
            }
            for sample in data.iter_mut() {
                *sample = rx.try_recv().unwrap_or(0.0);
            }
        },
        move |err| log::error!("output error: {err}"),
        None,
    )?;

    let input_stream = blackhole.build_input_stream(
        &input_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if test_tone {
                return;
            }
            for frame in data.chunks(2) {
                let l_in = frame[0];
                let r_in = frame.get(1).copied().unwrap_or(l_in);
                let l_raw = if bypass {
                    l_in
                } else {
                    sim_l.process_sample(l_in).unwrap_or(0.0)
                };
                let r_raw = if bypass {
                    r_in
                } else {
                    sim_r.process_sample(r_in).unwrap_or(0.0)
                };
                let l_out = (dc_l.process(l_raw) * vol).clamp(-1.0, 1.0);
                let r_out = (dc_r.process(r_raw) * vol).clamp(-1.0, 1.0);
                if tx.try_send(l_out).is_err() || tx.try_send(r_out).is_err() {
                    break;
                }
            }
        },
        move |err| log::error!("input error: {err}"),
        None,
    )?;

    input_stream.play()?;
    output_stream.play()?;

    if test_tone {
        log::info!("Mode: test tone (1 kHz)");
    } else if bypass {
        log::info!("Mode: bypass");
    } else {
        log::info!("Mode: 12AX7 single stage");
    }
    log::info!("Volume: {} dB ({:.3}x)", args.volume, vol);
    log::info!("Ctrl+C to stop");

    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::Relaxed);
    })?;
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }

    drop(input_stream);
    drop(output_stream);
    Ok(())
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    let args = Args::parse();

    if let Some(path) = &args.capture {
        if let Err(e) = capture_to_file(path, args.duration, args.volume) {
            log::error!("{e}");
            std::process::exit(1);
        }
    } else if let Err(e) = start_realtime(&args) {
        log::error!("{e}");
        std::process::exit(1);
    }
}
