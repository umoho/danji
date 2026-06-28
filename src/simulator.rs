use crate::circuit::element::{
    Capacitor, CircuitDef, CoupledInductor, CoupledInductor3, DiodeInstance, Inductor,
    PentodeInstance, Resistor, TriodeInstance,
};
use crate::circuit::node::NodeId;
use crate::circuit::solver::CircuitSolver;
use crate::error::DanjiError;
use crate::tube::diode::DiodeParams;
use crate::tube::params::PentodeParams;
use crate::tube::params::TriodeParams;
use log::{debug, info};

/// 仿真器核心结构。
///
/// 负责管理电路仿真状态，包括 MNA 求解器、真空管参数和动态元件状态。
/// 使用 [`SimConfig`] 构建电路配置，然后创建 `Simulator` 实例进行仿真。
///
/// ---
///
/// Core simulator structure.
///
/// Manages circuit simulation state including MNA solver, vacuum tube parameters,
/// and dynamic element states. Build circuit configuration with [`SimConfig`],
/// then create a `Simulator` instance for simulation.
pub struct Simulator {
    /// 电路配置
    ///
    /// Circuit configuration
    config: SimConfig,
    /// MNA 求解器
    ///
    /// MNA solver
    solver: CircuitSolver,
    /// 三极管参数表
    ///
    /// Triode parameter table
    triode_params: Vec<TriodeParams>,
    /// 五极管参数表
    ///
    /// Pentode parameter table
    pentode_params: Vec<PentodeParams>,
    /// 二极管参数表
    ///
    /// Diode parameter table
    diode_params: Vec<DiodeParams>,
    /// 已处理的采样点计数
    ///
    /// Number of processed samples
    sample_count: usize,
}

/// 电路仿真配置。
///
/// 使用构建器模式配置电路参数，包括采样率、节点数、电路元件和真空管实例。
///
/// ---
///
/// Circuit simulation configuration.
///
/// Configure circuit parameters using builder pattern, including sample rate,
/// node count, circuit elements, and vacuum tube instances.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// 采样率（单位：Hz，范围：8000 ~ 192000）
    ///
    /// Sample rate (unit: Hz, range: 8000 ~ 192000)
    pub sample_rate: u32,

    /// 电路节点总数（包含接地节点，范围：1 ~ 30）
    ///
    /// Total number of circuit nodes (including ground node, range: 1 ~ 30)
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

    /// 第二输入信号节点（用于推挽等场景）
    ///
    /// Second input signal node (for push-pull etc.)
    pub(crate) input2_node: NodeId,

    /// 第二输入信号电压
    ///
    /// Second input signal voltage
    pub(crate) input2_voltage: f64,

    /// 输出信号节点
    ///
    /// Output signal node
    pub output_node: NodeId,

    /// B+ 电源节点
    ///
    /// B+ power supply node
    pub bplus_node: NodeId,

    /// B+ 电源电压（单位：V，范围：0 ~ 500）
    ///
    /// B+ voltage (unit: V, range: 0 ~ 500)
    pub bplus_voltage: f64,
}

impl SimConfig {
    /// 创建新的电路配置。
    ///
    /// # 参数
    ///
    /// * `sample_rate` - 采样率（单位：Hz，范围：8000 ~ 192000）
    /// * `num_nodes` - 电路节点总数，包含接地节点（范围：1 ~ 30）
    ///
    /// # 返回值
    ///
    /// 返回空的电路配置，可通过链式调用添加元件
    ///
    /// ---
    ///
    /// Create a new circuit configuration.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate (unit: Hz, range: 8000 ~ 192000)
    /// * `num_nodes` - Total circuit nodes including ground node (range: 1 ~ 30)
    ///
    /// # Returns
    ///
    /// Returns an empty circuit configuration, ready for chaining element additions
    pub fn new(sample_rate: u32, num_nodes: usize) -> Self {
        Self {
            sample_rate,
            num_nodes,
            resistors: Vec::new(),
            capacitors: Vec::new(),
            inductors: Vec::new(),
            coupled_inductors: Vec::new(),
            coupled_inductors3: Vec::new(),
            triodes: Vec::new(),
            pentodes: Vec::new(),
            diodes: Vec::new(),
            input_node: NodeId(0),
            input2_node: NodeId(0),
            input2_voltage: 0.0,
            output_node: NodeId(0),
            bplus_node: NodeId(0),
            bplus_voltage: 0.0,
        }
    }

    /// 添加电阻元件。
    ///
    /// 在节点 `a` 和 `b` 之间添加一个电阻值为 `ohms` 的电阻。
    ///
    /// # 参数
    ///
    /// * `a` - 节点 A
    /// * `b` - 节点 B
    /// * `ohms` - 电阻值（单位：欧姆，范围：0.0 ~ 1e9）
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Add a resistor element.
    ///
    /// Adds a resistor with value `ohms` between nodes `a` and `b`.
    ///
    /// # Arguments
    ///
    /// * `a` - Node A
    /// * `b` - Node B
    /// * `ohms` - Resistance value (unit: ohms, range: 0.0 ~ 1e9)
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn add_resistor(&mut self, a: NodeId, b: NodeId, ohms: f64) -> &mut Self {
        self.resistors.push(Resistor::new(a, b, ohms));
        self
    }

    /// 添加电容元件。
    ///
    /// 在节点 `a` 和 `b` 之间添加一个电容值为 `farads` 的电容。
    /// 使用 Backward Euler 离散化模型。
    ///
    /// # 参数
    ///
    /// * `a` - 节点 A
    /// * `b` - 节点 B
    /// * `farads` - 电容值（单位：法拉，范围：1e-15 ~ 1.0）
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Add a capacitor element.
    ///
    /// Adds a capacitor with value `farads` between nodes `a` and `b`.
    /// Uses Backward Euler discretization model.
    ///
    /// # Arguments
    ///
    /// * `a` - Node A
    /// * `b` - Node B
    /// * `farads` - Capacitance (unit: farads, range: 1e-15 ~ 1.0)
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn add_capacitor(&mut self, a: NodeId, b: NodeId, farads: f64) -> &mut Self {
        self.capacitors.push(Capacitor::new(a, b, farads));
        self
    }

    /// 添加电感元件。
    ///
    /// 在节点 `a` 和 `b` 之间添加一个电感值为 `henrys` 的电感。
    /// 使用 Backward Euler 离散化模型。
    ///
    /// # 参数
    ///
    /// * `a` - 节点 A
    /// * `b` - 节点 B
    /// * `henrys` - 电感值（单位：亨利，范围：1e-9 ~ 100.0）
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Add an inductor element.
    ///
    /// Adds an inductor with value `henrys` between nodes `a` and `b`.
    /// Uses Backward Euler discretization model.
    ///
    /// # Arguments
    ///
    /// * `a` - Node A
    /// * `b` - Node B
    /// * `henrys` - Inductance (unit: henrys, range: 1e-9 ~ 100.0)
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn add_inductor(&mut self, a: NodeId, b: NodeId, henrys: f64) -> &mut Self {
        self.inductors.push(Inductor::new(a, b, henrys));
        self
    }

    /// 添加双绕组耦合电感元件。
    ///
    /// 添加一个双绕组耦合电感（如输出变压器），初级和次级绕组分别连接在指定节点对之间。
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
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Add a coupled inductor element.
    ///
    /// Adds a dual-winding coupled inductor (e.g., output transformer), with
    /// primary and secondary windings connected between specified node pairs.
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
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    #[allow(clippy::too_many_arguments)]
    pub fn add_coupled_inductor(
        &mut self,
        p_a: NodeId,
        p_b: NodeId,
        s_a: NodeId,
        s_b: NodeId,
        l_primary: f64,
        l_secondary: f64,
        coupling: f64,
    ) -> &mut Self {
        self.coupled_inductors.push(CoupledInductor::new(
            p_a,
            p_b,
            s_a,
            s_b,
            l_primary,
            l_secondary,
            coupling,
        ));
        self
    }

    /// 添加三绕组耦合电感元件。
    ///
    /// 添加一个三绕组耦合电感（如推挽输出变压器），用于模拟具有中心抽头的变压器。
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
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Add a three-winding coupled inductor element.
    ///
    /// Adds a three-winding coupled inductor (e.g., push-pull output transformer)
    /// for modeling center-tapped transformers.
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
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    #[allow(clippy::too_many_arguments)]
    pub fn add_coupled_inductor3(
        &mut self,
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
    ) -> &mut Self {
        self.coupled_inductors3.push(CoupledInductor3::new(
            p1, ct, p2, s1, s2, l1, l2, l3, k12, k13, k23,
        ));
        self
    }

    /// 添加三极管实例。
    ///
    /// 添加一个三极管，其屏极、栅极和阴极分别连接到指定节点。
    /// `params_idx` 索引三极管参数表中的参数。
    ///
    /// # 参数
    ///
    /// * `plate` - 屏极节点
    /// * `grid` - 栅极节点
    /// * `cathode` - 阴极节点
    /// * `params_idx` - 参数索引（指向 [`Simulator`] 中的三极管参数表）
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Add a triode instance.
    ///
    /// Adds a triode with plate, grid, and cathode connected to specified nodes.
    /// `params_idx` indexes into the triode parameter table.
    ///
    /// # Arguments
    ///
    /// * `plate` - Plate (anode) node
    /// * `grid` - Grid node
    /// * `cathode` - Cathode node
    /// * `params_idx` - Parameter index (into the triode parameter table in [`Simulator`])
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn add_triode(
        &mut self,
        plate: NodeId,
        grid: NodeId,
        cathode: NodeId,
        params_idx: usize,
    ) -> &mut Self {
        self.triodes.push(TriodeInstance {
            plate,
            grid,
            cathode,
            params_idx,
        });
        self
    }

    /// 添加五极管实例。
    ///
    /// 添加一个五极管，其屏极、栅极、阴极和帘栅极分别连接到指定节点。
    /// `params_idx` 索引五极管参数表中的参数。
    ///
    /// # 参数
    ///
    /// * `plate` - 屏极节点
    /// * `grid` - 控制栅极节点
    /// * `cathode` - 阴极节点
    /// * `screen` - 帘栅极节点
    /// * `params_idx` - 参数索引（指向 [`Simulator`] 中的五极管参数表）
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Add a pentode instance.
    ///
    /// Adds a pentode with plate, grid, cathode, and screen connected to specified nodes.
    /// `params_idx` indexes into the pentode parameter table.
    ///
    /// # Arguments
    ///
    /// * `plate` - Plate (anode) node
    /// * `grid` - Control grid node
    /// * `cathode` - Cathode node
    /// * `screen` - Screen grid node
    /// * `params_idx` - Parameter index (into the pentode parameter table in [`Simulator`])
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn add_pentode(
        &mut self,
        plate: NodeId,
        grid: NodeId,
        cathode: NodeId,
        screen: NodeId,
        params_idx: usize,
    ) -> &mut Self {
        self.pentodes.push(PentodeInstance {
            plate,
            grid,
            cathode,
            screen,
            params_idx,
        });
        self
    }

    /// 添加二极管实例。
    ///
    /// 添加一个二极管，其阳极和阴极分别连接到指定节点。
    /// `params_idx` 索引二极管参数表中的参数。
    ///
    /// # 参数
    ///
    /// * `anode` - 阳极节点
    /// * `cathode` - 阴极节点
    /// * `params_idx` - 参数索引（指向 [`Simulator`] 中的二极管参数表）
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Add a diode instance.
    ///
    /// Adds a diode with anode and cathode connected to specified nodes.
    /// `params_idx` indexes into the diode parameter table.
    ///
    /// # Arguments
    ///
    /// * `anode` - Anode node
    /// * `cathode` - Cathode node
    /// * `params_idx` - Parameter index (into the diode parameter table in [`Simulator`])
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn add_diode(&mut self, anode: NodeId, cathode: NodeId, params_idx: usize) -> &mut Self {
        self.diodes.push(DiodeInstance {
            anode,
            cathode,
            params_idx,
        });
        self
    }

    /// 设置输入信号节点。
    ///
    /// # 参数
    ///
    /// * `node` - 输入信号节点
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Set the input signal node.
    ///
    /// # Arguments
    ///
    /// * `node` - Input signal node
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn input(&mut self, node: NodeId) -> &mut Self {
        self.input_node = node;
        self
    }

    /// 设置输出信号节点。
    ///
    /// # 参数
    ///
    /// * `node` - 输出信号节点
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Set the output signal node.
    ///
    /// # Arguments
    ///
    /// * `node` - Output signal node
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn output(&mut self, node: NodeId) -> &mut Self {
        self.output_node = node;
        self
    }

    /// 设置 B+ 电源节点和电压。
    ///
    /// # 参数
    ///
    /// * `node` - B+ 电源节点
    /// * `voltage` - B+ 电源电压（单位：V，范围：0 ~ 500）
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Set the B+ power supply node and voltage.
    ///
    /// # Arguments
    ///
    /// * `node` - B+ power supply node
    /// * `voltage` - B+ voltage (unit: V, range: 0 ~ 500)
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    pub fn bplus(&mut self, node: NodeId, voltage: f64) -> &mut Self {
        self.bplus_node = node;
        self.bplus_voltage = voltage;
        self
    }

    /// 设置第二输入信号节点。
    ///
    /// 用于推挽电路等需要两个输入信号的场景。
    ///
    /// # 参数
    ///
    /// * `node` - 第二输入信号节点
    ///
    /// # 返回值
    ///
    /// 返回自身引用，支持链式调用
    ///
    /// ---
    ///
    /// Set the second input signal node.
    ///
    /// Used for push-pull circuits and other scenarios requiring two input signals.
    ///
    /// # Arguments
    ///
    /// * `node` - Second input signal node
    ///
    /// # Returns
    ///
    /// Returns self reference for method chaining
    #[allow(dead_code)]
    pub fn input2(&mut self, node: NodeId) -> &mut Self {
        self.input2_node = node;
        self
    }

    /// 将配置转换为内部电路定义。
    fn to_circuit_def(&self) -> CircuitDef {
        CircuitDef {
            num_nodes: self.num_nodes,
            resistors: self.resistors.clone(),
            capacitors: self.capacitors.clone(),
            inductors: self.inductors.clone(),
            coupled_inductors: self.coupled_inductors.clone(),
            coupled_inductors3: self.coupled_inductors3.clone(),
            triodes: self.triodes.clone(),
            pentodes: self.pentodes.clone(),
            diodes: self.diodes.clone(),
            input_node: self.input_node,
            input2_node: self.input2_node,
            input2_voltage: self.input2_voltage,
            output_node: self.output_node,
            bplus_node: self.bplus_node,
            bplus_voltage: self.bplus_voltage,
        }
    }
}

impl Simulator {
    /// 创建新的仿真器实例。
    ///
    /// # 参数
    ///
    /// * `config` - 电路配置
    /// * `triode_params` - 三极管参数表
    /// * `pentode_params` - 五极管参数表
    /// * `diode_params` - 二极管参数表
    ///
    /// # 返回值
    ///
    /// 返回初始化后的仿真器实例
    ///
    /// ---
    ///
    /// Create a new simulator instance.
    ///
    /// # Arguments
    ///
    /// * `config` - Circuit configuration
    /// * `triode_params` - Triode parameter table
    /// * `pentode_params` - Pentode parameter table
    /// * `diode_params` - Diode parameter table
    ///
    /// # Returns
    ///
    /// Returns initialized simulator instance
    pub fn new(
        config: SimConfig,
        triode_params: Vec<TriodeParams>,
        pentode_params: Vec<PentodeParams>,
        diode_params: Vec<DiodeParams>,
    ) -> Self {
        let solver = CircuitSolver::new(config.num_nodes);
        info!(
            "create simulator: {} nodes, {} R, {} C, {} L, {} CL, {} CL3, {} triodes, {} pentodes, {} diodes",
            config.num_nodes,
            config.resistors.len(),
            config.capacitors.len(),
            config.inductors.len(),
            config.coupled_inductors.len(),
            config.coupled_inductors3.len(),
            config.triodes.len(),
            config.pentodes.len(),
            config.diodes.len(),
        );
        Self {
            config,
            solver,
            triode_params,
            pentode_params,
            diode_params,
            sample_count: 0,
        }
    }

    /// 重置仿真器状态。
    ///
    /// 将所有动态元件（电容、电感）的内部状态重置为零，
    /// 并将采样计数器清零。用于重新开始仿真。
    ///
    /// ---
    ///
    /// Reset simulator state.
    ///
    /// Resets all dynamic element (capacitor, inductor) internal states to zero,
    /// and clears the sample counter. Used to restart simulation.
    pub fn reset(&mut self) {
        debug!("simulator reset after {} samples", self.sample_count);
        self.solver.reset();
        for ind in &mut self.config.inductors {
            ind.i_prev = 0.0;
        }
        for ci in &mut self.config.coupled_inductors {
            ci.i1_prev = 0.0;
            ci.i2_prev = 0.0;
        }
        self.sample_count = 0;
    }

    /// 处理双输入采样点。
    ///
    /// 用于推挽电路等需要两个输入信号的场景。
    ///
    /// # 参数
    ///
    /// * `input1` - 第一输入信号（单位：V，范围：-1.0 ~ 1.0）
    /// * `input2` - 第二输入信号（单位：V，范围：-1.0 ~ 1.0）
    ///
    /// # 返回值
    ///
    /// 返回输出信号电压（单位：V）
    ///
    /// # Panics
    ///
    /// 当迭代发散时返回 `Err(DanjiError::Diverged)`
    ///
    /// ---
    ///
    /// Process a sample with dual inputs.
    ///
    /// Used for push-pull circuits and other scenarios requiring two input signals.
    ///
    /// # Arguments
    ///
    /// * `input1` - First input signal (unit: V, range: -1.0 ~ 1.0)
    /// * `input2` - Second input signal (unit: V, range: -1.0 ~ 1.0)
    ///
    /// # Returns
    ///
    /// Returns output signal voltage (unit: V)
    ///
    /// # Panics
    ///
    /// Returns `Err(DanjiError::Diverged)` when iteration diverges
    pub fn process_sample_dual(&mut self, input1: f32, input2: f64) -> Result<f32, DanjiError> {
        let prev = self.config.input2_voltage;
        self.config.input2_voltage = input2;
        let result = self.process_sample(input1);
        self.config.input2_voltage = if result.is_ok() { 0.0 } else { prev };
        result
    }

    /// 处理单个采样点。
    ///
    /// 对输入信号进行一个采样周期的仿真，更新所有动态元件状态，
    /// 并返回输出节点的电压值。
    ///
    /// # 参数
    ///
    /// * `input` - 输入信号（单位：V，范围：-1.0 ~ 1.0）
    ///
    /// # 返回值
    ///
    /// 返回输出信号电压（单位：V）
    ///
    /// # Panics
    ///
    /// 当 Newton-Raphson 迭代发散时返回 `Err(DanjiError::Diverged)`
    ///
    /// ---
    ///
    /// Process a single sample.
    ///
    /// Performs one sample period of simulation on the input signal,
    /// updates all dynamic element states, and returns the output node voltage.
    ///
    /// # Arguments
    ///
    /// * `input` - Input signal (unit: V, range: -1.0 ~ 1.0)
    ///
    /// # Returns
    ///
    /// Returns output signal voltage (unit: V)
    ///
    /// # Panics
    ///
    /// Returns `Err(DanjiError::Diverged)` when Newton-Raphson iteration diverges
    pub fn process_sample(&mut self, input: f32) -> Result<f32, DanjiError> {
        let fs = self.config.sample_rate as f64;
        let h = 1.0 / fs;
        let circuit_def = self.config.to_circuit_def();

        self.solver.solve(
            &circuit_def,
            &self.triode_params,
            &self.pentode_params,
            &self.diode_params,
            h,
            input as f64,
        )?;

        for ind in &mut self.config.inductors {
            let a = ind.a.0;
            let b = ind.b.0;
            let v_a = if a > 0 { self.solver.v[a] } else { 0.0 };
            let v_b = if b > 0 { self.solver.v[b] } else { 0.0 };
            let gl = ind.henrys.recip() * h;
            ind.i_prev += gl * (v_a - v_b);
            // BE DC leakage: prevents DC accumulation in lossless BE model
            // Use ~100Ω DCR as damping, not the full reflected load
            let damping = (h * 100.0 / ind.henrys).min(0.5);
            ind.i_prev *= 1.0 - damping;
        }

        for ci in &mut self.config.coupled_inductors {
            let (pa, pb) = (ci.p_a.0, ci.p_b.0);
            let (sa, sb) = (ci.s_a.0, ci.s_b.0);
            let v_p = if pa > 0 { self.solver.v[pa] } else { 0.0 }
                - if pb > 0 { self.solver.v[pb] } else { 0.0 };
            let v_s = if sa > 0 { self.solver.v[sa] } else { 0.0 }
                - if sb > 0 { self.solver.v[sb] } else { 0.0 };
            let m = ci.coupling * (ci.l_primary * ci.l_secondary).sqrt();
            let det = ci.l_primary * ci.l_secondary - m * m;
            if det <= 0.0 {
                continue;
            }
            let g11 = h * ci.l_secondary / det;
            let g22 = h * ci.l_primary / det;
            let g12 = -h * m / det;
            // BE with Rdc damping: prevents unbounded DC accumulation
            // dI/dt = V/L - R/L * I  →  I[n] = (1-hR/L)*I[n-1] + h/L*V[n]
            let rdc_pri = 150.0;
            let rdc_sec = 0.5;
            let d1 = h * rdc_pri / ci.l_primary;
            let d2 = h * rdc_sec / ci.l_secondary;
            ci.i1_prev = (1.0 - d1) * ci.i1_prev + g11 * v_p + g12 * v_s;
            ci.i2_prev = (1.0 - d2) * ci.i2_prev + g12 * v_p + g22 * v_s;
        }

        for ci in &mut self.config.coupled_inductors3 {
            let v = [
                if ci.p1.0 > 0 {
                    self.solver.v[ci.p1.0]
                } else {
                    0.0
                } - if ci.ct.0 > 0 {
                    self.solver.v[ci.ct.0]
                } else {
                    0.0
                },
                if ci.p2.0 > 0 {
                    self.solver.v[ci.p2.0]
                } else {
                    0.0
                } - if ci.ct.0 > 0 {
                    self.solver.v[ci.ct.0]
                } else {
                    0.0
                },
                if ci.s1.0 > 0 {
                    self.solver.v[ci.s1.0]
                } else {
                    0.0
                } - if ci.s2.0 > 0 {
                    self.solver.v[ci.s2.0]
                } else {
                    0.0
                },
            ];
            let ls = [ci.l1, ci.l2, ci.l3];
            let m12 = ci.k12 * (ls[0] * ls[1]).sqrt();
            let m13 = ci.k13 * (ls[0] * ls[2]).sqrt();
            let m23 = ci.k23 * (ls[1] * ls[2]).sqrt();
            let det = ls[0] * ls[1] * ls[2] + 2.0 * m12 * m13 * m23
                - ls[0] * m23 * m23
                - ls[1] * m13 * m13
                - ls[2] * m12 * m12;
            if det <= 1e-30 {
                continue;
            }
            let y00 = h * (ls[1] * ls[2] - m23 * m23) / det;
            let y11 = h * (ls[0] * ls[2] - m13 * m13) / det;
            let y22 = h * (ls[0] * ls[1] - m12 * m12) / det;
            let y01 = h * (m13 * m23 - ls[2] * m12) / det;
            let y02 = h * (m12 * m23 - ls[1] * m13) / det;
            let y12 = h * (m12 * m13 - ls[0] * m23) / det;
            let d = (h * 150.0 / ls[0]).min(0.5); // primary DCR damping
            ci.i1_prev = (1.0 - d) * ci.i1_prev + y00 * v[0] + y01 * v[1] + y02 * v[2];
            ci.i2_prev = (1.0 - d) * ci.i2_prev + y01 * v[0] + y11 * v[1] + y12 * v[2];
            ci.i3_prev = (1.0 - d) * ci.i3_prev + y02 * v[0] + y12 * v[1] + y22 * v[2];
        }

        self.sample_count += 1;

        let out = self.config.output_node.0;
        Ok(if out > 0 {
            self.solver.v[out] as f32
        } else {
            0.0
        })
    }

    /// 处理音频缓冲区。
    ///
    /// 逐采样处理输入缓冲区，将结果写入输出缓冲区。
    ///
    /// # 参数
    ///
    /// * `input` - 输入缓冲区
    /// * `output` - 输出缓冲区（长度必须与输入相同）
    ///
    /// # 返回值
    ///
    /// 成功返回 `()`，缓冲区长度不匹配时返回 `Err(DanjiError::BufferSize)`
    ///
    /// # Panics
    ///
    /// 当输入缓冲区长度与输出缓冲区长度不同时 panic
    ///
    /// ---
    ///
    /// Process an audio buffer.
    ///
    /// Processes input buffer sample by sample, writing results to output buffer.
    ///
    /// # Arguments
    ///
    /// * `input` - Input buffer
    /// * `output` - Output buffer (must be same length as input)
    ///
    /// # Returns
    ///
    /// Returns `()` on success, `Err(DanjiError::BufferSize)` if buffer lengths don't match
    ///
    /// # Panics
    ///
    /// Panics when input buffer length differs from output buffer length
    pub fn process_buffer(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), DanjiError> {
        if input.len() != output.len() {
            return Err(DanjiError::BufferSize {
                expected: input.len(),
                actual: output.len(),
            });
        }
        for (i, sample) in output.iter_mut().enumerate() {
            *sample = self.process_sample(input[i])?;
        }
        Ok(())
    }

    /// 获取电路节点总数。
    ///
    /// ---
    ///
    /// Get the total number of circuit nodes.
    pub fn num_nodes(&self) -> usize {
        self.config.num_nodes
    }

    /// 获取已处理的采样点计数。
    ///
    /// ---
    ///
    /// Get the number of processed samples.
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// 设置 B+ 电源电压。
    ///
    /// # 参数
    ///
    /// * `voltage` - B+ 电源电压（单位：V，范围：0 ~ 500）
    ///
    /// ---
    ///
    /// Set the B+ power supply voltage.
    ///
    /// # Arguments
    ///
    /// * `voltage` - B+ voltage (unit: V, range: 0 ~ 500)
    pub fn set_bplus(&mut self, voltage: f64) {
        self.config.bplus_voltage = voltage;
    }

    /// 设置第二输入信号电压。
    ///
    /// # 参数
    ///
    /// * `voltage` - 第二输入信号电压（单位：V，范围：-1.0 ~ 1.0）
    ///
    /// ---
    ///
    /// Set the second input signal voltage.
    ///
    /// # Arguments
    ///
    /// * `voltage` - Second input signal voltage (unit: V, range: -1.0 ~ 1.0)
    pub fn set_input2(&mut self, voltage: f64) {
        self.config.input2_voltage = voltage;
    }

    /// 获取指定节点的电压。
    ///
    /// # 参数
    ///
    /// * `node` - 节点标识
    ///
    /// # 返回值
    ///
    /// 返回节点电压（单位：V），节点不存在时返回 0.0
    ///
    /// ---
    ///
    /// Get the voltage at a specified node.
    ///
    /// # Arguments
    ///
    /// * `node` - Node identifier
    ///
    /// # Returns
    ///
    /// Returns node voltage (unit: V), returns 0.0 if node doesn't exist
    pub fn node_voltage(&self, node: NodeId) -> f32 {
        let n = node.0;
        if n < self.solver.v.len() {
            self.solver.v[n] as f32
        } else {
            0.0
        }
    }
}

/// 创建单三极管共阴极放大电路配置。
///
/// 生成一个典型的单三极管共阴极放大器电路配置，包含：
/// - 屏极负载电阻
/// - 阴极电阻（带旁路电容）
/// - 栅极电阻
/// - B+ 电源
///
/// # 参数
///
/// * `sample_rate` - 采样率（单位：Hz，范围：8000 ~ 192000）
/// * `plate_resistor` - 屏极负载电阻（单位：欧姆，范围：1e3 ~ 500e3）
/// * `cathode_resistor` - 阴极电阻（单位：欧姆，范围：100 ~ 10e3）
/// * `cathode_capacitor` - 阴极旁路电容（单位：法拉，范围：0.0 ~ 1.0，0 表示无旁路电容）
/// * `grid_resistor` - 栅极电阻（单位：欧姆，范围：1e3 ~ 1e6）
/// * `bplus` - B+ 电源电压（单位：V，范围：50 ~ 500）
///
/// # 返回值
///
/// 返回配置好的电路配置，可直接用于创建 `Simulator`
///
/// ---
///
/// Create a single triode common-cathode amplifier circuit configuration.
///
/// Generates a typical single triode common-cathode amplifier configuration:
/// - Plate load resistor
/// - Cathode resistor (with bypass capacitor)
/// - Grid resistor
/// - B+ power supply
///
/// # Arguments
///
/// * `sample_rate` - Sample rate (unit: Hz, range: 8000 ~ 192000)
/// * `plate_resistor` - Plate load resistor (unit: ohms, range: 1e3 ~ 500e3)
/// * `cathode_resistor` - Cathode resistor (unit: ohms, range: 100 ~ 10e3)
/// * `cathode_capacitor` - Cathode bypass capacitor (unit: farads, range: 0.0 ~ 1.0, 0 = no bypass)
/// * `grid_resistor` - Grid resistor (unit: ohms, range: 1e3 ~ 1e6)
/// * `bplus` - B+ voltage (unit: V, range: 50 ~ 500)
///
/// # Returns
///
/// Returns configured circuit configuration, ready for creating `Simulator`
pub fn single_triode_config(
    sample_rate: u32,
    plate_resistor: f64,
    cathode_resistor: f64,
    cathode_capacitor: f64,
    grid_resistor: f64,
    bplus: f64,
) -> SimConfig {
    use crate::circuit::node::NodeId;
    let gnd = NodeId(0);
    let grid = NodeId(1);
    let cathode = NodeId(2);
    let plate = NodeId(3);
    let bplus_node = NodeId(4);

    let mut config = SimConfig::new(sample_rate, 5);
    config
        .resistors
        .push(Resistor::new(plate, bplus_node, plate_resistor));
    config
        .resistors
        .push(Resistor::new(cathode, gnd, cathode_resistor));
    config
        .resistors
        .push(Resistor::new(grid, gnd, grid_resistor));
    config.resistors.push(Resistor::new(bplus_node, gnd, 1e6));
    if cathode_capacitor > 0.0 {
        config
            .capacitors
            .push(Capacitor::new(cathode, gnd, cathode_capacitor));
    }
    config.triodes.push(TriodeInstance {
        plate,
        grid,
        cathode,
        params_idx: 0,
    });
    config.input_node = grid;
    config.output_node = plate;
    config.bplus_node = bplus_node;
    config.bplus_voltage = bplus;
    config
}
