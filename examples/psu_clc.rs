use danji::{SimConfig, Simulator, DiodeParams, NodeId};

fn crc_config(sr: u32) -> SimConfig {
    let mut cfg = SimConfig::new(sr, 4);
    let (gnd, ac, b1, bplus) = (NodeId(0), NodeId(1), NodeId(2), NodeId(3));
    cfg.add_diode(ac, b1, 0)
       .add_capacitor(b1, gnd, 47e-6)
       .add_resistor(b1, bplus, 100.0)
       .add_capacitor(bplus, gnd, 47e-6)
       .add_resistor(bplus, gnd, 220e3)
       .input(ac).output(bplus);
    cfg
}

fn run_psu(sim: &mut Simulator, sr: u32, num_samples: usize, name: &str) -> Result<(), danji::DanjiError> {
    let mut out = vec![0.0f32; num_samples];
    let mut inp = vec![0.0f32; num_samples];
    for (i, s) in inp.iter_mut().enumerate() {
        let t = i as f64 / sr as f64;
        *s = (300.0 * (2.0 * std::f64::consts::PI * 60.0 * t).sin().abs()) as f32;
    }
    sim.process_buffer(&inp, &mut out)?;

    let steady: Vec<f32> = out.iter().skip(sr as usize).copied().collect();
    let dc: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
    let min_v = steady.iter().fold(f32::MAX, |a, &b| a.min(b));
    let max_v = steady.iter().fold(f32::MIN, |a, &b| a.max(b));
    let ripple = max_v - min_v;

    println!("{}: B+={:.0}V  ripple={:.3}Vp-p  load={:.2}mA",
        name, dc, ripple, dc / 220e3_f32 * 1000.0);
    Ok(())
}

fn main() -> Result<(), danji::DanjiError> {
    let sr = 44100u32;
    let n = sr as usize * 2;

    run_psu(&mut Simulator::new(crc_config(sr), vec![], vec![DiodeParams::new_5ar4()]), sr, n, "CRC (R=100Ω)")?;

    // CLC with choke: inductor model needs trapezoidal integration for stability
    // (BE inductor can diverge with high L/low R combinations)
    eprintln!("CLC skipped: needs trapezoidal inductor integration");

    Ok(())
}
