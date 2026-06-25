# 推挽优化 + output_transformer 修复 + 耦合电感试错

## 推挽功率提升 (6.0mW)

从 0.2mW 提升到 6.0mW（30x），三个改动叠加：

| 改动 | 前 | 后 | 效果 |
|------|-----|-----|------|
| B+ | 250V | 300V | 摆幅更大 |
| LTP 尾阻 | 47kΩ | 10kΩ | 倒相器增益提高 |
| 输入幅度 | 0.5Vpk | 1.0Vpk | 驱动更强 |

EL84 DC 偏置：Vgk=-8.9V, Vp=268V（B+ ramp 后 vs 之前 236V，更合理）。

### API 修复

`process_sample_dual` 在失败时未重置 `input2_voltage`，导致后续调用
携带残留电压。修复：失败时恢复上一个值。同时该字段 `pub(crate)`。

## output_transformer.rs 修复

预存的 NaN bug（`-inf` 屏压），根源是 12AU7 在 B+=350V 冷启动时
通过 5kΩ 负载产生巨大压降导致 Vpk 负。

- B+ 缓启（0→350V 5000 步）
- 5kΩ 改为 100Ω DCR + analytical turns ratio（同 pentode_stage 做法）
- 输出正常：Plate DC=341V, 2.4mW

## 耦合电感 OPT 尝试（未成功）

在双 Simulator 输出级尝试了 `add_coupled_inductor` + 47Ω snubber，
仍然发散。BE 下 `L/h ≈ 110kΩ`（2.5H @ 44.1kHz）是根本限制。

放弃本次。后续可用简单 OPT（100Ω+10H + analytical turns ratio），
这是当前最稳定的方案。

## 仓库状态

```
0daac71 feat: increase push-pull power to 6mW; fix output_transformer NaN
502c251 refactor: add process_sample_dual; hide input2 plumbing
967c60c feat: dual-Simulator push-pull with external RC coupling
8b81735 chore: add push-pull debug devlog
3bba81b fix: add NaN guard at convergence
4f51106 fix: remove pivoting + pentode NaN guards
```
