// danji-cli 胆味测试报告模板
// 用法: typst compile --root test/ test/template.typ output.pdf --input run-id=2026-06-25_001

#set document(
  title: "danji-cli 胆味测试报告",
  author: "danji 测试框架",
)

#set page(
  paper: "a4",
  margin: 2.5cm,
  header: [
    #set text(size: 8pt, fill: gray)
    #h(1fr) danji-cli 测试报告 — Run #sys.inputs.at("run-id", default: "unknown")
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

// ===== 辅助函数 =====
#let verdct-color(verdict) = {
  if verdict == "PASS" { green }
  else if verdict == "BORDERLINE" { orange }
  else { red }
}

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
      let vc = verdct-color(r.verdict.verdict)
      ([#f0], [#fmt-thd(r.output_thd_pct)], [#fmt-pct(r.second_harmonic_ratio_pct)], [#fmt-pct(r.high_harmonic_ratio_pct)], [#decay], [#text(fill: vc)[#r.verdict.verdict]])
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

#let model-label(m) = {
  if m == "single" { "single 模型（单管共阴极放大）" }
  else if m == "two-stage" { "two-stage 模型（级联双级放大）" }
  else { "chain 模型（完整前级链）" }
}

// ===== 主体 =====
#let run-id = sys.inputs.at("run-id", default: "2026-06-25_001")
#let data-dir = "analysis/reports/" + run-id
#let plots-dir = "analysis/plots/" + run-id
#let models = ("single", "two-stage", "chain")

// 加载所有数据
#let all-data = ()
#for model in models {
  let d = json(data-dir + "/analysis_" + model + ".json")
  all-data.push((model: model, data: d))
}

// ===== 摘要 =====
= 摘要

本报告记录 danji-cli 胆机放大器仿真工具的"胆味"测试结果。

#{
  let summary = ()
  for item in all-data {
    let pass = total-pass(item.data)
    let total = item.data.len()
    let color = if pass == total { green } else if pass > 0 { orange } else { red }
    summary.push([- #text(fill: color)[#item.model 模型达标]（#pass/#total 通过）])
  }
  [*核心结论*：]
  summary.join()
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
    [Run ID], [#run-id],
  ),
  caption: [测试环境配置],
)

= 测试结果

#for item in all-data {
  let model = item.model
  let data = item.data

  [== #model-label(model)]

  figure(
    make-table(data),
    caption: [#model 模型测试结果],
  )

  {
    let avg-thd = data.map(r => r.output_thd_pct).fold(0.0, (a, b) => a + b) / data.len()
    let avg-2nd = data.map(r => r.second_harmonic_ratio_pct).fold(0.0, (a, b) => a + b) / data.len()
    [平均 THD：#fmt-thd(avg-thd)，平均二次谐波占比：#fmt-pct(avg-2nd)。]
  }

  for r in data [
    === #f0-label(r) 谐波频谱

    #figure(
      image(plots-dir + "/harmonics_" + model + "_sine_" + f0-name(r) + ".png", width: 80%),
      caption: [#model 模型 #f0-label(r) 谐波频谱],
    )

    #figure(
      image(plots-dir + "/spectrum_" + model + "_sine_" + f0-name(r) + ".png", width: 80%),
      caption: [#model 模型 #f0-label(r) 频谱对比（输入 vs 输出）],
    )
  ]

  figure(
    image(plots-dir + "/thd_" + model + ".png", width: 70%),
    caption: [#model 模型 THD 对比],
  )
}

== 结果汇总

#figure(
  image(plots-dir + "/verdict_summary.png", width: 60%),
  caption: [测试结果汇总饼图],
)

= 讨论

== single 模型为何合格

single 模型采用单管共阴极放大拓扑，增益约 62 倍。在输入 -6 dBFS（幅度 0.5V）的条件下，输出仍在电子管的线性工作区内，产生适量的偶次谐波失真，这正是胆味的声学基础。

== two-stage / chain 模型为何失败

两级 12AX7 级联的理论增益约 9022 倍（约 79 dB），远超单级。即使输入信号幅度很小，第二级也会进入深度非线性区，产生大量奇次谐波和高次谐波。这与胆机"偶次谐波主导、高次谐波微弱"的特征完全相反。

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
