//! 真空管物理模型模块。
//!
//! 提供三极管、五极管和二极管的物理模型，基于 Koren 模型和 Child-Langmuir 定律。
//!
//! # 子模块
//!
//! - [`triode`] - 三极管模型
//! - [`pentode`] - 五极管模型
//! - [`diode`] - 二极管模型
//! - [`params`] - 真空管参数
//!
//! ---
//!
//! Vacuum tube physics model module.
//!
//! Provides triode, pentode, and diode physics models based on Koren model
//! and Child-Langmuir law.
//!
//! # Submodules
//!
//! - [`triode`] - Triode model
//! - [`pentode`] - Pentode model
//! - [`diode`] - Diode model
//! - [`params`] - Tube parameters

pub mod diode;
pub mod params;
pub mod pentode;
pub mod triode;
