//! 电路节点标识模块。
//!
//! 提供 [`NodeId`] 类型，用于标识电路中的节点。
//!
//! ---
//!
//! Circuit node identifier module.
//!
//! Provides [`NodeId`] type for identifying nodes in a circuit.

/// 电路节点标识。
///
/// 使用 `NodeId(0)` 表示接地节点（GND），其他节点从 1 开始编号。
///
/// ---
///
/// Circuit node identifier.
///
/// `NodeId(0)` represents ground (GND), other nodes are numbered starting from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);
