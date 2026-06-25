use clap::Parser;
use danji::{single_triode_config, DiodeParams, NodeId, SimConfig, Simulator, TriodeParams};
use hound::{WavReader, WavSpec, WavWriter};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "danji-cli",
    about = "Tube amplifier simulator - offline audio processor"
)]
struct Args {
    /// Input WAV file
    #[arg(short, long)]
    input: PathBuf,

    /// Output WAV file
    #[arg(short, long)]
    output: PathBuf,

    /// Amplifier model: single, two-stage, chain
    #[arg(short, long, default_value = "single")]
    model: String,

    /// Input gain (dB)
    #[arg(long, default_value = "0.0")]
    gain: f64,

    /// B+ voltage
    #[arg(long, default_value = "300.0")]
    bplus: f64,

    /// Dry/wet mix (0.0 = bypass, 1.0 = full)
    #[arg(long, default_value = "1.0")]
    mix: f32,
}

fn build_single(cfg: &SimConfig) -> Simulator {
    Simulator::new(cfg.clone(), vec![TriodeParams::new_12ax7()], vec![], vec![])
}

fn build_two_stage() -> (Simulator, Simulator) {
    let c = single_triode_config(44100, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0);
    let mut s1 = Simulator::new(c.clone(), vec![TriodeParams::new_12ax7()], vec![], vec![]);
    let mut s2 = Simulator::new(c, vec![TriodeParams::new_12ax7()], vec![], vec![]);
    for _ in 0..3000 {
        s1.process_sample(0.0).unwrap();
    }
    for _ in 0..3000 {
        s2.process_sample(0.0).unwrap();
    }
    (s1, s2)
}

fn build_chain(bplus: f64) -> (Vec<f32>, Simulator, Simulator, Simulator) {
    let sr = 44100u32;
    let n = sr as usize;
    let mut p = SimConfig::new(sr, 4);
    let (g, a, b1, bp) = (NodeId(0), NodeId(1), NodeId(2), NodeId(3));
    p.add_diode(a, b1, 0)
        .add_capacitor(b1, g, 47e-6)
        .add_resistor(b1, bp, 100.0)
        .add_capacitor(bp, g, 47e-6)
        .add_resistor(bp, g, 220e3)
        .input(a)
        .output(bp);
    let mut psu = Simulator::new(p, vec![], vec![], vec![DiodeParams::new_5ar4()]);
    let mut bp_v = vec![0.0f32; n];
    for (i, v) in bp_v.iter_mut().enumerate() {
        let t = i as f64 / sr as f64;
        let vac = (bplus * (2.0 * std::f64::consts::PI * 60.0 * t).sin().abs()) as f32;
        psu.process_sample(vac).unwrap();
        *v = psu.node_voltage(bp);
    }

    let sc = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 0.0);
    let mut s1 = Simulator::new(sc.clone(), vec![TriodeParams::new_12ax7()], vec![], vec![]);
    let mut s2 = Simulator::new(sc, vec![TriodeParams::new_12ax7()], vec![], vec![]);
    for _ in 0..3000 {
        s1.set_bplus(bplus);
        s1.process_sample(0.0).unwrap();
    }
    for _ in 0..3000 {
        s2.set_bplus(bplus);
        s2.process_sample(0.0).unwrap();
    }
    (bp_v, s1, s2, {
        let mut t = SimConfig::new(sr, 4);
        let (ti, tm, to) = (NodeId(1), NodeId(2), NodeId(3));
        t.add_resistor(ti, tm, 100_000.0)
            .add_capacitor(tm, g, 330e-12)
            .add_capacitor(ti, to, 0.022e-6)
            .add_resistor(to, g, 100_000.0)
            .input(ti)
            .output(to);
        Simulator::new(t, vec![], vec![], vec![])
    })
}

fn process(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(&args.input)?;
    let spec = reader.spec();
    if spec.sample_rate != 44100 {
        eprintln!(
            "Warning: sample rate {} Hz, 44.1kHz expected",
            spec.sample_rate
        );
    }

    let gain_linear = 10.0_f64.powf(args.gain / 20.0);
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| s.unwrap_or(0) as f32 / 32768.0)
        .collect();

    eprintln!(
        "Input: {} samples, {:.1}s",
        samples.len(),
        samples.len() as f64 / spec.sample_rate as f64
    );
    eprintln!(
        "Model: {}, B+={}V, gain={}dB, mix={}",
        args.model, args.bplus, args.gain, args.mix
    );

    let output: Vec<f32> = match args.model.as_str() {
        "single" => {
            let cfg = single_triode_config(
                spec.sample_rate,
                100_000.0,
                1_500.0,
                22e-6,
                1_000_000.0,
                args.bplus,
            );
            let mut sim = build_single(&cfg);
            samples
                .iter()
                .map(|&s| {
                    let s = s * gain_linear as f32;
                    sim.process_sample(s).unwrap_or(0.0)
                })
                .collect()
        }
        "two-stage" => {
            let (mut s1, mut s2) = build_two_stage();
            let h = 1.0 / spec.sample_rate as f64;
            let alpha = 1.0 - (-h / (1_000_000.0 * 0.022e-6)).exp();
            let mut cv = 0.0;
            samples
                .iter()
                .map(|&s| {
                    let s = s * gain_linear as f32;
                    let v1 = s1.process_sample(s).unwrap_or(0.0) as f64;
                    let ac = v1 - cv;
                    cv += alpha * (v1 - cv);
                    s2.process_sample(ac as f32).unwrap_or(0.0)
                })
                .collect()
        }
        "chain" => {
            let (bp_v, mut s1, mut s2, mut tone) = build_chain(args.bplus);
            let h = 1.0 / spec.sample_rate as f64;
            let tau = 1_000_000.0 * 0.022e-6;
            let a = 1.0 - (-h / tau).exp();
            let mut c1 = 0.0;
            let mut c2 = 0.0;
            samples
                .iter()
                .enumerate()
                .map(|(i, &s)| {
                    let s = s * gain_linear as f32;
                    let bp = bp_v[i.min(bp_v.len() - 1)] as f64;
                    s1.set_bplus(bp);
                    s2.set_bplus(bp);
                    let v1 = s1.process_sample(s).unwrap_or(0.0) as f64;
                    let a1 = v1 - c1;
                    c1 += a * (v1 - c1);
                    let vt = tone.process_sample(a1 as f32).unwrap_or(0.0) as f64;
                    let a2 = vt - c2;
                    c2 += a * (vt - c2);
                    s2.process_sample(a2 as f32).unwrap_or(0.0)
                })
                .collect()
        }
        _ => {
            eprintln!("Unknown model");
            return Ok(());
        }
    };

    let output: Vec<f32> = output
        .iter()
        .zip(samples.iter())
        .map(|(&w, &d)| w * args.mix + d * (1.0 - args.mix))
        .collect();

    // AC-couple: remove DC offset, then normalize
    let dc_offset: f32 = output.iter().sum::<f32>() / output.len() as f32;
    let ac: Vec<f32> = output.iter().map(|&s| s - dc_offset).collect();
    let max_ac = ac.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    let scale = if max_ac > 0.99 { 0.99 / max_ac } else { 1.0 };

    eprintln!(
        "DC offset: {:.0}V, AC peak: {:.1}V, scale: {:.3}",
        dc_offset, max_ac, scale
    );

    let out_spec = WavSpec {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create(&args.output, out_spec)?;
    for &s in &ac {
        writer.write_sample((s * scale * 32767.0) as i16)?;
    }
    writer.finalize()?;

    eprintln!("Done. AC peak: {:.2}V, scale: {:.3}", max_ac, scale);
    Ok(())
}

fn main() {
    let args = clap::Parser::parse();
    if let Err(e) = process(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
