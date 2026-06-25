use danji::{NodeId, SimConfig, Simulator, TriodeParams};

fn main() -> Result<(), danji::DanjiError> {
    let sr = 44100u32;
    env_logger::init();

    // SE power stage: 12AU7 driver → OPT (ideal) → 8Ω speaker
    // OPT primary: 100Ω DCR + 10H magnetizing inductance
    // Reflected load 5kΩ = N² × 8Ω → N = 25:1
    let mut cfg = SimConfig::new(sr, 5);
    let (gnd, grid, cathode, plate, bplus) =
        (NodeId(0), NodeId(1), NodeId(2), NodeId(3), NodeId(4));

    cfg.add_resistor(cathode, gnd, 1_000.0)
        .add_resistor(grid, gnd, 470_000.0)
        .add_resistor(bplus, gnd, 1e6)
        .add_resistor(plate, bplus, 100.0) // OPT primary DCR
        .add_inductor(plate, bplus, 10.0) // OPT primary inductance
        .add_triode(plate, grid, cathode, 0)
        .input(grid)
        .output(plate)
        .bplus(bplus, 350.0);

    let mut sim = Simulator::new(cfg, vec![TriodeParams::new_12au7()], vec![], vec![]);

    // B+ ramp warmup
    for i in 0..5000 {
        sim.set_bplus(350.0 * (i as f64) / 5000.0);
        sim.process_sample(0.0)?;
    }
    sim.set_bplus(350.0);
    for _ in 0..5000 {
        sim.process_sample(0.0)?;
    }

    let num_samples = sr as usize;
    let amplitude = 0.5;
    let turns_ratio = (5000.0_f64 / 8.0).sqrt();
    let input: Vec<f32> = (0..num_samples)
        .map(|i| {
            (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin() as f32 * amplitude
        })
        .collect();
    let mut output_plate = vec![0.0f32; num_samples];
    sim.process_buffer(&input, &mut output_plate)?;

    let settle = 500;
    let steady: Vec<f32> = output_plate.iter().skip(settle).copied().collect();
    let dc: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
    let ac_rms: f64 = steady
        .iter()
        .map(|x| ((x - dc) * (x - dc)) as f64)
        .sum::<f64>()
        / steady.len() as f64;
    let ac_rms_speaker = (ac_rms.sqrt() / turns_ratio) as f32;

    println!("=== Single-Ended Power Stage (12AU7 + OPT) ===");
    println!("B+: 350V, OPT: 100Ω DCR + 10H, N={:.0}:1", turns_ratio);
    println!("Input: {:.0} mV peak, 1 kHz", amplitude * 1000.0);
    println!("Plate DC: {:.0} V", dc);
    println!("Speaker RMS: {:.1} V", ac_rms_speaker);
    println!(
        "Output power: {:.1} mW",
        ac_rms_speaker * ac_rms_speaker / 8.0 * 1000.0
    );

    Ok(())
}
