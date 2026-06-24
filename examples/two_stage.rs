use danji::{single_triode_config, Simulator, TriodeParams};

fn main() -> Result<(), danji::DanjiError> {
    let sr = 44100u32;

    let cfg1 = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0);
    let cfg2 = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0);

    let mut sim1 = Simulator::new(cfg1, vec![TriodeParams::new_12ax7()]);
    let mut sim2 = Simulator::new(cfg2, vec![TriodeParams::new_12ax7()]);

    // warmup stage 1
    for _ in 0..3000 {
        sim1.process_sample(0.0)?;
    }

    // warmup stage 2
    for _ in 0..3000 {
        sim2.process_sample(0.0)?;
    }

    let num_samples = 44100;
    let input: Vec<f32> = (0..num_samples)
        .map(|i| (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin() as f32 * 0.01)
        .collect();

    let h = 1.0 / sr as f64;
    let rc_tau = 1_000_000.0 * 0.022e-6;
    let coupling_alpha = 1.0 - (-h / rc_tau).exp();
    let mut cap_voltage = 203.0;

    let mut output = vec![0.0f32; num_samples];

    for i in 0..num_samples {
        let vout1 = sim1.process_sample(input[i])? as f64;
        let v_ac = vout1 - cap_voltage;
        cap_voltage += coupling_alpha * (vout1 - cap_voltage);
        output[i] = sim2.process_sample(v_ac as f32)?;
    }

    let dc: f32 = output.iter().sum::<f32>() / output.len() as f32;
    let ac_max = output.iter().map(|x| (x - dc).abs()).fold(0.0f32, f32::max);
    let gain = ac_max / 0.01;
    println!("=== Two-stage (independent) 12AX7 Preamplifier ===");
    println!("Input: 10 mV peak, 1 kHz");
    println!("Output DC offset: {:.1} V", dc);
    println!("Output AC peak: {:.3} V", ac_max);
    println!(
        "Total gain: {:.0}x ({:.1} dB)",
        gain,
        20.0 * (gain as f64).log10()
    );

    Ok(())
}
