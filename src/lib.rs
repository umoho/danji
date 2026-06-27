//! # danji - 真空管放大器物理建模仿真库
//!
//! 本库实现了基于 MNA（改进节点分析法）的真空管放大器电路仿真，
//! 支持三极管、五极管、二极管等真空管器件的物理建模。
//!
//! # 子模块
//!
//! - [`simulator`] - 仿真器核心
//! - [`circuit`] - 电路建模
//! - [`tube`] - 真空管物理模型
//! - [`error`] - 错误类型
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
//!
//! ---
//!
//! # danji - Vacuum Tube Amplifier Physical Modeling Library
//!
//! This library implements vacuum tube amplifier circuit simulation based on
//! MNA (Modified Nodal Analysis), supporting physical modeling of triode,
//! pentode, and diode vacuum tube devices.
//!
//! # Submodules
//!
//! - [`simulator`] - Core simulator
//! - [`circuit`] - Circuit modeling
//! - [`tube`] - Vacuum tube physics models
//! - [`error`] - Error types
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

pub mod api;
pub mod circuit;
pub mod error;
pub mod simulator;
pub mod tube;

pub use api::*;
