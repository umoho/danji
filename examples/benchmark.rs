use danji::{single_triode_config, Simulator, TriodeParams};
use std::time::Instant;

fn main() -> Result<(), danji::DanjiError> {
    let sample_rate = 44100u32;
    let duration_secs = 10.0;
    let num_samples = (sample_rate as f64 * duration_secs) as usize;

    let config = single_triode_config(sample_rate, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, 300.0);
    let params = vec![TriodeParams::new_12ax7()];

    let input: Vec<f32> = (0..num_samples)
        .map(|i| {
            (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sample_rate as f64).sin() as f32 * 0.1
        })
        .collect();

    let mut output = vec![0.0f32; num_samples];

    // warmup
    {
        let mut sim = Simulator::new(config.clone(), params.clone(), vec![], vec![]);
        sim.process_buffer(&input, &mut output)?;
    }

    // benchmark
    let mut sim = Simulator::new(config, params, vec![], vec![]);
    let start = Instant::now();
    sim.process_buffer(&input, &mut output)?;
    let elapsed = start.elapsed();

    let realtime_ratio = duration_secs / elapsed.as_secs_f64();
    let ms_per_sample = elapsed.as_secs_f64() / num_samples as f64 * 1000.0;

    println!("=== CPU Performance Benchmark ===");
    println!("Sample rate: {} Hz", sample_rate);
    println!("Duration: {:.1} s", duration_secs);
    println!("Samples: {}", num_samples);
    println!("Processing time: {:.3} s", elapsed.as_secs_f64());
    println!(
        "Samples per second: {:.0}",
        num_samples as f64 / elapsed.as_secs_f64()
    );
    println!("Real-time ratio: {:.2}x", realtime_ratio);
    println!("Time per sample: {:.2} us", ms_per_sample * 1000.0);

    if realtime_ratio >= 1.0 {
        println!(
            "STATUS: REALTIME (can process {}x faster than realtime)",
            realtime_ratio
        );
    } else {
        println!(
            "STATUS: NOT REALTIME (needs {:.1}x speedup)",
            1.0 / realtime_ratio
        );
    }

    Ok(())
}
