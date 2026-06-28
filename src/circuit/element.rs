//! 电路元件模块。
//!
//! 定义电阻、电容、电感、耦合电感和真空管实例等电路元件。
//!
//! # 主要类型
//!
//! - [`Resistor`] - 电阻
//! - [`Capacitor`] - 电容
//! - [`Inductor`] - 电感
//! - [`CoupledInductor`] - 双绕组耦合电感
//! - [`CoupledInductor3`] - 三绕组耦合电感
//! - [`TriodeInstance`] - 三极管实例
//! - [`PentodeInstance`] - 五极管实例
//! - [`DiodeInstance`] - 二极管实例
//! - [`CircuitDef`] - 电路定义
//!
//! ---
//!
//! Circuit element module.
//!
//! Defines resistor, capacitor, inductor, coupled inductor, and vacuum tube
//! instance circuit elements.
//!
//! # Main Types
//!
//! - [`Resistor`] - Resistor
//! - [`Capacitor`] - Capacitor
//! - [`Inductor`] - Inductor
//! - [`CoupledInductor`] - Dual-winding coupled inductor
//! - [`CoupledInductor3`] - Three-winding coupled inductor
//! - [`TriodeInstance`] - Triode instance
//! - [`PentodeInstance`] - Pentode instance
//! - [`DiodeInstance`] - Diode instance
//! - [`CircuitDef`] - Circuit definition

use crate::circuit::node::NodeId;

/// 电阻元件。
///
/// ---
///
/// Resistor element.
#[derive(Debug, Clone)]
pub struct Resistor {
    /// 节点 A
    ///
    /// Node A
    pub a: NodeId,
    /// 节点 B
    ///
    /// Node B
    pub b: NodeId,
    /// 电阻值（单位：欧姆）
    ///
    /// Resistance (unit: ohms)
    pub ohms: f64,
}

impl Resistor {
    /// 创建新的电阻。
    ///
    /// # 参数
    ///
    /// * `a` - 节点 A
    /// * `b` - 节点 B
    /// * `ohms` - 电阻值（单位：欧姆，范围：0.0 ~ 1e9）
    ///
    /// ---
    ///
    /// Create a new resistor.
    ///
    /// # Arguments
    ///
    /// * `a` - Node A
    /// * `b` - Node B
    /// * `ohms` - Resistance (unit: ohms, range: 0.0 ~ 1e9)
    pub fn new(a: NodeId, b: NodeId, ohms: f64) -> Self {
        Self { a, b, ohms }
    }
}

/// 电容元件。
///
/// 使用 Backward Euler 离散化模型。
///
/// ---
///
/// Capacitor element.
///
/// Uses Backward Euler discretization model.
#[derive(Debug, Clone)]
pub struct Capacitor {
    /// 节点 A
    ///
    /// Node A
    pub a: NodeId,
    /// 节点 B
    ///
    /// Node B
    pub b: NodeId,
    /// 电容值（单位：法拉）
    ///
    /// Capacitance (unit: farads)
    pub farads: f64,
}

impl Capacitor {
    /// 创建新的电容。
    ///
    /// # 参数
    ///
    /// * `a` - 节点 A
    /// * `b` - 节点 B
    /// * `farads` - 电容值（单位：法拉，范围：1e-15 ~ 1.0）
    ///
    /// ---
    ///
    /// Create a new capacitor.
    ///
    /// # Arguments
    ///
    /// * `a` - Node A
    /// * `b` - Node B
    /// * `farads` - Capacitance (unit: farads, range: 1e-15 ~ 1.0)
    pub fn new(a: NodeId, b: NodeId, farads: f64) -> Self {
        Self { a, b, farads }
    }
}

/// 电感元件。
///
/// 使用 Backward Euler 离散化模型。
///
/// ---
///
/// Inductor element.
///
/// Uses Backward Euler discretization model.
#[derive(Debug, Clone)]
pub struct Inductor {
    /// 节点 A
    ///
    /// Node A
    pub a: NodeId,
    /// 节点 B
    ///
    /// Node B
    pub b: NodeId,
    /// 电感值（单位：亨利）
    ///
    /// Inductance (unit: henrys)
    pub henrys: f64,
    /// 上一采样周期的电流（内部状态）
    ///
    /// Previous sample current (internal state)
    pub(crate) i_prev: f64,
}

impl Inductor {
    /// 创建新的电感。
    ///
    /// # 参数
    ///
    /// * `a` - 节点 A
    /// * `b` - 节点 B
    /// * `henrys` - 电感值（单位：亨利，范围：1e-9 ~ 100.0）
    ///
    /// ---
    ///
    /// Create a new inductor.
    ///
    /// # Arguments
    ///
    /// * `a` - Node A
    /// * `b` - Node B
    /// * `henrys` - Inductance (unit: henrys, range: 1e-9 ~ 100.0)
    pub fn new(a: NodeId, b: NodeId, henrys: f64) -> Self {
        Self {
            a,
            b,
            henrys,
            i_prev: 0.0,
        }
    }
}

/// 双绕组耦合电感元件。
///
/// 用于模拟输出变压器等耦合电感。
///
/// ---
///
/// Dual-winding coupled inductor element.
///
/// Used for modeling output transformers and other coupled inductors.
#[derive(Debug, Clone)]
pub struct CoupledInductor {
    /// 初级绕组节点 A
    ///
    /// Primary winding node A
    pub p_a: NodeId,
    /// 初级绕组节点 B
    ///
    /// Primary winding node B
    pub p_b: NodeId,
    /// 次级绕组节点 A
    ///
    /// Secondary winding node A
    pub s_a: NodeId,
    /// 次级绕组节点 B
    ///
    /// Secondary winding node B
    pub s_b: NodeId,
    /// 初级电感值（单位：亨利）
    ///
    /// Primary inductance (unit: henrys)
    pub l_primary: f64,
    /// 次级电感值（单位：亨利）
    ///
    /// Secondary inductance (unit: henrys)
    pub l_secondary: f64,
    /// 耦合系数（范围：0.0 ~ 1.0，越大耦合越强，典型值 0.9 ~ 0.99）
    ///
    /// Coupling coefficient (range: 0.0 ~ 1.0, higher = stronger coupling, typical: 0.9 ~ 0.99)
    pub coupling: f64,
    /// 初级绕组上一采样周期的电流（内部状态）
    ///
    /// Primary winding previous sample current (internal state)
    pub(crate) i1_prev: f64,
    /// 次级绕组上一采样周期的电流（内部状态）
    ///
    /// Secondary winding previous sample current (internal state)
    pub(crate) i2_prev: f64,
}

impl CoupledInductor {
    /// 创建新的双绕组耦合电感。
    ///
    /// # 参数
    ///
    /// * `p_a` - 初级绕组节点 A
    /// * `p_b` - 初级绕组节点 B
    /// * `s_a` - 次级绕组节点 A
    /// * `s_b` - 次级绕组节点 B
    /// * `l_primary` - 初级电感值（单位：亨利，范围：1e-6 ~ 100.0）
    /// * `l_secondary` - 次级电感值（单位：亨利，范围：1e-6 ~ 100.0）
    /// * `coupling` - 耦合系数（范围：0.0 ~ 1.0，越大耦合越强，典型值 0.9 ~ 0.99）
    ///
    /// ---
    ///
    /// Create a new dual-winding coupled inductor.
    ///
    /// # Arguments
    ///
    /// * `p_a` - Primary winding node A
    /// * `p_b` - Primary winding node B
    /// * `s_a` - Secondary winding node A
    /// * `s_b` - Secondary winding node B
    /// * `l_primary` - Primary inductance (unit: henrys, range: 1e-6 ~ 100.0)
    /// * `l_secondary` - Secondary inductance (unit: henrys, range: 1e-6 ~ 100.0)
    /// * `coupling` - Coupling coefficient (range: 0.0 ~ 1.0, higher = stronger coupling, typical: 0.9 ~ 0.99)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        p_a: NodeId,
        p_b: NodeId,
        s_a: NodeId,
        s_b: NodeId,
        l_primary: f64,
        l_secondary: f64,
        coupling: f64,
    ) -> Self {
        Self {
            p_a,
            p_b,
            s_a,
            s_b,
            l_primary,
            l_secondary,
            coupling,
            i1_prev: 0.0,
            i2_prev: 0.0,
        }
    }
}

/// 三绕组耦合电感元件。
///
/// 用于模拟推挽输出变压器等具有中心抽头的变压器。
///
/// ---
///
/// Three-winding coupled inductor element.
///
/// Used for modeling center-tapped transformers like push-pull output transformers.
#[derive(Debug, Clone)]
pub struct CoupledInductor3 {
    /// 初级绕组 1 节点
    ///
    /// Primary winding 1 node
    pub p1: NodeId,
    /// 初级中心抽头节点
    ///
    /// Primary center tap node
    pub ct: NodeId,
    /// 初级绕组 2 节点
    ///
    /// Primary winding 2 node
    pub p2: NodeId,
    /// 次级绕组 1 节点
    ///
    /// Secondary winding 1 node
    pub s1: NodeId,
    /// 次级绕组 2 节点
    ///
    /// Secondary winding 2 node
    pub s2: NodeId,
    /// 绕组 1 电感值（单位：亨利）
    ///
    /// Winding 1 inductance (unit: henrys)
    pub l1: f64,
    /// 绕组 2 电感值（单位：亨利）
    ///
    /// Winding 2 inductance (unit: henrys)
    pub l2: f64,
    /// 绕组 3 电感值（单位：亨利）
    ///
    /// Winding 3 inductance (unit: henrys)
    pub l3: f64,
    /// 绕组 1-2 耦合系数
    ///
    /// Winding 1-2 coupling coefficient
    pub k12: f64,
    /// 绕组 1-3 耦合系数
    ///
    /// Winding 1-3 coupling coefficient
    pub k13: f64,
    /// 绕组 2-3 耦合系数
    ///
    /// Winding 2-3 coupling coefficient
    pub k23: f64,
    /// 绕组 1 上一采样周期的电流（内部状态）
    ///
    /// Winding 1 previous sample current (internal state)
    pub(crate) i1_prev: f64,
    /// 绕组 2 上一采样周期的电流（内部状态）
    ///
    /// Winding 2 previous sample current (internal state)
    pub(crate) i2_prev: f64,
    /// 绕组 3 上一采样周期的电流（内部状态）
    ///
    /// Winding 3 previous sample current (internal state)
    pub(crate) i3_prev: f64,
}

impl CoupledInductor3 {
    /// 创建新的三绕组耦合电感。
    ///
    /// # 参数
    ///
    /// * `p1` - 初级绕组 1 节点
    /// * `ct` - 初级中心抽头节点
    /// * `p2` - 初级绕组 2 节点
    /// * `s1` - 次级绕组 1 节点
    /// * `s2` - 次级绕组 2 节点
    /// * `l1` - 绕组 1 电感值（单位：亨利，范围：1e-6 ~ 100.0）
    /// * `l2` - 绕组 2 电感值（单位：亨利，范围：1e-6 ~ 100.0）
    /// * `l3` - 绕组 3 电感值（单位：亨利，范围：1e-6 ~ 100.0）
    /// * `k12` - 绕组 1-2 耦合系数（范围：0.0 ~ 1.0，越大耦合越强，典型值 0.9 ~ 0.99）
    /// * `k13` - 绕组 1-3 耦合系数（范围：0.0 ~ 1.0，越大耦合越强，典型值 0.9 ~ 0.99）
    /// * `k23` - 绕组 2-3 耦合系数（范围：0.0 ~ 1.0，越大耦合越强，典型值 0.9 ~ 0.99）
    ///
    /// ---
    ///
    /// Create a new three-winding coupled inductor.
    ///
    /// # Arguments
    ///
    /// * `p1` - Primary winding 1 node
    /// * `ct` - Primary center tap node
    /// * `p2` - Primary winding 2 node
    /// * `s1` - Secondary winding 1 node
    /// * `s2` - Secondary winding 2 node
    /// * `l1` - Winding 1 inductance (unit: henrys, range: 1e-6 ~ 100.0)
    /// * `l2` - Winding 2 inductance (unit: henrys, range: 1e-6 ~ 100.0)
    /// * `l3` - Winding 3 inductance (unit: henrys, range: 1e-6 ~ 100.0)
    /// * `k12` - Winding 1-2 coupling coefficient (range: 0.0 ~ 1.0, higher = stronger coupling, typical: 0.9 ~ 0.99)
    /// * `k13` - Winding 1-3 coupling coefficient (range: 0.0 ~ 1.0, higher = stronger coupling, typical: 0.9 ~ 0.99)
    /// * `k23` - Winding 2-3 coupling coefficient (range: 0.0 ~ 1.0, higher = stronger coupling, typical: 0.9 ~ 0.99)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        p1: NodeId,
        ct: NodeId,
        p2: NodeId,
        s1: NodeId,
        s2: NodeId,
        l1: f64,
        l2: f64,
        l3: f64,
        k12: f64,
        k13: f64,
        k23: f64,
    ) -> Self {
        Self {
            p1,
            ct,
            p2,
            s1,
            s2,
            l1,
            l2,
            l3,
            k12,
            k13,
            k23,
            i1_prev: 0.0,
            i2_prev: 0.0,
            i3_prev: 0.0,
        }
    }
}

/// 三极管实例。
///
/// ---
///
/// Triode instance.
#[derive(Debug, Clone)]
pub struct TriodeInstance {
    /// 屏极节点
    ///
    /// Plate node
    pub plate: NodeId,
    /// 栅极节点
    ///
    /// Grid node
    pub grid: NodeId,
    /// 阴极节点
    ///
    /// Cathode node
    pub cathode: NodeId,
    /// 参数索引（指向三极管参数表）
    ///
    /// Parameter index (into triode parameter table)
    pub params_idx: usize,
}

/// 二极管实例。
///
/// ---
///
/// Diode instance.
#[derive(Debug, Clone)]
pub struct DiodeInstance {
    /// 阳极节点
    ///
    /// Anode node
    pub anode: NodeId,
    /// 阴极节点
    ///
    /// Cathode node
    pub cathode: NodeId,
    /// 参数索引（指向二极管参数表）
    ///
    /// Parameter index (into diode parameter table)
    pub params_idx: usize,
}

/// 五极管实例。
///
/// ---
///
/// Pentode instance.
#[derive(Debug, Clone)]
pub struct PentodeInstance {
    /// 屏极节点
    ///
    /// Plate node
    pub plate: NodeId,
    /// 控制栅极节点
    ///
    /// Control grid node
    pub grid: NodeId,
    /// 阴极节点
    ///
    /// Cathode node
    pub cathode: NodeId,
    /// 帘栅极节点
    ///
    /// Screen grid node
    pub screen: NodeId,
    /// 参数索引（指向五极管参数表）
    ///
    /// Parameter index (into pentode parameter table)
    pub params_idx: usize,
}

/// 最大节点数。
///
/// 电路最多支持 30 个节点（包含接地节点）。
///
/// ---
///
/// Maximum number of nodes.
///
/// Circuit supports up to 30 nodes (including ground node).
pub const MAX_NODES: usize = 30;

/// 电路定义。
///
/// 包含完整的电路拓扑和参数，用于 MNA 求解器。
///
/// ---
///
/// Circuit definition.
///
/// Contains complete circuit topology and parameters for MNA solver.
#[derive(Debug, Clone)]
pub struct CircuitDef {
    /// 节点总数
    ///
    /// Total number of nodes
    pub num_nodes: usize,
    /// 电阻列表
    ///
    /// Resistor list
    pub resistors: Vec<Resistor>,
    /// 电容列表
    ///
    /// Capacitor list
    pub capacitors: Vec<Capacitor>,
    /// 电感列表
    ///
    /// Inductor list
    pub inductors: Vec<Inductor>,
    /// 双绕组耦合电感列表
    ///
    /// Dual-winding coupled inductor list
    pub coupled_inductors: Vec<CoupledInductor>,
    /// 三绕组耦合电感列表
    ///
    /// Three-winding coupled inductor list
    pub coupled_inductors3: Vec<CoupledInductor3>,
    /// 三极管实例列表
    ///
    /// Triode instance list
    pub triodes: Vec<TriodeInstance>,
    /// 五极管实例列表
    ///
    /// Pentode instance list
    pub pentodes: Vec<PentodeInstance>,
    /// 二极管实例列表
    ///
    /// Diode instance list
    pub diodes: Vec<DiodeInstance>,
    /// 输入信号节点
    ///
    /// Input signal node
    pub input_node: NodeId,
    /// 第二输入信号节点
    ///
    /// Second input signal node
    pub input2_node: NodeId,
    /// 第二输入信号电压
    ///
    /// Second input signal voltage
    pub input2_voltage: f64,
    /// 输出信号节点
    ///
    /// Output signal node
    pub output_node: NodeId,
    /// B+ 电源节点
    ///
    /// B+ power supply node
    pub bplus_node: NodeId,
    /// B+ 电源电压（单位：V）
    ///
    /// B+ voltage (unit: V)
    pub bplus_voltage: f64,
}
