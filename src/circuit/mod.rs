//! 电路建模模块。
//!
//! 提供电路节点、元件（电阻、电容、电感、耦合电感）和 MNA 求解器的定义。
//!
//! ---
//!
//! Circuit modeling module.
//!
//! Provides definitions for circuit nodes, elements (resistor, capacitor, inductor,
//! coupled inductor), and MNA solver.

pub mod element;
pub mod node;
pub mod solver;
