## 2026-06-24

### 输出变压器模型

添加 `CoupledInductor` 耦合电感元件，用于模拟输出变压器（OPT）。

**实现**：
- 基于互感矩阵的伴随模型：`[I1; I2] = h * [Ls -M; -M Lp] / det * [V1; V2] + [I1_prev; I2_prev]`
- `det = Lp * Ls * (1 - k²)`
- 4 端子 MNA 支路（初+、初-、次+、次-）

**BE 数值稳定性问题**：
- 无损耗耦合电感在 BE 积分下发散（DC 电流线性增长）
- 加 `Rdc` 阻尼：`i_prev *= (1 - h*R/L)`，使 DC 分量指数衰减
- 同样修复了普通电感的 BE DC 泄漏问题

**示例** `examples/output_transformer.rs`：
```
12AU7 → OPT (10H + 5kΩ反射负载) → 8Ω扬声器
```
- 使用磁化电感 10H + 反射负载 5kΩ 简化建模
- 变比 N = sqrt(5000/8) ≈ 25:1
- 输出功率约 0.2mW（12AU7 驱动级，非功率管）

**CoupledInductor API**：
```rust
cfg.add_coupled_inductor(p_a, p_b, s_a, s_b, l_primary, l_secondary, coupling);
```

### 待解决

- 耦合电感的 BE 阻尼系数目前硬编码（假设 Rdc=150Ω/0.5Ω），需改为参数化
- 需添加铁芯饱和模型（非线性电感）
- 真正的推挽输出变压器（中心抽头）
