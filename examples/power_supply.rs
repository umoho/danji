use danji::{SimConfig, Simulator, DiodeParams, NodeId};

fn power_supply_config(sr: u32) -> SimConfig {
    let mut cfg = SimConfig::new(sr, 3);
    let (gnd, ac, bplus) = (NodeId(0), NodeId(1), NodeId(2));

    cfg.add_diode(ac, bplus, 0)       // rectifier
       .add_resistor(bplus, gnd, 220e3)  // bleeder + load
       .add_capacitor(bplus, gnd, 47e-6) // reservoir cap
       .input(ac)                        // AC input
       .output(bplus);
    cfg
}

fn main() -> Result<(), danji::DanjiError> {
    env_logger::init();
    let sr = 44100u32;
    let num_samples = sr as usize;

    let cfg = power_supply_config(sr);
    let mut sim = Simulator::new(cfg, vec![], vec![DiodeParams::new_5ar4()]);

    let mut output = vec![0.0f32; num_samples];
    let mut input = vec![0.0f32; num_samples];

    for (i, sample) in input.iter_mut().enumerate() {
        let t = i as f64 / sr as f64;
        let vac = 300.0 * (2.0 * std::f64::consts::PI * 60.0 * t).sin().abs();
        *sample = vac as f32;
    }

    sim.process_buffer(&input, &mut output)?;

    let settle = sr as usize / 2;
    let steady: Vec<f32> = output.iter().skip(settle).copied().collect();

    let dc: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
    let min_v = steady.iter().fold(f32::MAX, |a, &b| a.min(b));
    let max_v = steady.iter().fold(f32::MIN, |a, &b| a.max(b));

    println!("=== Vacuum Tube Rectifier Power Supply ===");
    println!("AC input: 300V peak, 60 Hz");
    println!("Rectifier: 5AR4 (GZ34)");
    println!("Reservoir cap: 47 µF");
    println!("Load: 220 kΩ");
    println!("B+ DC output: {:.0} V", dc);
    println!("Ripple (p-p): {:.1} V", max_v - min_v);
    println!("Min: {:.0} V, Max: {:.0} V", min_v, max_v);

    Ok(())
}
