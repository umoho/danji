use danji::{single_triode_config, DiodeParams, NodeId, SimConfig, Simulator, TriodeParams};

fn main() -> Result<(), danji::DanjiError> {
    env_logger::init();
    let sr = 44100u32;
    let n = sr as usize;

    // --- power supply: 5AR4 + CRC (R=100Ω, 47µF+47µF) ---
    let mut psu_cfg = SimConfig::new(sr, 4);
    let (gnd, ac_n, b1, bplus) = (NodeId(0), NodeId(1), NodeId(2), NodeId(3));
    psu_cfg
        .add_diode(ac_n, b1, 0)
        .add_capacitor(b1, gnd, 47e-6)
        .add_resistor(b1, bplus, 100.0)
        .add_capacitor(bplus, gnd, 47e-6)
        .add_resistor(bplus, gnd, 220e3)
        .input(ac_n)
        .output(bplus);

    let mut psu = Simulator::new(psu_cfg, vec![], vec![], vec![DiodeParams::new_5ar4()]);

    let mut bp_voltage = vec![0.0f32; n];
    for (i, v) in bp_voltage.iter_mut().enumerate() {
        let t = i as f64 / sr as f64;
        psu.process_sample((300.0 * (2.0 * std::f64::consts::PI * 60.0 * t).sin().abs()) as f32)?;
        *v = psu.node_voltage(bplus);
    }
    let bp_dc: f32 = bp_voltage.iter().sum::<f32>() / bp_voltage.len() as f32;
    println!("B+ DC: {:.0} V", bp_dc);

    // --- two amplifier stages ---
    let stage_cfg = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, bp_dc as f64);
    let mut stage1 = Simulator::new(stage_cfg.clone(), vec![TriodeParams::new_12ax7()], vec![], vec![]);
    let mut stage2 = Simulator::new(stage_cfg, vec![TriodeParams::new_12ax7()], vec![], vec![]);

    // --- tone control: simple RC shelving filters ---
    let mut tone_cfg = SimConfig::new(sr, 4);
    let (t_in, t_mid, t_out) = (NodeId(1), NodeId(2), NodeId(3));
    tone_cfg
        .add_resistor(t_in, t_mid, 100_000.0)
        .add_capacitor(t_mid, gnd, 330e-12) // treble cut ~4.8kHz
        .add_capacitor(t_in, t_out, 0.022e-6) // bass cut ~72Hz
        .add_resistor(t_out, gnd, 100_000.0)
        .input(t_in)
        .output(t_out);
    let mut tone = Simulator::new(tone_cfg, vec![], vec![], vec![]);

    // --- warmup stages with average B+ ---
    for _ in 0..2000 {
        stage1.set_bplus(bp_dc as f64);
        stage1.process_sample(0.0)?;
        stage2.set_bplus(bp_dc as f64);
        stage2.process_sample(0.0)?;
    }

    // --- signal chain ---
    let h = 1.0 / sr as f64;
    let tau = 1_000_000.0 * 0.022e-6;
    let alpha = 1.0 - (-h / tau).exp();
    let mut cap1 = 0.0;
    let mut cap2 = 0.0;

    let input: Vec<f32> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin() as f32 * 0.01)
        .collect();
    let mut output = vec![0.0f32; n];

    for i in 0..n {
        let bp = bp_voltage[i] as f64;
        stage1.set_bplus(bp);
        let v1 = stage1.process_sample(input[i])? as f64;
        let ac1 = v1 - cap1;
        cap1 += alpha * (v1 - cap1);

        let vt = tone.process_sample(ac1 as f32)? as f64;

        let ac2 = vt - cap2;
        cap2 += alpha * (vt - cap2);

        stage2.set_bplus(bp);
        output[i] = stage2.process_sample(ac2 as f32)?;
    }

    let settle = 500;
    let steady: Vec<f32> = output.iter().skip(settle).copied().collect();
    let dc: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
    let ac_max = steady.iter().map(|x| (x - dc).abs()).fold(0.0f32, f32::max);
    let gain = ac_max / 0.01;

    println!("=== Complete Tube Preamplifier Signal Chain ===");
    println!("Power Supply: 5AR4 → 100Ω → 47µF → 47µF → 220kΩ");
    println!("Stages: 12AX7 × 2, common cathode, bypassed");
    println!("Tone: treble shelf ~4.8kHz, bass shelf ~72Hz");
    println!("Input: 10 mV peak, 1 kHz");
    println!("Output DC: {:.0} V", dc);
    println!("Output AC peak: {:.1} V", ac_max);
    println!(
        "Total gain: {:.0}x ({:.1} dB)",
        gain,
        20.0 * (gain as f64).log10()
    );
    println!("Samples: {}", n);

    Ok(())
}
