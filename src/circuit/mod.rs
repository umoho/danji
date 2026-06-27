//! 电路建模模块。
//!
//! 提供电路节点、元件（电阻、电容、电感、耦合电感）和 MNA 求解器的定义。
//!
//! # 子模块
//!
//! - [`node`] - 电路节点
//! - [`element`] - 电路元件
//! - [`solver`] - MNA 求解器
//!
//! ---
//!
//! Circuit modeling module.
//!
//! Provides definitions for circuit nodes, elements (resistor, capacitor, inductor,
//! coupled inductor), and MNA solver.
//!
//! # Submodules
//!
//! - [`node`] - Circuit nodes
//! - [`element`] - Circuit elements
//! - [`solver`] - MNA solver

pub mod element;
pub mod node;
pub mod solver;
