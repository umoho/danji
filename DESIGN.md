# danji - 胆机模拟库 设计文档

## 1. 项目概述

### 1.1 项目定位
`danji` 是一个高精度物理建模的真空管放大器仿真库，使用 Rust 技术栈，提供同步 API 供上层应用调用。

### 1.2 核心目标
- **物理真实性**：基于 Child-Langmuir 定律和 Koren 改进模型
- **实时性**：支持 44.1kHz/96kHz/192kHz 音频采样率
- **可扩展性**：从单管电路逐步扩展到完整胆机

### 1.3 技术路线
- **建模视角**：宏观（端口特性），基于物理原理的代数方程
- **计算方法**：改进节点分析法 (MNA) + 牛顿-拉夫森迭代 + 后向欧拉法
- **加速方案**：当前 CPU 实现足够（192kHz 下单管 4x 实时），GPU 加速暂不需要

---

## 2. 当前实现状态

### 2.1 已实现

| 模块 | 文件 | 状态 |
|------|------|------|
| 三极管模型 (Koren) | `tube/triode.rs` | ✅ 9 管型 |
| 五极管模型 (Koren) | `tube/pentode.rs` | ✅ 5 管型 |
| 真空二极管模型 | `tube/diode.rs` | ✅ 4 管型 |
| 管型参数库 | `tube/params.rs` | ✅ 三极管 9 + 五极管 5 |
| 电阻/电容/电感元件 | `circuit/element.rs` | ✅ |
| 耦合电感 (变压器) | `circuit/element.rs` | ✅ 需数值阻尼 |
| MNA 求解器 | `circuit/solver.rs` | ✅ 高斯消元 + 牛顿迭代 |
| 仿真器封装 | `simulator.rs` | ✅ process_buffer/process_sample |
| 公共 API | `api.rs` | ✅ |
| DanjiError 错误类型 | `error.rs` | ✅ 6 种变体 |
| CLI 音频处理器 | `bin/danji-cli.rs` | ✅ WAV 输入输出 |

### 2.2 示例

| 示例 | 文件 | 描述 |
|------|------|------|
| 单管共阴极 | `examples/single_tube.rs` | 12AX7, 增益 62x |
| 级联双级 | `examples/two_stage.rs` | 12AX7×2 独立耦合, 9022x |
| 频响测量 | `examples/tone_control.rs` | 旁路/无旁路/小电容对比 |
| 电源滤波 | `examples/power_supply.rs` | 5AR4 + 单 C 滤波, 297V |
| CRC π 型滤波 | `examples/psu_clc.rs` | 100R + 2×47µF, 纹波 0.044V |
| 完整前级链 | `examples/full_chain.rs` | 电源→前级×2→音调→输出 |
| 输出变压器 | `examples/output_transformer.rs` | 12AU7 + OPT, 25:1 变比 |
| 五极管功率级 | `examples/pentode_stage.rs` | EL84 + 5kΩ 负载 |
| 性能基准 | `examples/benchmark.rs` | 多采样率测试 |
| 特性曲线 | `examples/plot_curves.rs` | 12AX7 曲线数据 |

### 2.3 单元测试

| 模块 | 测试数 | 覆盖 |
|------|--------|------|
| 三极管 | 9 | 截止/导通/单调性/偏置点/内阻/跨导/导数 |
| 二极管 | 5 | 反向/正向/伏安/电导/对比 |
| 五极管 | 4 | 板流/屏栅流/截止/导数 |

### 2.4 已知问题

| 问题 | 根因 | 影响 |
|------|------|------|
| **BE 电容短路** | 大电容在 BE 模型下 Gc=C/h 可达 1S (1Ω)，短接到地 | 22µF 屏栅旁路电容无法使用 |
| **BE 电感发散** | 无损耗电感器 BE 积分 I[n]=I[n-1]+h/L×V[n] 在 DC 下累积 | 10H 扼流圈需要外加阻尼 |
| **耦合电感 DC 累积** | 耦合电感的 BE 模型同样无 DC 损耗 | 需 Rdc 阻尼因子 |

---

## 3. 架构设计

### 3.1 模块结构

```
danji/
├── Cargo.toml
├── DESIGN.md
├── src/
│   ├── lib.rs                # 库入口
│   ├── bin/
│   │   └── danji-cli.rs      # CLI 音频处理器
│   ├── api.rs                # 公共 API
│   ├── error.rs              # DanjiError
│   ├── simulator.rs          # Simulator + SimConfig
│   ├── tube/
│   │   ├── mod.rs
│   │   ├── params.rs         # 三极管/五极管/二极管参数
│   │   ├── triode.rs         # 三极管 Koren 模型
│   │   ├── pentode.rs        # 五极管 Koren 模型
│   │   └── diode.rs          # 真空二极管模型
│   └── circuit/
│       ├── mod.rs
│       ├── node.rs           # NodeId
│       ├── element.rs        # R/C/L/耦合电感/三极管/五极管/二极管实例
│       └── solver.rs         # MNA 求解器
├── examples/
│   ├── single_tube.rs
│   ├── two_stage.rs
│   ├── tone_control.rs
│   ├── power_supply.rs
│   ├── psu_clc.rs
│   ├── full_chain.rs
│   ├── output_transformer.rs
│   ├── pentode_stage.rs
│   ├── benchmark.rs
│   ├── sr_bench.rs
│   └── plot_curves.rs
└── devlog/
    ├── 2026-06-24-initial-implementation.md
    ├── 2026-06-24-fixes-and-two-stage.md
    ├── 2026-06-24-diode-tone-psu.md
    ├── 2026-06-24-full-chain-psu.md
    ├── 2026-06-24-output-transformer.md
    └── 2026-06-24-pentode.md
```

### 3.2 核心类型

```rust
// 错误类型
#[derive(Debug)]
pub enum DanjiError {
    GpuInit(String),
    Diverged { sample: usize, iterations: usize },
    SingularMatrix { node: usize },
    InvalidCircuit(String),
    Numerical(String),
    BufferSize { expected: usize, actual: usize },
    InvalidParam(String),
}

// 三极管参数
pub struct TriodeParams { pub mu: f64, pub ex: f64, pub kg1: f64, pub kp: f64, pub kvb: f64 }

// 五极管参数 (额外 kg2)
pub struct PentodeParams { pub mu: f64, pub ex: f64, pub kg1: f64, pub kg2: f64, pub kp: f64, pub kvb: f64 }

// 二极管参数
pub struct DiodeParams { pub k: f64, pub gamma: f64 }

// 电路元件
pub struct Resistor { pub a: NodeId, pub b: NodeId, pub ohms: f64 }
pub struct Capacitor { pub a: NodeId, pub b: NodeId, pub farads: f64, pub(crate) v_prev: f64 }
pub struct Inductor { pub a: NodeId, pub b: NodeId, pub henrys: f64, pub(crate) i_prev: f64 }
pub struct CoupledInductor { pub p_a: NodeId, pub p_b: NodeId, pub s_a: NodeId, pub s_b: NodeId, ... }
pub struct TriodeInstance { pub plate: NodeId, pub grid: NodeId, pub cathode: NodeId, pub params_idx: usize }
pub struct PentodeInstance { pub plate: NodeId, pub grid: NodeId, pub cathode: NodeId, pub screen: NodeId, ... }
pub struct DiodeInstance { pub anode: NodeId, pub cathode: NodeId, pub params_idx: usize }

// 仿真器
pub struct Simulator { config: SimConfig, solver: CircuitSolver, ... }

// 电路配置 (builder 模式)
pub struct SimConfig { pub sample_rate: u32, pub num_nodes: usize, ... }
```

---

## 4. 物理模型

### 4.1 三极管 (Koren)

**中间变量 $E_1$：**
$$
E_1 = \frac{V_p}{k_P} \cdot \ln\left(1 + \exp\left(k_P \left(\frac{1}{\mu} + \frac{V_g}{\sqrt{k_{VB} + V_p^2}}\right)\right)\right)
$$

**阳极电流 $I_p$：**
$$
I_p = \frac{E_1^{E_x} + E_1 \cdot E_1^{E_x - 1}}{k_{G1}}
$$

### 4.2 五极管 (Koren)

**中间变量 $E_1$（使用屏栅电压 $V_{g2}$ 替代板压）：**
$$
E_1 = \frac{V_{g2}}{k_P} \cdot \ln\left(1 + \exp\left(k_P \left(\frac{1}{\mu} + \frac{V_{g1}}{V_{g2}}\right)\right)\right)
$$

**阳极电流（含 arctan 膝点）：**
$$
I_p = \frac{E_1^{E_x} + E_1 \cdot E_1^{E_x - 1}}{k_{G1}} \cdot \arctan\left(\frac{V_{pk}}{k_{VB}}\right)
$$

**屏栅电流：**
$$
I_{g2} = \frac{(V_{g1} + V_{g2}/\mu)^{3/2}}{k_{G2}}
$$

### 4.3 二极管 (Child-Langmuir)
$$
I = K \cdot V^{\gamma} \quad (V > 0), \quad I = 0 \quad (V \leq 0)
$$

### 4.4 动态元件 (后向欧拉)

**电容：**
$$
I_C[n] = \frac{C}{h} \cdot \left(V_C[n] - V_C[n-1]\right)
$$

**电感：**
$$
I_L[n] = I_L[n-1] + \frac{h}{L} \cdot V_L[n]
$$

**耦合电感：**
$$
\begin{bmatrix} I_1 \\ I_2 \end{bmatrix} = h \cdot \begin{bmatrix} L_2 & -M \\ -M & L_1 \end{bmatrix} / \det \cdot \begin{bmatrix} V_1 \\ V_2 \end{bmatrix} + \begin{bmatrix} I_{1,prev} \\ I_{2,prev} \end{bmatrix}
$$
其中 $M = k\sqrt{L_1 L_2}$, $\det = L_1 L_2 - M^2$

### 4.5 管型参数库

| 三极管 | $\mu$ | $E_x$ | $k_{G1}$ | $k_P$ | $k_{VB}$ |
|--------|-------|-------|----------|-------|----------|
| 12AX7 | 100 | 1.4 | 1060 | 600 | 300 |
| 12AU7 | 21.5 | 1.3 | 1180 | 84 | 300 |
| 12AT7 | 60 | 1.35 | 1200 | 200 | 300 |
| 6DJ8 | 28 | 1.3 | 330 | 320 | 300 |
| 6L6GC | 8.7 | 1.35 | 1460 | 48 | 12 |
| 6550 | 7.9 | 1.35 | 890 | 60 | 24 |
| EL34 | 10 | 1.35 | 1200 | 50 | 15 |
| KT88 | 8.8 | 1.35 | 730 | 32 | 16 |

| 五极管 | $\mu$ | $E_x$ | $k_{G1}$ | $k_{G2}$ | $k_P$ | $k_{VB}$ |
|--------|-------|-------|----------|----------|-------|----------|
| EL84 | 19 | 1.35 | 700 | 2000 | 100 | 20 |
| EL34 | 10 | 1.35 | 1200 | 4500 | 50 | 15 |
| 6L6GC | 8.7 | 1.35 | 1460 | 4500 | 48 | 12 |
| 6550 | 7.9 | 1.35 | 890 | 4200 | 60 | 24 |
| KT88 | 8.8 | 1.35 | 730 | 4200 | 32 | 16 |

| 整流管 | $K$ | $\gamma$ |
|--------|-------|-----|
| 5AR4/GZ34 | 0.005 | 1.5 |
| 5U4G | 0.003 | 1.5 |
| 6X4 | 0.002 | 1.5 |
| EZ81 | 0.004 | 1.5 |

### 4.6 求解器

- **线性部分**：高斯消元 + 部分主元消去，矩阵最大 30×30
- **非线性部分**：牛顿-拉夫森迭代（最大 50 次，容差 1e-9）
- **步长限制**：前 5 次迭代限制 50V，之后限制 200V
- **电压源**：大电导法（`VSRC_G = 1e6`）

---

## 5. 公共 API

```rust
impl Simulator {
    pub fn new(config: SimConfig, triode_params: Vec<TriodeParams>,
               pentode_params: Vec<PentodeParams>, diode_params: Vec<DiodeParams>) -> Self;
    pub fn process_buffer(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), DanjiError>;
    pub fn process_sample(&mut self, input: f32) -> Result<f32, DanjiError>;
    pub fn reset(&mut self);
    pub fn set_bplus(&mut self, voltage: f64);
    pub fn node_voltage(&self, node: NodeId) -> f32;
    pub fn sample_count(&self) -> usize;
}

impl SimConfig {
    pub fn new(sample_rate: u32, num_nodes: usize) -> Self;
    pub fn add_resistor(&mut self, a: NodeId, b: NodeId, ohms: f64) -> &mut Self;
    pub fn add_capacitor(&mut self, a: NodeId, b: NodeId, farads: f64) -> &mut Self;
    pub fn add_inductor(&mut self, a: NodeId, b: NodeId, henrys: f64) -> &mut Self;
    pub fn add_coupled_inductor(&mut self, p_a, p_b, s_a, s_b, l_primary, l_secondary, coupling) -> &mut Self;
    pub fn add_triode(&mut self, plate, grid, cathode, params_idx) -> &mut Self;
    pub fn add_pentode(&mut self, plate, grid, cathode, screen, params_idx) -> &mut Self;
    pub fn add_diode(&mut self, anode, cathode, params_idx) -> &mut Self;
    pub fn input(&mut self, node: NodeId) -> &mut Self;
    pub fn output(&mut self, node: NodeId) -> &mut Self;
    pub fn bplus(&mut self, node: NodeId, voltage: f64) -> &mut Self;
}
```

---

## 6. 性能基准

| 采样率 | 单管 (xRT) | 说明 |
|--------|-----------|------|
| 44.1kHz | 12.6x | 实时余量充裕 |
| 96kHz | 8.0x | |
| 192kHz | 4.0x | 单管仍可实时 |

---

## 7. 待实现

| 功能 | 优先级 | 说明 |
|------|--------|------|
| BE 数值稳定性修复 | 高 | 大电容短路 + 电感发散 + 耦合电感阻尼 |
| 推挽输出变压器 | 中 | 中心抽头初级 |
| WGSL GPU 加速 | 低 | CPU 实现足够 |
| 直接耦合多级 MNA | 低 | 当前独立级联架构可用 |
| 实时音频插件 | 低 | VST3/AU 封装 |

---

## 8. 参考资料

1. Pakarinen & Yeh (2009). "A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers"
2. Norman Koren. "Improved vacuum tube models for SPICE"
3. SwankyAmp. https://github.com/resonantdsp/SwankyAmp

---

*文档版本: 0.2.0*
*最后更新: 2026-06-24*
