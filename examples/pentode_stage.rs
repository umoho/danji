use danji::{SimConfig, Simulator, PentodeParams, NodeId};

fn main() -> Result<(), danji::DanjiError> {
    env_logger::init();
    let sr = 44100u32;
    let mut cfg = SimConfig::new(sr, 6);
    let (gnd, grid, cathode, plate, screen, bplus) =
        (NodeId(0), NodeId(1), NodeId(2), NodeId(3), NodeId(4), NodeId(5));

    cfg.add_resistor(cathode, gnd, 150.0)
       .add_resistor(grid, gnd, 470_000.0)
       .add_resistor(screen, bplus, 100.0)    // screen: almost at B+
       .add_resistor(plate, bplus, 5_000.0)    // plate load
       .add_resistor(bplus, gnd, 1e6)
       .add_pentode(plate, grid, cathode, screen, 0)
       .input(grid).output(plate).bplus(bplus, 300.0);

    let mut sim = Simulator::new(cfg, vec![], vec![PentodeParams::new_el84()], vec![]);
    for _ in 0..10000 { sim.process_sample(0.0)?; }

    let num_samples = sr as usize;
    let amp = 0.1;
    let input: Vec<f32> = (0..num_samples)
        .map(|i| (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin() as f32 * amp)
        .collect();
    let mut out = vec![0.0f32; num_samples];
    sim.process_buffer(&input, &mut out)?;

    let steady: Vec<f32> = out.iter().skip(500).copied().collect();
    let dc: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
    let ac_rms: f64 = steady.iter().map(|x| ((x-dc)*(x-dc)) as f64).sum::<f64>() / steady.len() as f64;
    println!("=== EL84 Pentode ===");
    println!("Vp DC: {:.0} V", dc);
    println!("Gain: {:.0}x ({:.1} dB)", ac_rms.sqrt() / (amp as f64 / 1.414), 20.0 * (ac_rms.sqrt() / (amp as f64 / 1.414)).log10());

    Ok(())
}
