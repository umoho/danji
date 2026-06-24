# danji - 胆机模拟库 设计文档

## 1. 项目概述

### 1.1 项目定位
`danji` 是一个高精度物理建模的真空管放大器仿真库，使用 Rust + WGSL 技术栈，提供同步 API 供上层应用调用。

### 1.2 核心目标
- **物理真实性**：基于 Child-Langmuir 定律和 Koren 改进模型
- **实时性**：支持 44.1kHz/48kHz 音频采样率
- **可扩展性**：从单管电路逐步扩展到完整胆机

### 1.3 技术路线
- **建模视角**：宏观（端口特性），基于物理原理的代数方程
- **计算方法**：改进节点分析法 (MNA) + 牛顿-拉夫森迭代
- **加速方案**：GPU 并行计算（wgpu + WGSL）

---

## 2. 依赖分析

### 2.1 核心依赖

| Crate | 版本 | 用途 | 备注 |
|-------|------|------|------|
| `wgpu` | 22.x | GPU 计算 | 跨平台，支持 Vulkan/Metal/DX12 |
| `pollster` | 0.3 | 阻塞 async | 简化 wgpu 异步调用 |
| `bytemuck` | 1.x | 字节转换 | 安全的类型转换 |
| `thiserror` | 2.x | 错误定义 | 派生宏定义错误类型 |
| `log` | 0.4 | 日志 | 通用日志接口 |

### 2.2 可选依赖

| Crate | 版本 | 用途 | 场景 |
|-------|------|------|------|
| `nalgebra` | 0.33 | 线性代数 | CPU 端矩阵运算（备用） |
| `serde` | 1.x | 序列化 | 配置文件支持 |
| `env_logger` | 0.11 | 日志实现 | 开发/示例用 |

### 2.3 开发依赖

| Crate | 版本 | 用途 |
|-------|------|------|
| `cpal` | 0.15 | 音频 I/O（示例） |
| `plotters` | 0.3 | 绘图（测试验证） |

### 2.4 依赖决策

**选择 `wgpu` 而非 `rust-gpu` 的原因：**
- `wgpu` 是 WebGPU 标准的 Rust 实现，跨平台
- `rust-gpu` 需要特殊工具链，编译复杂
- WGSL 是 WebGPU 标准着色器语言，社区支持好

**不引入 `nalgebra` 的原因：**
- 初期电路规模小（<10 节点），手写高斯消元足够
- 减少依赖，加快编译
- 后期可按需引入

---

## 3. 架构设计

### 3.1 模块结构

```
danji/
├── Cargo.toml
├── DESIGN.md              # 本文档
├── src/
│   ├── lib.rs             # 库入口，导出公共 API
│   ├── error.rs           # 错误类型定义
│   ├── tube/
│   │   ├── mod.rs         # 管型模块
│   │   ├── triode.rs      # 三极管模型
│   │   └── params.rs      # 管型参数数据库
│   ├── circuit/
│   │   ├── mod.rs         # 电路模块
│   │   ├── element.rs     # 线性元件 (R/C/L)
│   │   ├── node.rs        # 节点定义
│   │   └── solver.rs      # MNA 求解器
│   ├── gpu/
│   │   ├── mod.rs         # GPU 模块
│   │   ├── context.rs     # wgpu 上下文
│   │   ├── pipeline.rs    # 计算管线
│   │   └── shaders/
│   │       └── simulate.wgsl  # 计算着色器
│   ├── simulator.rs       # 仿真器封装
│   └── api.rs             # 公共 API
└── examples/
    ├── single_tube.rs     # 单管放大器示例
    └── plot_curves.rs     # 绘制特性曲线
```

### 3.2 核心类型

```rust
// 错误类型
#[derive(Debug, thiserror::Error)]
pub enum DanjiError {
    #[error("GPU initialization failed: {0}")]
    GpuInit(String),
    #[error("Simulation diverged at sample {sample}")]
    Diverged { sample: usize },
    #[error("Invalid circuit: {0}")]
    InvalidCircuit(String),
}

// 三极管参数
#[derive(Debug, Clone)]
pub struct TriodeParams {
    pub mu: f64,      // 放大系数
    pub ex: f64,      // 指数 (通常 1.3-1.4)
    pub kg1: f64,     // 阳极电流系数
    pub kp: f64,      // 平滑参数
    pub kvb: f64,     // 膝点参数
    pub cg: f64,      // 栅-阴电容 (pF)
    pub cp: f64,      // 阳-栅电容 (pF)
    pub ck: f64,      // 阳-阴电容 (pF)
}

// 电路节点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

// 线性元件
#[derive(Debug, Clone)]
pub enum Element {
    Resistor { node_a: NodeId, node_b: NodeId, resistance: f64 },
    Capacitor { node_a: NodeId, node_b: NodeId, capacitance: f64, state: f64 },
}

// 三极管
#[derive(Debug, Clone)]
pub struct Triode {
    pub plate: NodeId,
    pub grid: NodeId,
    pub cathode: NodeId,
    pub params: TriodeParams,
}

// 电路配置
#[derive(Debug, Clone)]
pub struct CircuitConfig {
    pub sample_rate: u32,
    pub elements: Vec<Element>,
    pub triodes: Vec<Triode>,
    pub input_node: NodeId,
    pub output_node: NodeId,
    pub ground_node: NodeId,
}

// 仿真器
pub struct Simulator {
    config: CircuitConfig,
    state: SimulationState,
    gpu_context: Option<GpuContext>,
}

// GPU 上下文
struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    buffers: GpuBuffers,
}
```

### 3.3 数据流

```
输入音频 f32
    ↓
┌─ Simulator::process_buffer() ─────────────────────┐
│  1. 上传输入缓冲区到 GPU                          │
│  2. 设置 uniform 参数 (当前电路状态)              │
│  3. 分发计算着色器 (每采样点一个 invocation)       │
│  4. 读回输出缓冲区                                │
└───────────────────────────────────────────────────┘
    ↓
输出音频 f32
```

---

## 4. 物理模型

### 4.1 三极管模型 (Koren 改进)

#### 4.1.1 原始 Child-Langmuir 模型

真空管阳极电流的基本物理模型源自 Child-Langmuir 定律（1911），描述空间电荷限制下的电流：

$$
I_p = K \left( V_g + \frac{V_p}{\mu} \right)^{\gamma}
$$

其中：
- $I_p$ — 阳极电流 (A)
- $V_g$ — 栅-阴极电压 (V)
- $V_p$ — 阳-阴极电压 (V)
- $\mu$ — 放大系数
- $K$ — 管子常数
- $\gamma$ — 指数，理论值 $3/2$

**局限性**：该模型在截止区（$V_g + V_p/\mu \leq 0$）不连续，且无法准确描述大负栅压区域。

#### 4.1.2 Koren 改进模型

Norman Koren 提出的改进模型使用平滑函数解决不连续问题：

**中间变量 $E_1$：**

$$
E_1 = \frac{V_p}{k_P} \cdot \ln\left(1 + \exp\left(k_P \left(\frac{1}{\mu} + \frac{V_g}{\sqrt{k_{VB} + V_p^2}}\right)\right)\right)
$$

**阳极电流 $I_p$：**

$$
I_p = \frac{E_1^{E_x} + |E_1| \cdot E_1^{E_x - 1}}{k_{G1}}
$$

其中：
- $k_P$ — 平滑参数，控制截止区→导通区的过渡
- $k_{VB}$ — 膝点参数，决定曲线的"弯曲"位置
- $E_x$ — 指数项，通常 $1.3 \leq E_x \leq 1.4$
- $k_{G1}$ — 电流系数，决定 I-V 曲线的斜率

**物理意义**：
- $\ln(1 + \exp(x))$ 函数在 $x \gg 1$ 时趋近 $x$，在 $x \ll 1$ 时趋近 $0$，实现平滑过渡
- $E_x$ 源自 Child-Langmuir 定律的 $3/2$ 次方，实测值略有偏差
- $k_{VB}$ 中的 $\sqrt{k_{VB} + V_p^2}$ 项确保在所有 $V_p$ 下都有定义

#### 4.1.3 截止条件

$$
I_p = \begin{cases}
\frac{E_1^{E_x} + |E_1| \cdot E_1^{E_x - 1}}{k_{G1}} & \text{if } E_1 > 0 \\
0 & \text{if } E_1 \leq 0
\end{cases}
$$

#### 4.1.4 常用管型参数

| 管型 | $\mu$ | $E_x$ | $k_{G1}$ | $k_P$ | $k_{VB}$ |
|------|-------|-------|----------|-------|----------|
| 12AX7 | 100 | 1.4 | 1060 | 600 | 300 |
| 12AU7 | 21.5 | 1.3 | 1180 | 84 | 300 |
| 6L6CG | 8.7 | 1.35 | 1460 | 48 | 12 |
| 6550 | 7.9 | 1.35 | 890 | 60 | 24 |

#### 4.1.5 实现代码

```rust
fn triode_ip(vp: f64, vg: f64, params: &TriodeParams) -> f64 {
    // 中间变量 E1
    let e1 = (vp / params.kp) * (1.0 + (
        (1.0 / params.mu + vg / (params.kvb + vp * vp).sqrt()) * params.kp
    ).exp()).ln();

    // 阳极电流
    let ip = (e1.powf(params.ex) + e1.abs() * e1.powf(params.ex - 1.0)) / params.kg1;

    ip.max(0.0)
}
```

### 4.2 动态元件

#### 4.2.1 电容的离散化

使用后向欧拉法（Backward Euler）对电容进行时域离散化：

$$
I_C[n] = C \cdot \frac{V_C[n] - V_C[n-1]}{h}
$$

其中：
- $I_C[n]$ — 当前采样点的电容电流
- $C$ — 电容值 (F)
- $V_C[n]$ — 当前采样点电压
- $V_C[n-1]$ — 上一采样点电压（状态存储）
- $h = 1/f_s$ — 时间步长

**等效电路**：电容可等效为一个电导 $G_C = C/h$ 并联一个电流源 $I_{hist} = C \cdot V_C[n-1] / h$

#### 4.2.2 电感的离散化

使用后向欧拉法对电感进行时域离散化：

$$
V_L[n] = L \cdot \frac{I_L[n] - I_L[n-1]}{h}
$$

等效为：

$$
I_L[n] = I_L[n-1] + \frac{h}{L} \cdot V_L[n]
$$

**等效电路**：电感可等效为一个电导 $G_L = h/L$ 并联一个电流源 $I_{hist} = I_L[n-1]$

### 4.3 节点分析法 (MNA)

#### 4.3.1 基本原理

改进节点分析法（Modified Nodal Analysis）将电路表示为线性方程组：

$$
\mathbf{G} \cdot \mathbf{V} = \mathbf{I}
$$

其中：
- $\mathbf{G}$ — 导纳矩阵（$N \times N$，$N$ 为节点数）
- $\mathbf{V}$ — 节点电压向量
- $\mathbf{I}$ — 激励电流向量

#### 4.3.2 电阻的贡献

对于电阻 $R$ 连接节点 $a$ 和 $b$，电导 $G = 1/R$：

$$
\mathbf{G}_{aa} \mathrel{+}= G, \quad \mathbf{G}_{bb} \mathrel{+}= G, \quad \mathbf{G}_{ab} \mathrel{-}= G, \quad \mathbf{G}_{ba} \mathrel{-}= G
$$

#### 4.3.3 电容的贡献（后向欧拉）

对于电容 $C$ 连接节点 $a$ 和 $b$，等效电导 $G_C = C/h$：

$$
\mathbf{G}_{aa} \mathrel{+}= \frac{C}{h}, \quad \mathbf{G}_{bb} \mathrel{+}= \frac{C}{h}, \quad \mathbf{G}_{ab} \mathrel{-}= \frac{C}{h}, \quad \mathbf{G}_{ba} \mathrel{-}= \frac{C}{h}
$$

历史电流源：

$$
\mathbf{I}_a \mathrel{+}= \frac{C}{h} \cdot (V_b[n-1] - V_a[n-1])
$$

$$
\mathbf{I}_b \mathrel{+}= \frac{C}{h} \cdot (V_a[n-1] - V_b[n-1])
$$

#### 4.3.4 求解方法

阶段 1（MVP）：高斯消元法，适用于小规模电路（$N < 20$）

阶段 2（优化）：LU 分解，支持矩阵复用

### 4.4 牛顿-拉夫森迭代

#### 4.4.1 非线性问题

三极管模型是非线性的，无法直接求解 $\mathbf{G} \cdot \mathbf{V} = \mathbf{I}$。需要使用牛顿-拉夫森迭代将其线性化。

#### 4.4.2 迭代公式

给定当前估计 $\mathbf{V}^{(k)}$，求解：

$$
\mathbf{J}(\mathbf{V}^{(k)}) \cdot \Delta \mathbf{V} = -\mathbf{F}(\mathbf{V}^{(k)})
$$

其中：
- $\mathbf{J}$ — 雅可比矩阵，$\mathbf{J}_{ij} = \partial F_i / \partial V_j$
- $\mathbf{F}$ — 残差向量，$\mathbf{F} = \mathbf{G}(\mathbf{V}) \cdot \mathbf{V} - \mathbf{I}$
- $\Delta \mathbf{V} = \mathbf{V}^{(k+1)} - \mathbf{V}^{(k)}$

更新：

$$
\mathbf{V}^{(k+1)} = \mathbf{V}^{(k)} + \Delta \mathbf{V}
$$

#### 4.4.3 收敛条件

$$
\|\Delta \mathbf{V}\| < \epsilon
$$

其中 $\epsilon$ 为容差，通常取 $10^{-6}$。

#### 4.4.4 三极管的雅可比元素

对于三极管阳极电流 $I_p = f(V_p, V_g)$：

$$
\frac{\partial I_p}{\partial V_p} = \frac{1}{k_{G1}} \cdot \left( E_x \cdot E_1^{E_x - 1} \cdot \frac{\partial E_1}{\partial V_p} + \text{sgn}(E_1) \cdot E_1^{E_x - 1} \cdot \frac{\partial E_1}{\partial V_p} \right)
$$

其中 $\frac{\partial E_1}{\partial V_p}$ 可通过链式法则计算。

#### 4.4.5 实现代码

```rust
fn solve_nonlinear(circuit: &Circuit, guess: &[f64; N]) -> Result<[f64; N], DanjiError> {
    let mut x = *guess;
    for _ in 0..MAX_ITERATIONS {
        let (jacobian, residual) = circuit.jacobian_and_residual(&x);
        let delta = solve_linear(&jacobian, &residual)?;
        x = x - delta;
        if delta.iter().map(|d| d.abs()).sum::<f64>() < TOLERANCE {
            return Ok(x);
        }
    }
    Err(DanjiError::Diverged { sample: 0 })
}
```

---

## 5. GPU 加速

### 5.1 计算着色器 (WGSL)

```wgsl
// simulate.wgsl
struct Params {
    sample_rate: u32,
    num_samples: u32,
    triode_mu: f32,
    triode_ex: f32,
    triode_kg1: f32,
    triode_kp: f32,
    triode_kvb: f32,
}

@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= params.num_samples) {
        return;
    }

    let input_sample = input[idx];

    // 三极管模型计算
    let vg = input_sample;
    let vp = 300.0;  // 阳极电压 (简化)
    let e1 = (vp / params.triode_kp) *
        log(1.0 + exp((1.0 / params.triode_mu + vg / sqrt(params.triode_kvb + vp * vp)) * params.triode_kp));
    let ip = (pow(e1, params.triode_ex) + abs(e1) * pow(e1, params.triode_ex - 1.0)) / params.triode_kg1;

    output[idx] = ip;
}
```

### 5.2 缓冲区设计

| 缓冲区 | 类型 | 用途 |
|--------|------|------|
| `input` | Storage (read) | 输入音频采样 |
| `output` | Storage (read_write) | 输出音频采样 |
| `params` | Uniform | 电路参数 |
| `state` | Storage (read_write) | 动态元件状态 (电容电压等) |

### 5.3 分发策略

```
Workgroup 大小: 64 (WebGPU 推荐值)
总 invocation 数: ceil(num_samples / 64) * 64
每个 invocation 处理一个采样点
```

---

## 6. 公共 API

### 6.1 核心 API

```rust
impl Simulator {
    /// 创建新的仿真器
    pub fn new(config: &CircuitConfig) -> Result<Self, DanjiError>;

    /// 更新电路参数 (实时调参)
    pub fn update_param(&mut self, param: ParamId, value: f64) -> Result<(), DanjiError>;

    /// 处理音频缓冲区 (批量)
    pub fn process_buffer(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), DanjiError>;

    /// 处理单个采样点
    pub fn process_sample(&mut self, input: f32) -> Result<f32, DanjiError>;

    /// 重置仿真状态
    pub fn reset(&mut self);

    /// 获取当前电路状态快照
    pub fn state_snapshot(&self) -> SimulationState;
}
```

### 6.2 配置 API

```rust
impl CircuitConfig {
    /// 创建默认单管共阴极放大器配置
    pub fn single_triode(triode: TriodeParams) -> Self;

    /// 添加电阻
    pub fn add_resistor(&mut self, node_a: NodeId, node_b: NodeId, resistance: f64) -> NodeId;

    /// 添加电容
    pub fn add_capacitor(&mut self, node_a: NodeId, node_b: NodeId, capacitance: f64) -> NodeId;

    /// 添加三极管
    pub fn add_triode(&mut self, triode: Triode) -> NodeId;
}
```

### 6.3 参数 API

```rust
pub enum ParamId {
    // 三极管参数
    TriodeMu(usize),        // 第 i 个三极管的 mu
    TriodeEx(usize),        // 第 i 个三极管的 ex
    TriodeKg1(usize),       // 第 i 个三极管的 kg1
    // ...
    // 电路参数
    ResistorValue(NodeId),  // 电阻值
    CapacitorValue(NodeId), // 电容值
}
```

---

## 7. 实现计划

### 阶段 1: 单管共阴极放大器 (MVP)

**目标：** 验证核心物理模型正确性

**步骤：**
1. 定义错误类型和基础结构
2. 实现三极管模型 (triode.rs)
3. 实现线性元件 (element.rs)
4. 实现 MNA 求解器 (solver.rs)
5. 实现仿真器封装 (simulator.rs)
6. 实现公共 API (api.rs)
7. 编写测试和示例

**验证方法：**
- 生成 12AX7 特性曲线 (Ip vs Vp, 不同 Vg)
- 与 Koren 参数表理论曲线对比
- 误差 < 5% 即通过

**预计时间：** 3-5 天

### 阶段 2: GPU 加速

**目标：** 实现并行采样点计算

**步骤：**
1. 实现 wgpu 上下文初始化
2. 编写 WGSL 计算着色器
3. 实现缓冲区管理
4. 集成到仿真器

**预计时间：** 2-3 天

### 阶段 3: 多级放大器

**目标：** 支持前级 + 后级电路

**步骤：**
1. 扩展电路拓扑支持
2. 实现级间耦合
3. 添加输出变压器模型

**预计时间：** 3-5 天

### 阶段 4: 完整胆机

**目标：** 功率放大 + 电源滤波 + 音调控制

**预计时间：** 5-7 天

---

## 8. 测试策略

### 8.1 单元测试

```rust
#[test]
fn test_triode_12ax7_ip() {
    let params = TriodeParams::new_12ax7();
    // Vp=250V, Vg=-2V 时，Ip 应约为 1.0mA
    let ip = triode_ip(250.0, -2.0, &params);
    assert!((ip - 0.001).abs() < 0.0001);
}

#[test]
fn test_capacitor_current() {
    let mut cap = Capacitor::new(0.001); // 1uF
    let i = capacitor_current(&mut cap, 10.0, 1.0/44100.0);
    assert!((i - 0.0441).abs() < 0.001);
}
```

### 8.2 集成测试

```rust
#[test]
fn test_single_tube_amplifier() {
    let config = CircuitConfig::single_triode(TriodeParams::new_12ax7());
    let mut sim = Simulator::new(&config).unwrap();

    // 输入 1kHz 正弦波
    let input: Vec<f32> = (0..44100)
        .map(|i| (2.0 * PI * 1000.0 * i as f64 / 44100.0).sin() as f32 * 0.1)
        .collect();

    let mut output = vec![0.0f32; 44100];
    sim.process_buffer(&input, &mut output).unwrap();

    // 验证输出幅度 (增益约 50-100)
    let input_rms = rms(&input);
    let output_rms = rms(&output);
    let gain = output_rms / input_rms;
    assert!(gain > 30.0 && gain < 200.0);
}
```

### 8.3 可视化验证

```rust
#[test]
fn plot_12ax7_characteristics() {
    let params = TriodeParams::new_12ax7();
    let mut plot = Plot::new();

    for vg in (-4..=0).step_by(1) {
        let points: Vec<(f64, f64)> = (0..=400)
            .map(|vp| (vp as f64, triode_ip(vp as f64, vg as f64, &params) * 1000.0))
            .collect();
        plot.add_curve(&format!("Vg={}V", vg), &points);
    }

    plot.save("12ax7_characteristics.png").unwrap();
}
```

---

## 9. 参考资料

### 学术论文
1. Pakarinen & Yeh (2009). "A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers". *Computer Music Journal*, 33(2), 85-100.
2. DAFX 2023. "A Quadric Surface Model of Vacuum Tubes for Virtual Analog Modeling".

### SPICE 模型
1. Norman Koren. "Improved vacuum tube models for SPICE". https://www.normankoren.com/Audio/Tube_params.html
2. Ayumi SPICE Models. https://www.diyaudio.com/community/threads/vacuum-tube-spice-models.243950/

### 开源项目
1. SwankyAmp. https://github.com/resonantdsp/SwankyAmp
2. Guitarix. https://github.com/brummer10/guitarix
3. Neural Amp Modeler. https://www.neuralampmodeler.com/

### Rust 生态
1. wgpu. https://github.com/gfx-rs/wgpu
2. Rust Audio. https://rust.audio/
3. neodsp. https://neodsp.com/

---

## 10. 待解决问题

1. **GPU 初始化时机**：是否在 `Simulator::new()` 中初始化？还是延迟初始化？
2. **状态管理**：电容/电感状态如何在 GPU 缓冲区中组织？
3. **参数更新同步**：参数修改是否需要立即同步到 GPU？
4. **错误恢复**：GPU 计算失败时如何回退到 CPU？

---

*文档版本: 0.1.0*
*最后更新: 2026-06-24*
