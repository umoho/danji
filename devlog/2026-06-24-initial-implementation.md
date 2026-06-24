## 2026-06-24

### 项目初始化

创建胆机模拟库 `danji`，基于 Rust + WGSL 技术栈，采用物理模拟方法。

### 调研阶段

查阅了以下关键资源：
- 论文: Pakarinen & Yeh (2009) "A Review of Digital Techniques for Modeling Vacuum-Tube Guitar Amplifiers", DAFx 2023 "Quadric Surface Model"
- SPICE 模型: Norman Koren 改进模型（业界标准），Ayumi 模型库
- 开源项目: SwankyAmp (FAUST+JUCE), Guitarix, Neural Amp Modeler

### 设计决策

- 建模视角: 宏观（端口特性），基于 Child-Langmuir 定律 + Koren 改进模型
- 电路求解: 改进节点分析法 (MNA) + 牛顿-拉夫森迭代 + 后向欧拉法
- API 风格: 同步 API（`process_buffer`, `process_sample`），无 tokio
- GPU 策略: 延迟到阶段 2 实现（先 CPU 验证模型正确性）
- 错误处理: 特定错误类型 `DanjiError`，不用 anyhow
- 线性代数: 手写高斯消元（小规模电路足够），不引入 nalgebra

### 依赖选择

| Crate | 版本 | 用途 |
|-------|------|------|
| wgpu | 22 | GPU 计算（阶段 2） |
| pollster | 0.4 | 阻塞 wgpu async 调用 |
| bytemuck | 1 | 字节转换 |
| thiserror | 2 | 错误类型派生 |
| log | 0.4 | 日志接口 |
| serde | 1 (可选) | 配置文件支持 |

不引入的依赖：tokio（同步 API）、anyhow（特定错误类型）、nalgebra（初期不需要）、dasp（不想引入外部 DSP 框架）

### 物理模型实现

实现了 Koren 改进三极管模型：

```
E1 = (Vp/kP) * ln(1 + exp(kP * (1/μ + Vg/√(kVB + Vp²))))
Ip = (E1^Ex + |E1| * E1^(Ex-1)) / kG1
```

使用有限差分计算雅可比元素（gp = dIp/dVp, gm = dIp/dVg）。

内置 9 种管型参数：12AX7、12AU7、12AT7、6DJ8、6L6GC、6550、EL34、KT88。

### 求解器实现

- MNA 矩阵组装（电阻、电容、三极管 VCCS）
- 电容使用后向欧拉法离散化，状态保存在 `v_prev`
- 电压源通过大电导法实现（G += 1e12, I += 1e12 * V）
- 高斯消元 + 部分主元消去
- 牛顿-拉夫森迭代（最大 50 次，容差 1e-9）
- 固定大小矩阵（MAX_NODES = 20）

### 验证结果

**三极管特性曲线**: 12AX7 曲线符合 Koren 参数表
- Vp=250V, Vg=-2V: Ip ≈ 0.95 mA
- rp ≈ 53.7 kΩ
- gm ≈ 1.67 mA/V

**单管放大器**: 12AX7 共阴极放大级
- 电路: B+ 300V, R_plate 100kΩ, R_k 1.5kΩ, C_k 22μF, R_g 1MΩ
- 静态工作点 Vp ≈ 103V
- 增益 ≈ 72x (37.1 dB)
- 输入 0.1V → 输出 7.2V AC 摆幅

### 单元测试

9 个测试全部通过，覆盖：截止区电流为零、正电流输出、Ip 随 Vp/Vg 单调增加、典型偏置点、内阻和跨导范围、12AX7 vs 12AU7 对比、导数正定性。

### 待实现

- GPU 加速: wgpu 计算着色器并行处理采样点
- 多级放大: 前级 + 后级级联
- 输出变压器: 耦合电感模型
- 电源滤波: 整流管 + RC 滤波
- 音调控制: 被动/主动音调网络
- 失真分析: THD 计算
