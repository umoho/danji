use danji::{single_triode_config, Simulator, TriodeParams};

fn stage_gain(sim: &mut Simulator, freq: f64, sr: u32, amplitude: f32) -> f64 {
    let n = sr as usize;
    let input: Vec<f32> = (0..n)
        .map(|i| {
            (2.0 * std::f64::consts::PI * freq * i as f64 / sr as f64).sin() as f32 * amplitude
        })
        .collect();
    let mut out = vec![0.0f32; n];
    sim.process_buffer(&input, &mut out).unwrap();
    let dc: f32 = out.iter().sum::<f32>() / out.len() as f32;
    let rms: f64 = out
        .iter()
        .map(|x| ((x - dc) * (x - dc)) as f64)
        .sum::<f64>()
        / n as f64;
    20.0 * (rms.sqrt() / (amplitude as f64 / 2.0_f64.sqrt())).log10()
}

fn main() -> Result<(), danji::DanjiError> {
    let sr = 44100u32;

    // standard stage
    let mut std_amp = Simulator::new(
        single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0),
        vec![TriodeParams::new_12ax7()],
        vec![],
    );
    for _ in 0..2000 {
        std_amp.process_sample(0.0)?;
    }

    // stage without cathode bypass cap
    let mut nc_amp = Simulator::new(
        single_triode_config(sr, 100_000.0, 1_500.0, 0.0, 1_000_000.0, 300.0),
        vec![TriodeParams::new_12ax7()],
        vec![],
    );
    for _ in 0..2000 {
        nc_amp.process_sample(0.0)?;
    }

    // stage with small coupling cap (simulated via smaller cathode cap)
    let mut hf_amp = Simulator::new(
        single_triode_config(sr, 100_000.0, 1_500.0, 0.22e-6, 1_000_000.0, 300.0),
        vec![TriodeParams::new_12ax7()],
        vec![],
    );
    for _ in 0..2000 {
        hf_amp.process_sample(0.0)?;
    }

    println!("=== 12AX7 Frequency Response ===");
    println!(
        "{:>8} {:>10} {:>15} {:>10}",
        "Freq", "std(dB)", "no-bypass(dB)", "small-cap(dB)"
    );

    for freq in &[
        20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0,
    ] {
        let g_std = stage_gain(&mut std_amp, *freq, sr, 0.01);
        let g_nc = stage_gain(&mut nc_amp, *freq, sr, 0.01);
        let g_hf = stage_gain(&mut hf_amp, *freq, sr, 0.01);
        println!("{:8.0} {:>+.1} {:>+14.1} {:>+9.1}", freq, g_std, g_nc, g_hf);
    }

    Ok(())
}
