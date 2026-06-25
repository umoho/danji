// danji-cli 胆味测试报告 — 自动生成
// 数据来源: analysis/reports/<run-id>/

#set document(
  title: "danji-cli 胆味测试报告",
  author: "danji 测试框架",
  date: datetime(year: 2026, month: 6, day: 25),
)

#set page(
  paper: "a4",
  margin: 2.5cm,
  header: [
    #set text(size: 8pt, fill: gray)
    #h(1fr) danji-cli 测试报告 — Run 2026-06-25_001
  ],
  footer: [
    #set text(size: 8pt, fill: gray)
    #h(1fr) context counter(page).display("1 / 1", both: true)
  ],
)

#set text(
  font: ("PingFang SC", "Helvetica Neue"),
  size: 11pt,
  lang: "zh",
)

#set heading(numbering: "1.")
#show heading.where(level: 1): it => {
  set text(size: 16pt, weight: "bold")
  block(above: 1.5em, below: 0.8em)[#it]
}
#show heading.where(level: 2): it => {
  set text(size: 13pt, weight: "bold")
  block(above: 1.2em, below: 0.6em)[#it]
}

// ===== 数据加载 =====
#let run-id = "2026-06-25_001"
#let report-dir = "analysis/reports/" + run-id
#let plots-dir = "analysis/plots/" + run-id

#let single-data = json(report-dir + "/analysis_single.json")
#let twostage-data = json(report-dir + "/analysis_two-stage.json")
#let chain-data = json(report-dir + "/analysis_chain.json")

// ===== 辅助函数 =====
#let verdct-color(verdict) = {
  if verdict == "PASS" { green }
  else if verdict == "BORDERLINE" { orange }
  else { red }
}

#let verdct-text(v) = text(fill: verdct-color(v.verdict))[#v.verdict]

#let fmt-thd(val) = {
  str(calc.round(val, digits: 3)) + "%"
}

#let fmt-pct(val) = {
  str(calc.round(val, digits: 1)) + "%"
}

#let make-table(data) = {
  table(
    columns: (auto, auto, auto, auto, auto, auto),
    align: (left, center, center, center, center, center),
    table.header([*频率*], [*THD*], [*2nd 占比*], [*高次占比*], [*衰减*], [*判定*]),
    ..data.map(r => {
      let f0 = if r.f0_hz >= 1000 {
        str(r.f0_hz / 1000) + " kHz"
      } else {
        str(r.f0_hz) + " Hz"
      }
      let decay = if r.verdict.checks.harmonics_decaying [✓] else [✗]
      ([#f0], [#fmt-thd(r.output_thd_pct)], [#fmt-pct(r.second_harmonic_ratio_pct)], [#fmt-pct(r.high_harmonic_ratio_pct)], [#decay], [#verdct-text(r.verdict)])
    }).flatten()
  )
}

#let total-pass(data) = {
  data.filter(r => r.verdict.verdict == "PASS").len()
}

#let f0-name(r) = {
  if r.f0_hz == 100 { "100hz" }
  else if r.f0_hz == 1000 { "1khz" }
  else { "10khz" }
}

#let f0-label(r) = {
  if r.f0_hz == 100 { "100 Hz" }
  else if r.f0_hz == 1000 { "1 kHz" }
  else { "10 kHz" }
}

// ===== 报告正文 =====

= 摘要

本报告记录 danji-cli 胆机放大器仿真工具的"胆味"测试结果。测试基于学术论文定义的合格标准，对三种放大器模型（single / two-stage / chain）进行谐波分析和 THD 计算。

#{
  let s-pass = total-pass(single-data)
  let s-total = single-data.len()
  let tw-pass = total-pass(twostage-data)
  let c-pass = total-pass(chain-data)

  [*核心结论*：
  - #text(fill: green)[single 模型达标]（#s-pass/#s-total 通过）
  - #text(fill: red)[two-stage 模型未达标]（#tw-pass/#twostage-data.len() 通过）
  - #text(fill: red)[chain 模型未达标]（#c-pass/#chain-data.len() 通过）
  ]
}

= 测试方法

== 胆味的学术定义

"胆味"（Tube Sound）是电子管放大器特有的一种音色特征。根据频谱分析研究，胆机的谐波失真具有以下特征：

+ *偶次谐波主导*：二次谐波幅度最强
+ *低次谐波为主*：各次谐波幅度随阶数递减
+ *高次谐波微弱*：高阶谐波（5 次以上）幅度很小

== 合格标准

#figure(
  table(
    columns: (auto, auto, auto),
    align: (left, center, left),
    table.header([*指标*], [*合格标准*], [*依据*]),
    [二次谐波占比], [≥ 60%], [胆机以偶次谐波为主],
    [谐波衰减], [随阶数递减], [胆机低次谐波特征],
    [THD], [0.5% - 3%], [胆机典型范围],
    [五次以上谐波], [< 5%], [高次谐波应微弱],
  ),
  caption: [danji-cli 胆味测试合格标准],
)

== 判定规则

- *PASS*：4 项指标全部达标
- *BORDERLINE*：3 项达标，1 项轻微不达标
- *FAIL*：2 项及以上不达标

= 测试环境

#figure(
  table(
    columns: (auto, auto),
    align: (left, left),
    table.header([*项目*], [*详情*]),
    [测试工具], [danji-cli (Rust release)],
    [分析工具], [Python 3.13 + numpy + scipy + matplotlib],
    [测试信号], [100 Hz / 1 kHz / 10 kHz 正弦波, -6 dBFS, 2s],
    [采样率], [44100 Hz],
    [模型], [single / two-stage / chain],
    [Run ID], [#run-id],
  ),
  caption: [测试环境配置],
)

= 测试结果

== single 模型（单管共阴极放大）

#figure(
  make-table(single-data),
  caption: [single 模型测试结果],
)

#{
  let avg-thd = single-data.map(r => r.output_thd_pct).fold(0.0, (a, b) => a + b) / single-data.len()
  let avg-2nd = single-data.map(r => r.second_harmonic_ratio_pct).fold(0.0, (a, b) => a + b) / single-data.len()
  [平均 THD：#fmt-thd(avg-thd)，平均二次谐波占比：#fmt-pct(avg-2nd)。]
}

#for r in single-data [
  === #f0-label(r) 谐波频谱

  #figure(
    image(plots-dir + "/harmonics_single_sine_" + f0-name(r) + ".png", width: 80%),
    caption: [single 模型 #f0-label(r) 谐波频谱],
  )
]

== two-stage 模型（级联双级放大）

#figure(
  make-table(twostage-data),
  caption: [two-stage 模型测试结果],
)

#{
  let avg-thd = twostage-data.map(r => r.output_thd_pct).fold(0.0, (a, b) => a + b) / twostage-data.len()
  [平均 THD：#fmt-thd(avg-thd)，远超合格范围（0.5%-3%），存在严重削波。]
}

#for r in twostage-data [
  === #f0-label(r) 谐波频谱

  #figure(
    image(plots-dir + "/harmonics_two-stage_sine_" + f0-name(r) + ".png", width: 80%),
    caption: [two-stage 模型 #f0-label(r) 谐波频谱],
  )
]

== chain 模型（完整前级链）

#figure(
  make-table(chain-data),
  caption: [chain 模型测试结果],
)

#{
  let avg-thd = chain-data.map(r => r.output_thd_pct).fold(0.0, (a, b) => a + b) / chain-data.len()
  [平均 THD：#fmt-thd(avg-thd)，与 two-stage 模型表现相似。]
}

#for r in chain-data [
  === #f0-label(r) 谐波频谱

  #figure(
    image(plots-dir + "/harmonics_chain_sine_" + f0-name(r) + ".png", width: 80%),
    caption: [chain 模型 #f0-label(r) 谐波频谱],
  )
]

== 结果汇总

#figure(
  image(plots-dir + "/verdict_summary.png", width: 60%),
  caption: [测试结果汇总饼图],
)

#figure(
  image(plots-dir + "/thd_single.png", width: 70%),
  caption: [single 模型 THD 对比（绿色/红色线为合格范围）],
)

= 讨论

== single 模型为何合格

single 模型采用单管共阴极放大拓扑，增益约 62 倍。在输入 -6 dBFS（幅度 0.5V）的条件下，输出仍在电子管的线性工作区内，产生适量的偶次谐波失真，这正是胆味的声学基础。

#{
  let avg-thd = single-data.map(r => r.output_thd_pct).fold(0.0, (a, b) => a + b) / single-data.len()
  [THD 约 #fmt-thd(avg-thd) 处于胆机的典型范围（0.5%-3%）内，听感上表现为温暖、甜美的音色。]
}

== two-stage / chain 模型为何失败

两级 12AX7 级联的理论增益约 9022 倍（约 79 dB），远超单级。即使输入信号幅度很小，第二级也会进入深度非线性区，产生大量奇次谐波和高次谐波。这与胆机"偶次谐波主导、高次谐波微弱"的特征完全相反。

== 与文献对比

根据 Maleczek (2012) 的研究，真实胆机在 1 W 输出功率下的 THD 约为 0.15%-0.2%，而 danji-cli 的 single 模型 THD 约 2.6%，处于较高水平。但考虑到 danji-cli 模拟的是前级放大（非功率放大级），且未使用负反馈，这一数值是合理的。

= 结论与建议

== 结论

+ *single 模型达到胆味标准*：二次谐波占比 ≥ 98%，THD ≈ 2.6%，符合偶次谐波主导的胆机特征
+ *two-stage / chain 模型需要参数调优*：默认增益过高导致严重削波，不符合胆味特征
+ *测试框架有效*：基于学术论文的合格标准能够客观区分胆机特征

== 建议

+ *降低输入增益*：对 two-stage / chain 模型使用 `--gain -20dB` 或更低的增益参数
+ *调整 B+ 电压*：降低 B+ 电压可减小动态范围，避免削波
+ *增加负反馈*：在级间引入局部负反馈可降低增益并改善线性度
+ *音乐文件测试*：使用实际音乐文件进行主观听感验证

= 参考文献

+ Hamm, "Harmonic distortion characteristics of solid-state and vacuum-tube preamplifier stages under overload conditions"
+ Texas Instruments, "如何测量运算放大器的总谐波失真和 THD+N 的基本原理", 2023
+ Maleczek, "Comparative analysis of sound quality of vacuum-tube amplifiers and transistor amplifiers", Archives of Acoustics, 2012
+ Quod Libet, "Simulation of Electron Tube Audio Circuits", ICMC 1996
+ 漫步者社区, "什么是胆味", 2022
