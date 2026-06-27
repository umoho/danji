//! # danji - 真空管放大器物理建模仿真库
//!
//! 本库实现了基于 MNA（改进节点分析法）的真空管放大器电路仿真，
//! 支持三极管、五极管、二极管等真空管器件的物理建模。
//!
//! ## 主要模块 / Modules
//!
//! - [`simulator`] - 仿真器核心，提供 `Simulator` 和 `SimConfig` 构建器
//! - [`circuit`] - 电路建模，包括节点、元件和求解器
//! - [`tube`] - 真空管物理模型（三极管、五极管、二极管）
//! - [`error`] - 错误类型定义
//!
//! ## 快速开始 / Quick Start
//!
//! ```rust
//! use danji::{Simulator, SimConfig, TriodeParams, NodeId};
//!
//! let config = SimConfig::new(44100, 5);
//! // ... 添加电路元件
//! let sim = Simulator::new(config, vec![], vec![], vec![]);
//! ```
//!
//! ---
//!
//! # danji - Vacuum Tube Amplifier Physical Modeling Library
//!
//! This library implements vacuum tube amplifier circuit simulation based on
//! MNA (Modified Nodal Analysis), supporting physical modeling of triode,
//! pentode, and diode vacuum tube devices.
//!
//! ## Modules
//!
//! - [`simulator`] - Core simulator with `Simulator` and `SimConfig` builder
//! - [`circuit`] - Circuit modeling with nodes, elements, and solver
//! - [`tube`] - Vacuum tube physics models (triode, pentode, diode)
//! - [`error`] - Error type definitions
//!
//! ## Quick Start
//!
//! ```rust
//! use danji::{Simulator, SimConfig, TriodeParams, NodeId};
//!
//! let config = SimConfig::new(44100, 5);
//! // ... add circuit elements
//! let sim = Simulator::new(config, vec![], vec![], vec![]);
//! ```

pub mod api;
pub mod circuit;
pub mod error;
pub mod simulator;
pub mod tube;

pub use api::*;
