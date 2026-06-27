//! 公共 API 模块，重新导出核心类型。
//!
//! 本模块汇总了库中最常用的类型，便于用户直接从 crate 根导入。
//!
//! ---
//!
//! Public API module, re-exporting core types.
//!
//! This module aggregates the most commonly used types for convenient
//! import directly from the crate root.

pub use crate::circuit::node::NodeId;
pub use crate::error::DanjiError;
pub use crate::simulator::{single_triode_config, SimConfig, Simulator};
pub use crate::tube::diode::DiodeParams;
pub use crate::tube::params::PentodeParams;
pub use crate::tube::params::TriodeParams;
