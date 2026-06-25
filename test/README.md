# danji-cli 胆味测试

基于学术论文的胆机放大器仿真工具测试框架。

## 快速开始

```bash
# 前置条件：cargo build --release 已编译 danji-cli

cd test
./scripts/run_test.sh
```

测试完成后查看 `reports/YYYY-MM-DD_NNN_report.pdf`。

## 测试内容

对 danji-cli 的三种放大器模型进行谐波分析：

| 模型 | 拓扑 | 增益 |
|------|------|------|
| single | 单管共阴极 | ~62x |
| two-stage | 级联双级 | ~9022x |
| chain | 完整前级链 | ~9022x + PSU + 音调 |

测试信号：100 Hz / 1 kHz / 10 kHz 正弦波（-6 dBFS, 2s, 44.1 kHz）

## 合格标准

| 指标 | 合格标准 | 依据 |
|------|----------|------|
| 二次谐波占比 | ≥ 60% | 胆机以偶次谐波为主 |
| 谐波衰减 | 随阶数递减 | 胆机低次谐波特征 |
| THD | 0.5% - 3% | 胆机典型范围 |
| 五次以上谐波 | < 5% | 高次谐波应微弱 |

## 目录结构

```
test/
├── template.typ              # typst 报告模板（数据驱动）
├── scripts/
│   ├── run_test.sh           # 一键测试脚本
│   ├── generate_test_audio.py # 生成测试音频
│   ├── analyze.py             # 谐波分析 + THD 计算
│   └── plot_results.py        # 图表生成
├── input/                     # 测试音频（自动生成）
├── output/                    # 处理后音频（自动生成）
├── analysis/
│   ├── reports/<run-id>/      # JSON 分析数据
│   └── plots/<run-id>/        # 图表
└── reports/                   # PDF 报告
```

## 工具链

- **编译**: Rust / cargo
- **分析**: Python 3.13 + numpy + scipy + matplotlib (via uv)
- **报告**: Typst 0.15

## 首轮测试结果

| 模型 | 100 Hz | 1 kHz | 10 kHz |
|------|--------|-------|--------|
| single | PASS | BORDERLINE | PASS |
| two-stage | FAIL | FAIL | BORDERLINE |
| chain | FAIL | FAIL | BORDERLINE |

single 模型达标：THD ≈ 2.6%，二次谐波占比 ≥ 98%。
two-stage / chain 模型增益过高导致严重削波，需参数调优。

## 参考文献

1. Hamm, "Harmonic distortion characteristics of solid-state and vacuum-tube preamplifier stages under overload conditions"
2. Texas Instruments, "如何测量运算放大器的总谐波失真和 THD+N 的基本原理", 2023
3. Maleczek, "Comparative analysis of sound quality of vacuum-tube amplifiers and transistor amplifiers", Archives of Acoustics, 2012
4. Quod Libet, "Simulation of Electron Tube Audio Circuits", ICMC 1996
5. 漫步者社区, "什么是胆味", 2022
