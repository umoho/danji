use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::StreamConfig;
use danji::{single_triode_config, Simulator, TriodeParams};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::params::{DcBlocker, MainCommand, SharedParams};

pub struct Engine {
    pub sim_l: Simulator,
    pub sim_r: Simulator,
    pub dc_l: DcBlocker,
    pub dc_r: DcBlocker,
}

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

pub fn build_engine(sr: u32, params: &SharedParams) -> Result<Engine, Box<dyn std::error::Error>> {
    let tube = params.tube.read().unwrap().clone();
    let tube_p = tube_params(&tube)?;
    let bplus = params.bplus_voltage();

    let cfg = single_triode_config(sr, 100_000.0, 1_500.0, 22e-6, 1_000_000.0, bplus);
    let mut sim_l = Simulator::new(cfg.clone(), tube_p.clone(), vec![], vec![]);
    let mut sim_r = Simulator::new(cfg, tube_p, vec![], vec![]);
    for _ in 0..3000 {
        sim_l.process_sample(0.0).ok();
        sim_r.process_sample(0.0).ok();
    }
    Ok(Engine {
        sim_l,
        sim_r,
        dc_l: DcBlocker::new(sr),
        dc_r: DcBlocker::new(sr),
    })
}

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
                    let l_raw = e.sim_l.process_sample(l_in).unwrap_or(0.0);
                    let r_raw = e.sim_r.process_sample(r_in).unwrap_or(0.0);
                    let l_out = (e.dc_l.process(l_raw) * vol).clamp(-1.0, 1.0);
                    let r_out = (e.dc_r.process(r_raw) * vol).clamp(-1.0, 1.0);
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
                    let msg = match tube_params(&tube) {
                        Ok(tp) => {
                            let bplus = params.bplus_voltage();
                            let cfg = single_triode_config(
                                sr,
                                100_000.0,
                                1_500.0,
                                22e-6,
                                1_000_000.0,
                                bplus,
                            );
                            let mut new_l = Simulator::new(cfg.clone(), tp.clone(), vec![], vec![]);
                            let mut new_r = Simulator::new(cfg, tp, vec![], vec![]);
                            for _ in 0..3000 {
                                new_l.process_sample(0.0).ok();
                            }
                            for _ in 0..3000 {
                                new_r.process_sample(0.0).ok();
                            }
                            let mut e = engine.lock().unwrap();
                            e.sim_l = new_l;
                            e.sim_r = new_r;
                            e.dc_l = DcBlocker::new(sr);
                            e.dc_r = DcBlocker::new(sr);
                            *params.tube.write().unwrap() = tube;
                            "OK".into()
                        }
                        Err(e) => format!("ERROR {e}"),
                    };
                    let _ = resp.send(msg);
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
