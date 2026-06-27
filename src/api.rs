//! 公共 API 模块，重新导出核心类型。
//!
//! 本模块汇总了库中最常用的类型，便于用户直接从 crate 根导入。
//!
//! # 主要类型
//!
//! - [`Simulator`] - 仿真器
//! - [`SimConfig`] - 电路配置
//! - [`TriodeParams`] - 三极管参数
//! - [`PentodeParams`] - 五极管参数
//! - [`DiodeParams`] - 二极管参数
//! - [`NodeId`] - 电路节点标识
//! - [`DanjiError`] - 错误枚举
//! - [`single_triode_config`] - 单三极管电路配置工厂函数
//!
//! ---
//!
//! Public API module, re-exporting core types.
//!
//! This module aggregates the most commonly used types for convenient
//! import directly from the crate root.
//!
//! # Main Types
//!
//! - [`Simulator`] - Simulator
//! - [`SimConfig`] - Circuit configuration
//! - [`TriodeParams`] - Triode parameters
//! - [`PentodeParams`] - Pentode parameters
//! - [`DiodeParams`] - Diode parameters
//! - [`NodeId`] - Circuit node identifier
//! - [`DanjiError`] - Error enum
//! - [`single_triode_config`] - Single triode circuit config factory

pub use crate::circuit::node::NodeId;
pub use crate::error::DanjiError;
pub use crate::simulator::{single_triode_config, SimConfig, Simulator};
pub use crate::tube::diode::DiodeParams;
pub use crate::tube::params::PentodeParams;
pub use crate::tube::params::TriodeParams;
