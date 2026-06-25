use danji::{NodeId, SimConfig, Simulator};

fn main() -> Result<(), danji::DanjiError> {
    let sr = 44100u32;
    // Test: 3-winding coupled inductor with k=0.99 (push-pull transformer core)
    // Windings: p1-ct (2.5H), p2-ct (2.5H), spk-gnd (0.016H)
    let mut cfg = SimConfig::new(sr, 5);
    let (g, p1, ct, p2, spk) = (NodeId(0), NodeId(1), NodeId(2), NodeId(3), NodeId(4));

    cfg.add_resistor(p1, g, 1_000.0)
        .add_resistor(p2, g, 1_000.0)
        .add_resistor(spk, g, 8.0)
        .add_coupled_inductor3(p1, ct, p2, spk, g, 2.5, 2.5, 0.016, 0.99, 0.95, 0.95)
        .input(p1)
        .output(spk)
        .bplus(ct, 10.0);

    let mut sim = Simulator::new(cfg, vec![], vec![], vec![]);
    for i in 0..10 {
        let v = sim.process_sample(0.0)?;
        eprintln!(
            "[{i}] Vp1={:.2} Vp2={:.2} Vct={:.2} Vspk={:.4}",
            sim.node_voltage(p1),
            sim.node_voltage(p2),
            sim.node_voltage(ct),
            v
        );
    }

    // Verify: CT at ~5V (voltage divider from two VSRC_G on p1 and ct... no, ct only from B+)
    // Actually ct has B+=10V, p1 has input=0V. Vct should be ~10V.
    eprintln!(
        "Steady: Vct={:.1} Vspk={:.3}",
        sim.node_voltage(ct),
        sim.node_voltage(spk)
    );

    Ok(())
}
