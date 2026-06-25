# 音乐测试与 chain 参数暴露

## 背景

danji-cli 有三个模型（single/two-stage/chain），之前正弦波测试已完成。这次聚焦音乐文件测试（渡口 + 加州旅馆），探索 chain 的 `--bplus`/`--mix` 参数和 two-stage 的增益对比。

## 发现：--bplus/--mix 未接入

检查代码发现 `src/bin/danji-cli.rs` 中：

- `--bplus` 只在 single 模型生效（传给 `single_triode_config`），chain 的 `build_chain()` 硬编码了 PSU AC 电压 300.0，two-stage 的 `build_two_stage()` 硬编码了 B+ 300.0
- `--mix` 在 Args 中有定义，`dry` 变量计算后立即 `drop(dry)`，从未使用

## 参数暴露

修改 `build_chain(bplus: f64)` 接受参数替代硬编码的 300V（PSU AC + 预热）。`--mix` 实装在 output 生成后与 samples 做 `wet * mix + dry * (1 - mix)` 线性混合。

commit: `2357f5f`

## 音乐测试结果

### two-stage 增益对比（统一代码版本）

| gain | 渡口质心 | 渡口 bass | 加州质心 | 加州 bass |
|------|---------|----------|---------|----------|
| 0 | +634Hz 🔴 | +18.8dB | +1696Hz 🔴 | +12.9dB |
| -36 | -112Hz 🟢 | -11.2dB | -43Hz 🟢 | -7.5dB |
| -38 | -139Hz 🟢 | -13.1dB | -60Hz 🟢 | -8.9dB |
| -40 | -167Hz 🟢 | -14.9dB | -78Hz 🟢 | -10.3dB |

-38 在 warmth vs bass 衰减之间最平衡。 -40 更暖但 bass 多损失 1.8dB。

### chain 增益对比

| gain | 渡口质心 | 渡口 bass |
|------|---------|----------|
| 0 | +756Hz 🔴 | +18.0dB |
| -35 | +199Hz 🟡 | -12.2dB |
| -40 | +168Hz 🟡 | -16.2dB |

chain 质心始终上升，最低 +168Hz（渡口），和旧基线结论一致。

### 负增益在新旧代码间稳定

two-stage gain=-38 和 chain gain=-40 的指标与旧基线完全一致（如 dukou two-stage -38：质心 -139Hz）。gain=0 在当前代码不可用（所有频段 +20dB，RMS -3.8dB）。

## gain=0 的胆机过载讨论

分析正弦波谐波结构（run 004）：适中增益下 2 次谐波占 99%+，高次 < 0.02%，是非常典型的 A 类胆味。 gain=0 时 cascaded 增益 3600×，信号被 B+ 轨硬削波，缺少真实胆机过载的"动态压缩"和"供电轨塌陷"。

## 验证

```bash
cargo clippy --all-targets  # 无新增警告
cargo fmt                    # 格式一致
cargo build --release        # 编译通过
```

## 踩坑

- 旧基线数据来自不同代码版本，gain=0 指标不可比。重新跑了所有对比组。
- `.gitignore` 缺少 `test/analysis/music/`，提交后补加。
