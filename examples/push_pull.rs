use danji::{NodeId, PentodeParams, SimConfig, Simulator, TriodeParams};
use std::f64::consts::PI;

const SR: u32 = 44100;

fn test_phase_inverter() -> Result<(), danji::DanjiError> {
    // 12AX7 long-tail pair phase inverter
    let num_nodes = 7;
    let (g, v1a_g, cath, v1a_p, v1b_g, v1b_p, b) = (
        NodeId(0), NodeId(1), NodeId(2), NodeId(3), NodeId(4), NodeId(5), NodeId(6),
    );

    let mut cfg = SimConfig::new(SR, num_nodes);
    cfg.add_resistor(v1b_g, g, 470_000.0)
        .add_resistor(cath, g, 47_000.0)
        .add_resistor(v1a_p, b, 100_000.0)
        .add_resistor(v1b_p, b, 100_000.0)
        .add_resistor(b, g, 1_000_000.0)
        .add_triode(v1a_p, v1a_g, cath, 0)
        .add_triode(v1b_p, v1b_g, cath, 0)
        .input(v1a_g)
        .output(v1b_p)
        .bplus(b, 250.0);

    let mut sim = Simulator::new(cfg, vec![TriodeParams::new_12ax7()], vec![], vec![]);

    for i in 0..5000 {
        sim.set_bplus(250.0 * (i as f64) / 5000.0);
        sim.process_sample(0.0)?;
    }
    sim.set_bplus(250.0);
    for _ in 0..5000 {
        sim.process_sample(0.0)?;
    }

    println!("=== Phase Inverter DC Bias ===");
    println!(
        "V1a: Vg={:.2} Vk={:.2} Vp={:.2}",
        sim.node_voltage(v1a_g),
        sim.node_voltage(cath),
        sim.node_voltage(v1a_p)
    );
    println!(
        "V1b: Vg={:.2} Vk={:.2} Vp={:.2}",
        sim.node_voltage(v1b_g),
        sim.node_voltage(cath),
        sim.node_voltage(v1b_p)
    );

    let n = (SR as f64 * 0.1) as usize;
    let mut vpa = Vec::new();
    let mut vpb = Vec::new();

    for i in 0..n {
        let t = i as f64 / SR as f64;
        let vin = (2.0 * PI * 1000.0 * t).sin() as f32 * 0.5;
        sim.process_sample(vin)?;
        vpa.push(sim.node_voltage(v1a_p));
        vpb.push(sim.node_voltage(v1b_p));
    }

    let dc_a: f32 = vpa.iter().sum::<f32>() / vpa.len() as f32;
    let dc_b: f32 = vpb.iter().sum::<f32>() / vpb.len() as f32;
    let ac_a: f32 = vpa
        .iter()
        .map(|x| (x - dc_a).abs())
        .fold(0.0f32, f32::max);
    let ac_b: f32 = vpb
        .iter()
        .map(|x| (x - dc_b).abs())
        .fold(0.0f32, f32::max);

    let mut sum = 0.0f64;
    for (a, b) in vpa.iter().zip(vpb.iter()) {
        sum += ((*a - dc_a) * (*b - dc_b)) as f64;
    }

    println!("=== Phase Inverter AC (1kHz, 0.5Vpk input) ===");
    println!("V1a plate: DC={:.1}V  ACpk={:.3}V", dc_a, ac_a);
    println!("V1b plate: DC={:.1}V  ACpk={:.3}V", dc_b, ac_b);
    println!(
        "Phase: {}",
        if sum < 0.0 { "INVERTED" } else { "SAME" }
    );

    Ok(())
}

fn test_full_push_pull() -> Result<(), danji::DanjiError> {
    // Full push-pull: 12AX7 phase inverter → EL84×2 → OPT → 8Ω
    //
    // Nodes:
    // 0:  gnd
    // 1:  v1a_g    (12AX7 V1a grid, input)
    // 2:  cath     (12AX7 shared cathode)
    // 3:  v1a_p    (12AX7 V1a plate)
    // 4:  v1b_g    (12AX7 V1b grid)
    // 5:  v1b_p    (12AX7 V1b plate)
    // 6:  el84a_g  (EL84 upper grid)
    // 7:  el84a_k  (EL84 upper cathode)
    // 8:  el84a_s  (EL84 upper screen)
    // 9:  el84a_p  (EL84 upper plate)
    // 10: el84b_g  (EL84 lower grid)
    // 11: el84b_k  (EL84 lower cathode)
    // 12: el84b_s  (EL84 lower screen)
    // 13: el84b_p  (EL84 lower plate)
    // 14: ct_bp   (transformer CT + B+)
    // 15: spk      (speaker)
    let num_nodes = 16;
    let (
        g, v1a_g, cath, v1a_p, v1b_g, v1b_p, el84a_g, el84a_k, el84a_s, el84a_p,
        el84b_g, el84b_k, el84b_s, el84b_p, ct_bp, spk,
    ) = (
        NodeId(0), NodeId(1), NodeId(2), NodeId(3), NodeId(4), NodeId(5),
        NodeId(6), NodeId(7), NodeId(8), NodeId(9),
        NodeId(10), NodeId(11), NodeId(12), NodeId(13),
        NodeId(14), NodeId(15),
    );

    let mut cfg = SimConfig::new(SR, num_nodes);

    cfg.add_resistor(v1b_g, g, 470_000.0)
        .add_resistor(cath, g, 47_000.0)
        .add_resistor(v1a_p, ct_bp, 100_000.0)
        .add_resistor(v1b_p, ct_bp, 100_000.0)
        .add_resistor(ct_bp, g, 1_000_000.0)
        // Coupling caps from 12AX7 plates to EL84 grids
        .add_capacitor(v1a_p, el84a_g, 0.022e-6)
        .add_capacitor(v1b_p, el84b_g, 0.022e-6)
        .add_resistor(el84a_g, g, 470_000.0)
        .add_resistor(el84b_g, g, 470_000.0)
        .add_resistor(el84a_k, g, 150.0)
        .add_resistor(el84b_k, g, 150.0)
        .add_resistor(el84a_s, ct_bp, 1_000.0)
        .add_resistor(el84b_s, ct_bp, 1_000.0)
        // Snubber: 47Ω from each EL84 plate to B+ provides a low-impedance
        // DC path during warmup (prevents Vpk from going negative when
        // EL84 grids are at 12AX7 plate potential through the coupling caps)
        .add_resistor(el84a_p, ct_bp, 47.0)
        .add_resistor(el84b_p, ct_bp, 47.0)
        .add_coupled_inductor(el84a_p, ct_bp, spk, g, 2.5, 0.016, 0.95)
        .add_coupled_inductor(el84b_p, ct_bp, spk, g, 2.5, 0.016, 0.95)
        .add_triode(v1a_p, v1a_g, cath, 0)
        .add_triode(v1b_p, v1b_g, cath, 0)
        .add_pentode(el84a_p, el84a_g, el84a_k, el84a_s, 0)
        .add_pentode(el84b_p, el84b_g, el84b_k, el84b_s, 0)
        .input(v1a_g)
        .output(spk)
        .bplus(ct_bp, 250.0);

    let triode_params = vec![TriodeParams::new_12ax7()];
    let pentode_params = vec![PentodeParams::new_el84()];
    let mut sim = Simulator::new(cfg, triode_params, pentode_params, vec![]);

    // Phase 1: ramp B+. The 47Ω snubber keeps EL84 plate voltage from
    // going negative despite the grid being at 12AX7 plate potential.
    // The coupling caps charge as the 12AX7 plates rise.
    for i in 0..20000 {
        let frac = (i as f64) / 20000.0;
        sim.set_bplus(250.0 * frac);
        sim.process_sample(0.0)?;
    }
    sim.set_bplus(250.0);

    // Phase 2: settle. The EL84 grids discharge through 470kΩ grid leaks
    // with τ = 10ms. After 50ms (2200 samples) they're near 0V.
    for _ in 0..20000 {
        sim.process_sample(0.0)?;
    }

    // Report DC bias
    println!();
    println!("=== Full Push-Pull DC Bias ===");
    println!(
        "12AX7 V1a: Vg={:.2} Vk={:.2} Vp={:.2}",
        sim.node_voltage(v1a_g),
        sim.node_voltage(cath),
        sim.node_voltage(v1a_p)
    );
    println!(
        "12AX7 V1b: Vg={:.2} Vk={:.2} Vp={:.2}",
        sim.node_voltage(v1b_g),
        sim.node_voltage(cath),
        sim.node_voltage(v1b_p)
    );
    println!(
        "EL84a: Vg={:.2} Vk={:.2} Vs={:.2} Vp={:.2}",
        sim.node_voltage(el84a_g),
        sim.node_voltage(el84a_k),
        sim.node_voltage(el84a_s),
        sim.node_voltage(el84a_p)
    );
    println!(
        "EL84b: Vg={:.2} Vk={:.2} Vs={:.2} Vp={:.2}",
        sim.node_voltage(el84b_g),
        sim.node_voltage(el84b_k),
        sim.node_voltage(el84b_s),
        sim.node_voltage(el84b_p)
    );

    // AC test with 1kHz sine
    let n = (SR as f64 * 0.5) as usize;
    let mut output = vec![0.0f32; n];

    for i in 0..n {
        let t = i as f64 / SR as f64;
        let vin = (2.0 * PI * 1000.0 * t).sin() as f32 * 0.5;
        output[i] = sim.process_sample(vin)?;
    }

    // Analyze: skip first 100ms for settling
    let settle = (SR as f64 * 0.1) as usize;
    let steady: &[f32] = &output[settle..];

    let spk_dc: f32 = steady.iter().sum::<f32>() / steady.len() as f32;
    let spk_rms: f64 = (steady
        .iter()
        .map(|x| ((*x - spk_dc) * (*x - spk_dc)) as f64)
        .sum::<f64>()
        / steady.len() as f64)
        .sqrt();
    let pwr_mw = spk_rms * spk_rms / 8.0 * 1000.0;

    println!();
    println!("=== Full Push-Pull AC (1kHz, 0.5Vpk input) ===");
    println!(
        "Speaker: {:.1} mV RMS, {:.1} mW",
        spk_rms * 1000.0,
        pwr_mw
    );
    println!("Output DC offset: {:.2} V", spk_dc);

    Ok(())
}

fn main() {
    env_logger::init();

    println!("=== Danji Push-Pull Power Stage ===");
    println!();

    if let Err(e) = test_phase_inverter() {
        eprintln!("Phase inverter test FAILED: {}", e);
        return;
    }

    println!();
    println!("Full push-pull: WIP (NaN convergence issue with BE coupled inductor");
    println!("  during coupling cap warmup transient. See devlog.)");
}
