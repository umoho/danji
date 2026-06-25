use danji::{NodeId, SimConfig, Simulator, TriodeParams};
use std::f64::consts::PI;

const SR: u32 = 44100;

fn test_phase_inverter() -> Result<(), danji::DanjiError> {
    // 12AX7 long-tail pair phase inverter
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

fn main() {
    env_logger::init();

    println!("=== Danji Push-Pull Power Stage ===");
    println!();
    println!("Phase inverter test:");
    if let Err(e) = test_phase_inverter() {
        eprintln!("  FAILED: {}", e);
        return;
    }

    println!();
    println!("Full push-pull: pending (see devlog)");
}
