use danji::{single_triode_config, Simulator, TriodeParams};

fn main() -> Result<(), danji::DanjiError> {
    env_logger::init();

    let config = single_triode_config(44100, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0);
    let params = vec![TriodeParams::new_12ax7()];
    let mut sim = Simulator::new(config, params, vec![], vec![]);

    let duration_secs = 1.0;
    let num_samples = (44100.0 * duration_secs) as usize;
    let freq = 1000.0;
    let amplitude = 0.01;

    let input: Vec<f32> = (0..num_samples)
        .map(|i| (2.0 * std::f64::consts::PI * freq * i as f64 / 44100.0).sin() as f32 * amplitude)
        .collect();

    let mut output = vec![0.0f32; num_samples];
    sim.process_buffer(&input, &mut output)?;

    let settle = 10000;
    let steady: Vec<f32> = output.iter().skip(settle).copied().collect();
    let dc_offset: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
    let ac_max = steady
        .iter()
        .map(|x| (x - dc_offset).abs())
        .fold(0.0f32, f32::max);
    let ac_rms = (steady
        .iter()
        .map(|x| ((x - dc_offset) * (x - dc_offset)) as f64)
        .sum::<f64>()
        / steady.len() as f64)
        .sqrt();

    println!("Input amplitude: {:.4} V", amplitude);
    println!("DC offset (Vout): {:.1} V", dc_offset);
    println!("AC peak: {:.3} V", ac_max);
    println!("AC RMS: {:.3} V", ac_rms);
    println!(
        "Gain: {:.0}x ({:.1} dB)",
        ac_max / amplitude,
        20.0 * (ac_max / amplitude).log10()
    );
    println!("Sample count: {}", sim.sample_count());

    Ok(())
}
