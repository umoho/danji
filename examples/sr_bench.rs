use danji::{single_triode_config, Simulator, TriodeParams, SimConfig, NodeId, DiodeParams};
use std::time::Instant;

fn bench_single(sr: u32, duration_secs: f64) -> f64 {
    let n = (sr as f64 * duration_secs) as usize;
    let cfg = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0);
    let mut sim = Simulator::new(cfg, vec![TriodeParams::new_12ax7()], vec![]);
    let input: Vec<f32> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin() as f32 * 0.01)
        .collect();
    let mut output = vec![0.0f32; n];
    let start = Instant::now();
    sim.process_buffer(&input, &mut output).unwrap();
    let elapsed = start.elapsed().as_secs_f64();
    duration_secs / elapsed
}

fn bench_two_stage(sr: u32, duration_secs: f64) -> f64 {
    let warmup = (sr as f64 * 0.1) as usize; // 100ms warmup
    let n = (sr as f64 * duration_secs) as usize;
    let cfg1 = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0);
    let cfg2 = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0);
    let mut s1 = Simulator::new(cfg1, vec![TriodeParams::new_12ax7()], vec![]);
    let mut s2 = Simulator::new(cfg2, vec![TriodeParams::new_12ax7()], vec![]);

    for _ in 0..warmup { s1.process_sample(0.0).unwrap(); }
    for _ in 0..warmup { s2.process_sample(0.0).unwrap(); }

    let input: Vec<f32> = (0..n)
        .map(|i| (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin() as f32 * 0.01)
        .collect();

    let h = 1.0 / sr as f64;
    let tau = 1_000_000.0 * 0.022e-6;
    let alpha = 1.0 - (-h / tau).exp();
    let mut cv = 0.0;
    let mut out = vec![0.0f32; n];
    let start = Instant::now();
    for i in 0..n {
        let v1 = s1.process_sample(input[i]).unwrap() as f64;
        let ac = v1 - cv; cv += alpha * (v1 - cv);
        out[i] = s2.process_sample(ac as f32).unwrap();
    }
    let elapsed = start.elapsed().as_secs_f64();
    duration_secs / elapsed
}

fn bench_chain(sr: u32, duration_secs: f64) -> f64 {
    let warmup = (sr as f64 * 0.1) as usize;
    let n = (sr as f64 * duration_secs) as usize;
    let mut psu_cfg = SimConfig::new(sr, 4);
    let (gnd, ac_n, b1, bplus) = (NodeId(0), NodeId(1), NodeId(2), NodeId(3));
    psu_cfg.add_diode(ac_n, b1, 0).add_capacitor(b1, gnd, 47e-6)
           .add_resistor(b1, bplus, 100.0).add_capacitor(bplus, gnd, 47e-6)
           .add_resistor(bplus, gnd, 220e3).input(ac_n).output(bplus);
    let mut psu = Simulator::new(psu_cfg, vec![], vec![DiodeParams::new_5ar4()]);

    let mut bp = vec![0.0f32; n];
    for (i, v) in bp.iter_mut().enumerate() {
        let t = i as f64 / sr as f64;
        psu.process_sample((300.0 * (2.0 * std::f64::consts::PI * 60.0 * t).sin().abs()) as f32).unwrap();
        *v = psu.node_voltage(bplus);
    }

    let sc = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 0.0);
    let mut s1 = Simulator::new(sc.clone(), vec![TriodeParams::new_12ax7()], vec![]);
    let mut s2 = Simulator::new(sc, vec![TriodeParams::new_12ax7()], vec![]);

    for _ in 0..warmup { s1.process_sample(0.0).unwrap(); }
    for _ in 0..warmup { s2.process_sample(0.0).unwrap(); }

    let h = 1.0 / sr as f64;
    let tau = 1_000_000.0 * 0.022e-6;
    let alpha = 1.0 - (-h / tau).exp();
    let mut c1 = 0.0; let mut c2 = 0.0;

    let input: Vec<f32> = (0..n).map(|i| {
        (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin() as f32 * 0.01
    }).collect();
    let mut output = vec![0.0f32; n];
    let start = Instant::now();
    for i in 0..n {
        let bp_v = bp[i] as f64;
        s1.set_bplus(bp_v); s2.set_bplus(bp_v);
        let v1 = s1.process_sample(input[i]).unwrap() as f64;
        let a1 = v1 - c1; c1 += alpha * (v1 - c1);
        let a2 = a1 - c2; c2 += alpha * (a1 - c2);
        output[i] = s2.process_sample(a2 as f32).unwrap();
    }
    let elapsed = start.elapsed().as_secs_f64();
    duration_secs / elapsed
}

fn main() {
    println!("{:>10} {:>12} {:>14}", "sr(Hz)", "dur(s)", "single(xRT)");
    for &sr in &[44100u32, 96000, 192000] {
        let s = bench_single(sr, 1.0);
        println!("{:>10} {:>12} {:>14.1}", sr, 1, s);
    }
}
