# push_pull: 推挽功率级 WIP

## 已完成

### Solver 修复：移除部分主元选取

partial pivoting 在 MNA 矩阵（跨 9 个数量级：VSRC_G=1e6 vs grid G=2e-6）
中导致 catastrophic cancellation → grid 电压进入 ±1.2µV 极限环 → 牛顿法发散。

修复：移除行交换，直接对角线消元。MNA 矩阵天然对角占优，不需要 pivoting。

### 12AX7 长尾倒相器验证

`test_phase_inverter()` 在 7 节点电路中验证通过：
- DC: Vk=3.21V, Vp=246.6V (B+=250V, Rk=47kΩ, Rl=100kΩ)
- AC: V1a ±2.45V, V1b ±1.91V, 相位反相 ✓

### Pentode 模型防护

1. **Vpk guard**: 添加 `vpk <= 0.0` 返回 0，防止 `Inf * atan(0) = NaN`
2. **exp overflow guard**: 仿照 triode 模型，当 `eg > 50.0` 时使用渐近近似
   `ln(1+exp(x)) ≈ x`，避免 exp(arg) 溢出

## 未完成：完整推挽电路发散

### 问题

完整推挽电路（12AX7 倒相器 + 耦合电容 + EL84×2 + 2×CoupledInductor + 8Ω 扬声器）
在 B+ 缓启到约 124V 时出现 NaN。

### 根因分析

1. **耦合电容在 BE 模型中的瞬态行为**：B+ 缓启时，12AX7 屏极电压通过
   耦合电容（BE 等效 ~1kΩ）直接耦合到 EL84 栅极，使 Vgk ≈ Vp_12AX7 ≈ 247V
   （极正偏），EL84 电流巨大（~5A+）

2. **BE 耦合电感的高效 DC 电阻**：对于 L=2.5H, k=0.95 的耦合电感，
   BE 模型的等效 DC 电阻为 R_eff = Lp(1-k²)/h ≈ 10.7kΩ。
   5A 电流通过 10.7kΩ 产生 53kV 压降 → Vpk 为负 → pentode 模型
   产生 Inf → Inf·atan(0) = NaN

3. **瞬态过程中 Newton 迭代振荡**：EL84 在导通/截止间振荡（每步从
   Ip ≈ 5A 跳到 0），max_delta 远大于 TOL，50 次迭代不收敛

### 尝试过的修复

- **Snubber 电阻**: 在每个 EL84 屏极到 B+ 并联 150Ω。这降低了等效阻抗，
  但 i1_prev 的 (1-d) 阻尼与矩阵 1/R 双重复用导致 i1_prev 计算不稳定
- **CoupledInductor 矩阵加入阻尼**: 在 solver stamp 中加 1/R 并行阻尼。
  但 i1_prev 的 BE 更新仍使用无阻尼公式，导致反馈环路正增益
- **DC 直耦替代耦合电容**: 用电阻分压器连接 12AX7 屏极和 EL84 栅极。
  虽然避免了对 EL84 栅极 DC 浮空，但 EL84 栅极 DC 偏置仍为正
  （Vgk > 0），导致静态电流过大
- **减小 12AX7 阴极电阻**: 从 47kΩ 改为 1kΩ，提高倒相器电流和增益，
  但 12AX7 屏压降低，EL84 栅压通过分压器仍为正

### 下一步方向

1. **耦合电感模型改进**：将 DCR 直接包含在 MNA stamp 中（非仅 state update），
   BE 更新公式相应调整为无阻尼纯 BE 形式
2. **分阶段预热**：先用 12AX7 建立工作点（EL84 屏栅接地），
   待耦合电容充电完毕再引入 EL84
3. **改用两个独立 Simulator**：倒相器一个、推挽输出级一个，
   通过外部 RC 滤波隔直耦合。这与 original decision 不同，
   但可能是当前 BE 限制下的唯一可行方案
4. **增加 MAX_ITER 和引入线搜索**：当前 50 次迭代对强非线性不够，
   线搜索可以防止 Newton 迭代 overshoot
