use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use danji::{single_triode_config, DiodeParams, NodeId, SimConfig, Simulator, TriodeParams};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::params::{DcBlocker, MainCommand, SharedParams};

// ── Inter-stage HPF (RC coupling) ──

pub struct HpfFilter {
    cv: f32,
    alpha: f32,
}

impl HpfFilter {
    /// R = grid leak resistor (Ω), C = coupling capacitor (F)
    pub fn new(sample_rate: u32, r: f64, c: f64) -> Self {
        let h = 1.0 / sample_rate as f64;
        let alpha = 1.0 - (-h / (r * c)).exp();
        Self {
            cv: 0.0,
            alpha: alpha as f32,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.cv = 0.0;
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let ac = x - self.cv;
        self.cv = self.cv + self.alpha * (x - self.cv);
        ac
    }
}

// ── Audio Engine ──

#[allow(clippy::large_enum_variant)]
pub enum AudioEngine {
    Single {
        sim_l: Simulator,
        sim_r: Simulator,
        dc_l: DcBlocker,
        dc_r: DcBlocker,
    },
    TwoStage {
        s1_l: Simulator,
        s1_r: Simulator,
        s2_l: Simulator,
        s2_r: Simulator,
        hpf_l: HpfFilter,
        hpf_r: HpfFilter,
        dc_l: DcBlocker,
        dc_r: DcBlocker,
    },
    Chain {
        s1_l: Simulator,
        s1_r: Simulator,
        s2_l: Simulator,
        s2_r: Simulator,
        tone_l: Simulator,
        tone_r: Simulator,
        hpf1_l: HpfFilter,
        hpf1_r: HpfFilter,
        hpf2_l: HpfFilter,
        hpf2_r: HpfFilter,
        dc_l: DcBlocker,
        dc_r: DcBlocker,
        psu_table: Vec<f32>,
        psu_idx: usize,
        _bplus: f64,
    },
}

impl AudioEngine {
    /// Process a stereo frame. Returns (left, right) raw voltage (DC still present).
    pub fn process_frame(&mut self, l_in: f32, r_in: f32) -> (f32, f32) {
        match self {
            AudioEngine::Single {
                sim_l,
                sim_r,
                dc_l,
                dc_r,
            } => {
                let l_raw = sim_l.process_sample(l_in).unwrap_or(0.0);
                let r_raw = sim_r.process_sample(r_in).unwrap_or(0.0);
                (dc_l.process(l_raw), dc_r.process(r_raw))
            }
            AudioEngine::TwoStage {
                s1_l,
                s1_r,
                s2_l,
                s2_r,
                hpf_l,
                hpf_r,
                dc_l,
                dc_r,
            } => {
                let l1 = s1_l.process_sample(l_in).unwrap_or(0.0);
                let r1 = s1_r.process_sample(r_in).unwrap_or(0.0);
                let l1_ac = hpf_l.process(l1);
                let r1_ac = hpf_r.process(r1);
                let l_raw = s2_l.process_sample(l1_ac).unwrap_or(0.0);
                let r_raw = s2_r.process_sample(r1_ac).unwrap_or(0.0);
                (dc_l.process(l_raw), dc_r.process(r_raw))
            }
            AudioEngine::Chain {
                s1_l,
                s1_r,
                s2_l,
                s2_r,
                tone_l,
                tone_r,
                hpf1_l,
                hpf1_r,
                hpf2_l,
                hpf2_r,
                dc_l,
                dc_r,
                psu_table,
                psu_idx,
                _bplus,
            } => {
                let bp = psu_table[*psu_idx] as f64;
                *psu_idx = (*psu_idx + 1) % psu_table.len();
                s1_l.set_bplus(bp);
                s1_r.set_bplus(bp);
                s2_l.set_bplus(bp);
                s2_r.set_bplus(bp);

                let l1 = s1_l.process_sample(l_in).unwrap_or(0.0);
                let r1 = s1_r.process_sample(r_in).unwrap_or(0.0);
                let l1_ac = hpf1_l.process(l1);
                let r1_ac = hpf1_r.process(r1);
                let lt = tone_l.process_sample(l1_ac).unwrap_or(0.0);
                let rt = tone_r.process_sample(r1_ac).unwrap_or(0.0);
                let l2_ac = hpf2_l.process(lt);
                let r2_ac = hpf2_r.process(rt);
                let l_raw = s2_l.process_sample(l2_ac).unwrap_or(0.0);
                let r_raw = s2_r.process_sample(r2_ac).unwrap_or(0.0);
                (dc_l.process(l_raw), dc_r.process(r_raw))
            }
        }
    }
}

// ── Device helpers ──

pub fn find_device<F>(host: &cpal::Host, predicate: F) -> Option<cpal::Device>
where
    F: Fn(&cpal::Device) -> bool,
{
    host.devices().ok()?.into_iter().find(predicate)
}

pub fn blackhole_device(host: &cpal::Host) -> Result<cpal::Device, String> {
    find_device(host, |d| {
        d.description()
            .map(|desc| desc.name().contains("BlackHole"))
            .unwrap_or(false)
    })
    .ok_or_else(|| "BlackHole device not found".into())
}

pub fn output_device(host: &cpal::Host) -> Result<cpal::Device, String> {
    find_device(host, |d| {
        d.description()
            .map(|desc| {
                !desc.name().contains("BlackHole")
                    && !desc.name().contains("多输出")
                    && !desc.name().contains("Aggregate")
            })
            .unwrap_or(false)
            && d.supported_output_configs()
                .ok()
                .is_some_and(|mut c| c.next().is_some())
    })
    .ok_or_else(|| "No physical output device found".into())
}

// ── Tube params ──

pub fn tube_params(name: &str) -> Result<Vec<TriodeParams>, String> {
    Ok(vec![match name {
        "12AX7" => TriodeParams::new_12ax7(),
        "12AU7" => TriodeParams::new_12au7(),
        "12AT7" => TriodeParams::new_12at7(),
        "6DJ8" => TriodeParams::new_6dj8(),
        "6L6GC" => TriodeParams::new_6l6gc(),
        "6550" => TriodeParams::new_6550(),
        "EL34" => TriodeParams::new_el34(),
        "KT88" => TriodeParams::new_kt88(),
        _ => return Err(format!("unknown tube: {name}")),
    }])
}

// ── Engine builders ──

fn warmup(sim: &mut Simulator) {
    for _ in 0..3000 {
        sim.process_sample(0.0).ok();
    }
}

fn build_single(sr: u32, tube_p: &[TriodeParams], bplus: f64) -> AudioEngine {
    let cfg = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, bplus);
    let mut sl = Simulator::new(cfg.clone(), tube_p.to_vec(), vec![], vec![]);
    let mut sr_sim = Simulator::new(cfg, tube_p.to_vec(), vec![], vec![]);
    warmup(&mut sl);
    warmup(&mut sr_sim);
    AudioEngine::Single {
        sim_l: sl,
        sim_r: sr_sim,
        dc_l: DcBlocker::new(sr),
        dc_r: DcBlocker::new(sr),
    }
}

fn build_two_stage(sr: u32, tube_p: &[TriodeParams], bplus: f64) -> AudioEngine {
    let cfg = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, bplus);
    let mut s1_l = Simulator::new(cfg.clone(), tube_p.to_vec(), vec![], vec![]);
    let mut s1_r = Simulator::new(cfg.clone(), tube_p.to_vec(), vec![], vec![]);
    let mut s2_l = Simulator::new(cfg.clone(), tube_p.to_vec(), vec![], vec![]);
    let mut s2_r = Simulator::new(cfg, tube_p.to_vec(), vec![], vec![]);
    warmup(&mut s1_l);
    warmup(&mut s1_r);
    warmup(&mut s2_l);
    warmup(&mut s2_r);
    let hpf = |sample_rate| HpfFilter::new(sample_rate, 1_000_000.0, 0.022e-6);
    AudioEngine::TwoStage {
        s1_l,
        s1_r,
        s2_l,
        s2_r,
        hpf_l: hpf(sr),
        hpf_r: hpf(sr),
        dc_l: DcBlocker::new(sr),
        dc_r: DcBlocker::new(sr),
    }
}

fn build_psu_table(sr: u32, bplus: f64) -> Vec<f32> {
    let period = (sr as f64 / 120.0).ceil() as usize;
    let mut psu_cfg = SimConfig::new(sr, 4);
    let (g, a, b1, bp) = (NodeId(0), NodeId(1), NodeId(2), NodeId(3));
    psu_cfg
        .add_diode(a, b1, 0)
        .add_capacitor(b1, g, 47e-6)
        .add_resistor(b1, bp, 100.0)
        .add_capacitor(bp, g, 47e-6)
        .add_resistor(bp, g, 220e3)
        .input(a)
        .output(bp);
    let mut psu = Simulator::new(psu_cfg, vec![], vec![], vec![DiodeParams::new_5ar4()]);
    let mut table = Vec::with_capacity(period);
    for i in 0..period {
        let t = i as f64 / sr as f64;
        let vac = (bplus * (2.0 * std::f64::consts::PI * 60.0 * t).sin().abs()) as f32;
        psu.process_sample(vac).ok();
        table.push(psu.node_voltage(bp));
    }
    table
}

fn build_chain(sr: u32, tube_p: &[TriodeParams], bplus: f64) -> AudioEngine {
    let cfg = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, bplus);

    let mut s1_l = Simulator::new(cfg.clone(), tube_p.to_vec(), vec![], vec![]);
    let mut s1_r = Simulator::new(cfg.clone(), tube_p.to_vec(), vec![], vec![]);
    let mut s2_l = Simulator::new(cfg.clone(), tube_p.to_vec(), vec![], vec![]);
    let mut s2_r = Simulator::new(cfg.clone(), tube_p.to_vec(), vec![], vec![]);
    warmup(&mut s1_l);
    warmup(&mut s1_r);
    warmup(&mut s2_l);
    warmup(&mut s2_r);

    // Tone control: passive RC network (fixed)
    let mut t_cfg = SimConfig::new(sr, 4);
    let (ti, tm, to) = (NodeId(1), NodeId(2), NodeId(3));
    t_cfg
        .add_resistor(ti, tm, 100_000.0)
        .add_capacitor(tm, NodeId(0), 330e-12)
        .add_capacitor(ti, to, 0.022e-6)
        .add_resistor(to, NodeId(0), 100_000.0)
        .input(ti)
        .output(to);
    let mut tone_l = Simulator::new(t_cfg.clone(), vec![], vec![], vec![]);
    let mut tone_r = Simulator::new(t_cfg, vec![], vec![], vec![]);
    warmup(&mut tone_l);
    warmup(&mut tone_r);

    let psu_table = build_psu_table(sr, bplus);
    let hpf_c = || HpfFilter::new(sr, 1_000_000.0, 0.022e-6);

    AudioEngine::Chain {
        s1_l,
        s1_r,
        s2_l,
        s2_r,
        tone_l,
        tone_r,
        hpf1_l: hpf_c(),
        hpf1_r: hpf_c(),
        hpf2_l: hpf_c(),
        hpf2_r: hpf_c(),
        dc_l: DcBlocker::new(sr),
        dc_r: DcBlocker::new(sr),
        psu_table,
        psu_idx: 0,
        _bplus: bplus,
    }
}

// ── Build engine from params ──

pub fn build_engine(
    sr: u32,
    params: &SharedParams,
) -> Result<AudioEngine, Box<dyn std::error::Error>> {
    let tube = params.tube.read().unwrap().clone();
    let tube_p = tube_params(&tube)?;
    let bplus = params.bplus_voltage();
    let model = params.model.read().unwrap().clone();

    Ok(match model.as_str() {
        "single" => build_single(sr, &tube_p, bplus),
        "two-stage" => build_two_stage(sr, &tube_p, bplus),
        "chain" => build_chain(sr, &tube_p, bplus),
        _ => build_single(sr, &tube_p, bplus),
    })
}

// ── Main engine runner (owns streams, processes commands) ──

pub fn run_engine(
    blackhole: &cpal::Device,
    output: &cpal::Device,
    input_cfg: &StreamConfig,
    output_cfg: &StreamConfig,
    params: Arc<SharedParams>,
    cmd_rx: mpsc::Receiver<MainCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sr = input_cfg.sample_rate;

    loop {
        let engine = Arc::new(Mutex::new(build_engine(sr, &params)?));
        let (tx, rx) = mpsc::sync_channel::<f32>(65536);

        let p = params.clone();
        let eng = engine.clone();
        let input_stream = blackhole.build_input_stream(
            input_cfg,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let vol = p.volume_linear();
                if p.bypass_enabled() {
                    for &s in data {
                        if tx.try_send(s * vol).is_err() {
                            break;
                        }
                    }
                    return;
                }
                let mut e = eng.lock().unwrap();
                for frame in data.chunks(2) {
                    let l_in = frame[0];
                    let r_in = frame.get(1).copied().unwrap_or(l_in);
                    let (l_raw, r_raw) = e.process_frame(l_in, r_in);
                    let l_out = (l_raw * vol).clamp(-1.0, 1.0);
                    let r_out = (r_raw * vol).clamp(-1.0, 1.0);
                    if tx.try_send(l_out).is_err() || tx.try_send(r_out).is_err() {
                        break;
                    }
                }
            },
            move |err| log::error!("input error: {err}"),
            None,
        )?;

        let output_stream = output.build_output_stream(
            output_cfg,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    *sample = rx.try_recv().unwrap_or(0.0);
                }
            },
            move |err| log::error!("output error: {err}"),
            None,
        )?;

        input_stream.play()?;
        output_stream.play()?;
        log::info!("Engine started");

        loop {
            match cmd_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(MainCommand::SetModel { model, resp }) => {
                    drop(input_stream);
                    drop(output_stream);
                    *params.model.write().unwrap() = model;
                    let msg = match build_engine(sr, &params) {
                        Ok(_) => "OK".into(),
                        Err(e) => format!("ERROR {e}"),
                    };
                    let _ = resp.send(msg);
                    break;
                }
                Ok(MainCommand::SetTube { tube, resp }) => {
                    // Full rebuild (same as model switch) for simplicity
                    drop(input_stream);
                    drop(output_stream);
                    let msg = match tube_params(&tube) {
                        Ok(_tp) => {
                            *params.tube.write().unwrap() = tube;
                            match build_engine(sr, &params) {
                                Ok(_) => "OK".into(),
                                Err(e) => format!("ERROR {e}"),
                            }
                        }
                        Err(e) => format!("ERROR {e}"),
                    };
                    let _ = resp.send(msg);
                    break;
                }
                Ok(MainCommand::Stop) => {
                    drop(input_stream);
                    drop(output_stream);
                    return Ok(());
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return Ok(()),
            }
        }
    }
}
