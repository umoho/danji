# push_pull 双 Simulator 方案跑通

## 问题

单 Simulator 推挽电路在 B+ 缓启时发散。根因：
BE 耦合电感等效 DC 阻抗 `L/h ≈ 110kΩ`（2.5H @ 44.1kHz），
EL84 在 Vgk 极正偏时电流巨大，经该阻抗产生压降 → Vpk 负 → Newton 振荡。

## 方案：双 Simulator + 外部 RC 隔直

```
12AX7 倒相器 Sim  ──→  外部 RC 高通滤波  ──→  EL84 推挽输出级 Sim
     (input2)     ──→  外部 RC 高通滤波  ──→  (input2)
```

关键设计：
- 倒相器和输出级分开，各自由 B+ 缓启
- 级间电容用外部 RC 指数滤波模拟（同 full_chain.rs）
- DC-block 滤波器预充电至屏极稳态 DC，避免启动瞬态
- 输出级 Simulator 通过 `input2` 接收反相信号

## API 改动

SimConfig 新增：
- `input2_node: NodeId` — 第二个输入节点
- `input2_voltage: f64` — 第二个输入电压
- `.input2(node)` — builder 方法

Simulator 新增：
- `set_input2(voltage)` — 每样本设置第二输入电压

Solver 新增：
- 第二个 VSRC stamp（`circuit.input2_node`）

## 结果

```
=== Push-Pull DC Bias ===
12AX7 V1a: Vg=0.00 Vk=3.21 Vp=246.59
12AX7 V1b: Vg=0.00 Vk=3.21 Vp=246.59
EL84a: Vg=0.00 Vk=7.20 Vs=243.96 Vp=198.25
EL84b: Vg=0.00 Vk=7.20 Vs=243.96 Vp=198.25

=== Push-Pull AC (1kHz, 0.5Vpk input) ===
Speaker: 39.9 mV RMS, 0.2 mW
```

- DC 偏置合理：EL84 Vgk=-7.2V, Vp=198V
- AC 输出功率低（0.2mW），因倒相器增益仅 5x，且 OPT 匝比 25:1
- 无 NaN、无发散

## 下一步优化

1. 提高输出功率：增大输入信号、使用固定偏置、改用耦合电感 OPT
2. 耦合电感收敛问题仍存在（单 Simulator 中的 ~110kΩ BE 阻抗），
   可在双 Simulator 的输出级中尝试
3. 现有 `output_transformer.rs` 示例有 NaN 问题（预存 bug，非本次引入）
