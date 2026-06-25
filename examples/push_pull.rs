use danji::{NodeId, PentodeParams, SimConfig, Simulator, TriodeParams};
use std::f64::consts::PI;

const SR: u32 = 44100;

fn test_phase_inverter() -> Result<(), danji::DanjiError> {
    let num_nodes = 7;
    let (g, v1a_g, cath, v1a_p, v1b_g, v1b_p, b) = (
        NodeId(0),
        NodeId(1),
        NodeId(2),
        NodeId(3),
        NodeId(4),
        NodeId(5),
        NodeId(6),
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
    let ac_a: f32 = vpa.iter().map(|x| (x - dc_a).abs()).fold(0.0f32, f32::max);
    let ac_b: f32 = vpb.iter().map(|x| (x - dc_b).abs()).fold(0.0f32, f32::max);

    let mut sum = 0.0f64;
    for (a, b) in vpa.iter().zip(vpb.iter()) {
        sum += ((*a - dc_a) * (*b - dc_b)) as f64;
    }

    println!("=== Phase Inverter AC (1kHz, 0.5Vpk input) ===");
    println!("V1a plate: DC={:.1}V  ACpk={:.3}V", dc_a, ac_a);
    println!("V1b plate: DC={:.1}V  ACpk={:.3}V", dc_b, ac_b);
    println!("Phase: {}", if sum < 0.0 { "INVERTED" } else { "SAME" });

    Ok(())
}

fn test_full_push_pull() -> Result<(), danji::DanjiError> {
    // Two-simulator approach:
    //   Sim1: 12AX7 long-tail pair phase inverter
    //   Sim2: EL84 push-pull output stage
    //
    // Coupling: external RC high-pass filter between stages
    // (BE capacitors cannot block DC in the MNA netlist)

    // --- Phase inverter (same as test_phase_inverter) ---
    let pi_num = 7;
    let (g, v1a_g, cath, v1a_p, v1b_g, v1b_p, pi_b) = (
        NodeId(0),
        NodeId(1),
        NodeId(2),
        NodeId(3),
        NodeId(4),
        NodeId(5),
        NodeId(6),
    );

    let mut pi_cfg = SimConfig::new(SR, pi_num);
    pi_cfg
        .add_resistor(v1b_g, g, 470_000.0)
        .add_resistor(cath, g, 10_000.0)
        .add_resistor(v1a_p, pi_b, 100_000.0)
        .add_resistor(v1b_p, pi_b, 100_000.0)
        .add_resistor(pi_b, g, 1_000_000.0)
        .add_triode(v1a_p, v1a_g, cath, 0)
        .add_triode(v1b_p, v1b_g, cath, 0)
        .input(v1a_g)
        .output(v1b_p)
        .bplus(pi_b, 300.0);

    let mut pi = Simulator::new(pi_cfg, vec![TriodeParams::new_12ax7()], vec![], vec![]);

    // --- Push-pull output stage (EL84×2 + OPT + speaker) ---
    // OPT: 100Ω DCR + 10H each half-primary
    // Speaker output computed analytically from differential plate voltage
    let op_num = 10;
    let (eag, eak, eas, eap, ebg, ebk, ebs, ebp, ct) = (
        NodeId(1),
        NodeId(2),
        NodeId(3),
        NodeId(4),
        NodeId(5),
        NodeId(6),
        NodeId(7),
        NodeId(8),
        NodeId(9),
    );

    let mut op_cfg = SimConfig::new(SR, op_num);
    op_cfg
        .add_resistor(eag, g, 470_000.0) // grid leak upper
        .add_resistor(ebg, g, 470_000.0) // grid leak lower
        .add_resistor(eak, g, 150.0) // cathode bias upper
        .add_resistor(ebk, g, 150.0) // cathode bias lower
        .add_resistor(eas, ct, 1_000.0) // screen resistor upper
        .add_resistor(ebs, ct, 1_000.0) // screen resistor lower
        .add_resistor(ct, g, 1_000_000.0) // B+ bleeder
        .add_resistor(eap, ct, 100.0)
        .add_inductor(eap, ct, 10.0)
        .add_resistor(ebp, ct, 100.0)
        .add_inductor(ebp, ct, 10.0)
        .add_pentode(eap, eag, eak, eas, 0) // EL84 upper
        .add_pentode(ebp, ebg, ebk, ebs, 0) // EL84 lower
        .input(eag)
        .input2(ebg)
        .output(eap) // output = upper plate for logging
        .bplus(ct, 300.0);

    let mut op = Simulator::new(op_cfg, vec![], vec![PentodeParams::new_el84()], vec![]);

    // Warmup: both Simulators
    for pi_ in 0..5000 {
        pi.set_bplus(300.0 * (pi_ as f64) / 5000.0);
        pi.process_sample(0.0)?;
    }
    pi.set_bplus(300.0);
    for _ in 0..5000 {
        pi.process_sample(0.0)?;
    }

    // Warmup output stage: B+ ramp + settle
    for i in 0..5000 {
        op.set_bplus(300.0 * (i as f64) / 5000.0);
        op.process_sample(0.0)?;
    }
    op.set_bplus(300.0);
    for _ in 0..20000 {
        op.process_sample(0.0)?;
    }

    println!();
    println!("=== Push-Pull DC Bias ===");
    println!(
        "12AX7 V1a: Vg=0.00 Vk={:.2} Vp={:.2}",
        pi.node_voltage(cath),
        pi.node_voltage(v1a_p)
    );
    println!(
        "12AX7 V1b: Vg=0.00 Vk={:.2} Vp={:.2}",
        pi.node_voltage(cath),
        pi.node_voltage(v1b_p)
    );
    println!(
        "EL84a: Vg={:.2} Vk={:.2} Vs={:.2} Vp={:.2}",
        op.node_voltage(eag),
        op.node_voltage(eak),
        op.node_voltage(eas),
        op.node_voltage(eap)
    );
    println!(
        "EL84b: Vg={:.2} Vk={:.2} Vs={:.2} Vp={:.2}",
        op.node_voltage(ebg),
        op.node_voltage(ebk),
        op.node_voltage(ebs),
        op.node_voltage(ebp)
    );

    // AC test: drive phase inverter, AC-couple to output stage
    let n = (SR as f64 * 0.5) as usize;
    let mut vpa_log = vec![0.0f32; n];
    let mut vpb_log = vec![0.0f32; n];

    // External RC high-pass (simulates coupling cap + grid leak)
    let h = 1.0 / SR as f64;
    let tau = 470_000.0 * 0.022e-6;
    let alpha = 1.0 - (-h / tau).exp();
    let mut dc_block_a = 0.0;
    let mut dc_block_b = 0.0;

    // Settle DC-block filters: charge to steady-state plate DC with zero input
    for _ in 0..5000 {
        let _ = pi.process_sample(0.0)?;
        let vpa = pi.node_voltage(v1a_p) as f64;
        let vpb = pi.node_voltage(v1b_p) as f64;
        dc_block_a += alpha * (vpa - dc_block_a);
        dc_block_b += alpha * (vpb - dc_block_b);
        let _ = op.process_sample(0.0)?;
    }

    for i in 0..n {
        let t = i as f64 / SR as f64;
        let vin = (2.0 * PI * 1000.0 * t).sin() as f32 * 1.0;

        let _ = pi.process_sample(vin)?;
        let vpa = pi.node_voltage(v1a_p) as f64;
        let vpb = pi.node_voltage(v1b_p) as f64;

        let ac_a = vpa - dc_block_a;
        dc_block_a += alpha * (vpa - dc_block_a);
        let ac_b = vpb - dc_block_b;
        dc_block_b += alpha * (vpb - dc_block_b);

        let _ = op.process_sample_dual(ac_a as f32, ac_b as f64)?;
        vpa_log[i] = op.node_voltage(eap);
        vpb_log[i] = op.node_voltage(ebp);
    }

    // Analyze output: differential plate voltage → speaker via turns ratio
    let settle = (SR as f64 * 0.1) as usize;
    let vpa_steady = &vpa_log[settle..];
    let vpb_steady = &vpb_log[settle..];
    let turns = f64::sqrt(5000.0 / 8.0); // 25:1 for 5kΩ:8Ω

    let dc_a: f32 = vpa_steady.iter().sum::<f32>() / vpa_steady.len() as f32;
    let dc_b: f32 = vpb_steady.iter().sum::<f32>() / vpb_steady.len() as f32;
    let mut spk_rms = 0.0f64;
    for (a, b) in vpa_steady.iter().zip(vpb_steady.iter()) {
        let diff = ((*a - dc_a) - (*b - dc_b)) as f64;
        spk_rms += diff * diff;
    }
    spk_rms = (spk_rms / vpa_steady.len() as f64 / turns / turns).sqrt();
    let pwr_mw = spk_rms * spk_rms / 8.0 * 1000.0;

    println!();
    println!("=== Push-Pull AC (1kHz, 1.0Vpk input, 10kΩ LTP tail) ===");
    println!("Speaker: {:.1} mV RMS, {:.1} mW", spk_rms * 1000.0, pwr_mw);

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

    match test_full_push_pull() {
        Ok(()) => eprintln!("Full push-pull: OK"),
        Err(e) => eprintln!("Full push-pull FAILED: {}", e),
    }
}
