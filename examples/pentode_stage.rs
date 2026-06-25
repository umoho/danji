use danji::{NodeId, PentodeParams, SimConfig, Simulator};

fn main() -> Result<(), danji::DanjiError> {
    let sr = 44100u32;
    env_logger::init();

    // EL84 pentode power stage, self-biased, no driver
    let mut cfg = SimConfig::new(sr, 6);
    let (gnd, g, k, p, s, b) = (
        NodeId(0),
        NodeId(1),
        NodeId(2),
        NodeId(3),
        NodeId(4),
        NodeId(5),
    );
    cfg.add_resistor(k, gnd, 150.0)
        .add_resistor(g, gnd, 470_000.0)
        .add_resistor(s, b, 1_000.0)
        .add_resistor(p, b, 100.0)
        .add_inductor(p, b, 10.0)
        .add_resistor(b, gnd, 1e6)
        .add_pentode(p, g, k, s, 0)
        .input(g)
        .output(p)
        .bplus(b, 300.0);

    let mut pow = Simulator::new(cfg, vec![], vec![PentodeParams::new_el84()], vec![]);

    // B+ ramp warmup
    for i in 0..5000 {
        pow.set_bplus(300.0 * (i as f64) / 5000.0);
        pow.process_sample(0.0)?;
    }
    pow.set_bplus(300.0);
    for _ in 0..5000 {
        pow.process_sample(0.0)?;
    }

    eprintln!(
        "DC: Vp={:.0} Vk={:.1} Vs={:.0}",
        pow.node_voltage(p),
        pow.node_voltage(k),
        pow.node_voltage(s)
    );

    // Test with sine input
    let n = sr as usize;
    let input: Vec<f32> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin() as f32 * 0.5)
        .collect();
    let mut output = vec![0.0f32; n];
    pow.process_buffer(&input, &mut output)?;

    let steady: Vec<f32> = output.iter().skip(500).copied().collect();
    let dc: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
    let ac_rms: f64 = steady
        .iter()
        .map(|x| ((x - dc) * (x - dc)) as f64)
        .sum::<f64>()
        / steady.len() as f64;
    let turns = (5000.0_f64 / 8.0).sqrt();
    let spk_rms = (ac_rms.sqrt() / turns) as f32;
    let pwr = spk_rms * spk_rms / 8.0 * 1000.0;

    println!("=== EL84 Pentode Power Stage ===");
    println!("B+: 300V, Screen: 1kΩ, Rk: 150Ω, OPT: 100Ω+10H");
    println!("Input: 0.5V peak, 1kHz");
    println!("Vp DC: {:.0} V", dc);
    println!("Speaker: {:.3} V RMS, {:.1} mW", spk_rms, pwr);

    Ok(())
}
