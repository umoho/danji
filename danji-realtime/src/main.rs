use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use danji::{single_triode_config, Simulator, TriodeParams};
use hound::{WavSpec, WavWriter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

fn analyze_wav(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap_or(0)).collect();

    let total = samples.len();
    let nonzero = samples.iter().filter(|&&s| s != 0).count();
    let max_abs = samples.iter().map(|&s| s.abs()).max().unwrap_or(0);
    let dc = samples.iter().map(|&s| s as i64).sum::<i64>() as f64 / total as f64;
    let rms = (samples.iter().map(|&s| (s as f64).powi(2)).sum::<f64>() / total as f64).sqrt();
    let zero_runs = samples.split(|&s| s == 0).filter(|s| s.len() > 100).count();

    println!("=== WAV Analysis: {} ===", path);
    println!(
        "  Format:        {} Hz, {} ch, {}-bit",
        spec.sample_rate, spec.channels, spec.bits_per_sample
    );
    println!("  Total samples: {}", total);
    println!(
        "  Non-zero:      {} ({:.1}%)",
        nonzero,
        nonzero as f64 / total as f64 * 100.0
    );
    println!(
        "  Peak:          {} ({:.4} FS)",
        max_abs,
        max_abs as f64 / 32768.0
    );
    println!("  DC offset:     {:.2}", dc);
    println!(
        "  RMS:           {:.4} ({:.1} dBFS)",
        rms / 32768.0,
        20.0 * (rms / 32768.0).log10()
    );
    println!("  Zero runs >100: {}", zero_runs);
    Ok(())
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

    eprintln!(
        "Device: {} ({} Hz, {} ch)",
        blackhole.description()?.name(),
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
        move |err| eprintln!("input error: {err}"),
        None,
    )?;

    _stream.play()?;
    eprintln!("Capturing {} seconds to {}...", duration_secs, path);

    let total_ms = (duration_secs * 1000.0) as u64;
    for _ in 0..total_ms / 100 {
        thread::sleep(Duration::from_millis(100));
    }

    writer.lock().unwrap().take();
    eprintln!("Done.");
    Ok(())
}

fn start_realtime(
    volume_db: f64,
    bypass: bool,
    test_tone: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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

    eprintln!(
        "Input:  {} ({} Hz, {} ch)",
        blackhole.description()?.name(),
        input_config.sample_rate,
        input_config.channels
    );
    eprintln!(
        "Output: {} ({} Hz, {} ch)",
        output_device.description()?.name(),
        output_config.sample_rate,
        output_config.channels
    );
    eprintln!(
        "Volume: {} dB ({:.3}x)",
        volume_db,
        10.0_f64.powf(volume_db / 20.0)
    );

    let vol = 10.0_f64.powf(volume_db / 20.0) as f32;
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
    let sample_rate = output_config.sample_rate;

    let output_stream = output_device.build_output_stream(
        &output_config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            if test_tone {
                let mut phase = 0.0f32;
                let delta = 2.0 * std::f32::consts::PI * 1000.0 / sample_rate as f32;
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
        move |err| eprintln!("output error: {err}"),
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
                let r_in = if frame.len() > 1 { frame[1] } else { l_in };
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
                if tx.try_send(l_out).is_err() {
                    break;
                }
                if tx.try_send(r_out).is_err() {
                    break;
                }
            }
        },
        move |err| eprintln!("input error: {err}"),
        None,
    )?;

    input_stream.play()?;
    output_stream.play()?;

    if test_tone {
        eprintln!("Mode:  test tone (1kHz)");
    } else if bypass {
        eprintln!("Mode:  bypass (no tube)");
    } else {
        eprintln!("Mode:  12AX7 single stage");
    }
    eprintln!("Ctrl+C to stop.");

    let r = running.clone();
    ctrlc::set_handler(move || {
        eprintln!("\nShutting down...");
        r.store(false, Ordering::SeqCst);
    })?;
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    drop(input_stream);
    drop(output_stream);
    eprintln!("Stopped.");
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut capture = false;
    let mut analyze = false;
    let mut bypass = false;
    let mut test_tone = false;
    let mut capture_path = String::from("capture.wav");
    let mut capture_duration = 3.0;
    let mut volume_db = -12.0;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--capture" => {
                capture = true;
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    capture_path = args[i].clone();
                }
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    capture_duration = args[i].parse().unwrap_or(3.0);
                }
            }
            "--analyze" | "--check" => {
                analyze = true;
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    capture_path = args[i].clone();
                }
            }
            "--test-tone" => test_tone = true,
            "--bypass" => bypass = true,
            a if a.parse::<f64>().is_ok() => volume_db = a.parse().unwrap(),
            _ => {}
        }
        i += 1;
    }

    if analyze {
        if let Err(e) = analyze_wav(&capture_path) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    } else if capture {
        if let Err(e) = capture_to_file(&capture_path, capture_duration, volume_db) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    } else {
        if let Err(e) = start_realtime(volume_db, bypass, test_tone) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
