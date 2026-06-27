//! 共享参数模块。
//!
//! 提供音频线程和控制线程之间共享的仿真器参数，
//! 使用原子操作和读写锁实现线程安全。
//!
//! ---
//!
//! Shared parameters module.
//!
//! Provides simulator parameters shared between audio and control threads,
//! using atomic operations and read-write locks for thread safety.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

pub struct SharedParams {
    pub bypass: AtomicBool,
    gain: AtomicU32,
    volume: AtomicU32,
    bplus: AtomicU32,
    pub tube: RwLock<String>,
    pub model: RwLock<String>,
}

impl SharedParams {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            bypass: AtomicBool::new(false),
            gain: AtomicU32::new(f32::to_bits(10.0_f64.powf(-38.0 / 20.0) as f32)),
            volume: AtomicU32::new(f32::to_bits(10.0_f64.powf(-12.0 / 20.0) as f32)),
            bplus: AtomicU32::new(f32::to_bits(300.0)),
            tube: RwLock::new("12AX7".into()),
            model: RwLock::new("single".into()),
        })
    }

    pub fn gain_linear(&self) -> f32 {
        f32::from_bits(self.gain.load(Ordering::Relaxed))
    }
    pub fn volume_linear(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }
    pub fn bypass_enabled(&self) -> bool {
        self.bypass.load(Ordering::Relaxed)
    }
    pub fn bplus_voltage(&self) -> f64 {
        f32::from_bits(self.bplus.load(Ordering::Relaxed)) as f64
    }

    pub fn set_gain_db(&self, db: f64) {
        self.gain.store(
            f32::to_bits(10.0_f64.powf(db / 20.0) as f32),
            Ordering::Relaxed,
        );
    }
    pub fn set_volume_db(&self, db: f64) {
        self.volume.store(
            f32::to_bits(10.0_f64.powf(db / 20.0) as f32),
            Ordering::Relaxed,
        );
    }
    pub fn set_bplus(&self, v: f64) {
        self.bplus.store(f32::to_bits(v as f32), Ordering::Relaxed);
    }
}

pub struct DcBlocker {
    x_prev: f32,
    y_prev: f32,
    alpha: f32,
}

impl DcBlocker {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            x_prev: 0.0,
            y_prev: 0.0,
            alpha: (-2.0 * std::f32::consts::PI * 10.0 / sample_rate as f32).exp(),
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x_prev + self.alpha * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }
}

pub enum MainCommand {
    SetModel {
        model: String,
        resp: std::sync::mpsc::Sender<String>,
    },
    SetTube {
        tube: String,
        resp: std::sync::mpsc::Sender<String>,
    },
    Stop,
}
