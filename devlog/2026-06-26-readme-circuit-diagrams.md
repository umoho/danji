# README 与电路拓扑图

为项目添加面向读者的 README 文档，并使用 netlistsvg 自动生成电路原理图。

## 决策

- **语言**：中文正文 + 英文翻译版，互相链接
- **电路图工具**：试过 Typst + Zap（手动定位太痛苦），最终选 netlistsvg（JSON 网表 → SVG 自动布局）
- **嵌入方式**：直接嵌入 SVG（无损缩放），而非 PNG
- **chain 模型**：元件太多（17+），拆分为 PSU 和 Tone Control 两个子模块分别画图

## 变更

- `README.md` — 中文 README，含功能特性、Crate 架构、电路拓扑（single / two-stage / chain）、支持的电子管、目录结构、性能数据
- `README.en.md` — 英文翻译版
- `images/circuit.svg` — single 模型（单级共阴放大）
- `images/two_stage.svg` — two-stage 模型（两级 RC 耦合级联）
- `images/psu.svg` — chain 模型电源模块（5AR4 + CRC π 滤波）
- `images/tone.svg` — chain 模型音调控制模块

## 提交

```
8e13a65 docs: add README with circuit topology diagram
febf03f docs: add two-stage circuit topology diagram
2858afd fix: widen two-stage SVG to prevent text clipping
2d3d03e docs: add chain model PSU and tone control circuit diagrams
1e59ca0 fix: widen PSU and tone control SVGs to prevent clipping
```

## 踩坑

- Typst + Zap：三极管符号不在内置库中，手动用 CeTZ 画符号 + 手动定位元件，效率极低且效果差
- netlistsvg：为 Yosys 数字逻辑设计，模拟元件需用 analog skin；三极管不在默认 skin 中，显示为 generic box（可接受）
- SVG 裁切：netlistsvg 生成的 SVG 宽度偏窄，需手动加宽以防止右侧文字被裁
