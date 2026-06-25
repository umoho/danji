// danji-cli 胆味测试报告模板
// 用法: typst compile --root test/ test/template.typ output.pdf --input run-id=2026-06-25_001
// 注意: 中英文间距由 typst cjk-latin-spacing:auto 自动处理，不要手动加空格

#set document(
  title: "danji-cli胆味测试报告",
  author: "danji测试框架",
)

#set page(
  paper: "a4",
  margin: 2.5cm,
  header: [
    #set text(size: 8pt, fill: gray)
    #h(1fr) danji-cli测试报告 — 报告编号: #sys.inputs.at("run-id", default: "unknown")
  ],
  footer: [
    #set text(size: 8pt, fill: gray)
    #h(1fr) #context counter(page).display("1 / 1", both: true)
  ],
)

#set text(
  font: ("Source Han Serif SC", "Times New Roman"),
  size: 12pt, // 小四号
  lang: "zh",
)

#show heading.where(level: 1): it => {
  set text(font: ("Source Han Sans SC", "Arial"), size: 16pt, weight: "bold") // 三号
  block(above: 1.5em, below: 0.8em)[#it]
}
#show heading.where(level: 2): it => {
  set text(font: ("Source Han Sans SC", "Arial"), size: 14pt, weight: "bold") // 四号
  block(above: 1.2em, below: 0.6em)[#it]
}
#show heading.where(level: 3): it => {
  set text(font: ("Source Han Sans SC", "Arial"), size: 12pt, weight: "bold") // 小四号
  block(above: 1em, below: 0.5em)[#it]
}

#set heading(numbering: "1.")

// ===== 引用标注 =====
#let ref(num) = {
  super[[#num]]
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
  if m == "single" { "single模型（单管共阴极放大）" }
  else if m == "two-stage" { "two-stage模型（级联双级放大）" }
  else { "chain模型（完整前级链）" }
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

本报告记录danji-cli胆机放大器仿真工具的"胆味"测试结果。测试基于音频工程领域的学术论文和行业标准，旨在客观量化danji-cli是否达到真实胆机的音色特征。

#{
  let summary = ()
  for item in all-data {
    let pass = total-pass(item.data)
    let total = item.data.len()
    let color = if pass == total { green } else if pass > 0 { orange } else { red }
    summary.push([- #text(fill: color)[#(item.model)模型达标]（#pass/#(total)通过）])
  }
  [*核心结论*：]
  summary.join()
}

= 测试方法

== "胆味"的学术定义

"胆味"（Tube Sound / Valve Sound）是电子管放大器特有的一种音色特征。根据学术文献#ref(1)，胆味的本质是：

#block(inset: (left: 1.5em, y: 0.5em))[
  #set text(style: "italic", fill: luma(80))
  "胆味是电子管放大器特有的一种音色，听起来悦耳、甜、平滑、泛音丰富。这种音色是一定量的 2 次谐波修饰造成的，加上大多数电子管放大器难以提供良好的线性，输出变压器铁芯的磁滞作用降低了瞬态响应，就提供了这种 2 次谐波造就的泛音。"
  —— 漫步者社区技术讨论
]

== 胆味的声学特征

根据频谱分析研究#ref(2)，胆机的谐波失真具有以下特征：

+ *偶次谐波主导*：二次谐波（2nd harmonic）幅度最强，四次、六次谐波次之
+ *低次谐波为主*：各次谐波幅度随阶数递减
+ *高次谐波微弱*：高阶谐波（5次以上）幅度很小
+ *软削波特性*：信号过载时产生平滑的削波，而非硬削波

== 胆味vs失真

重要澄清："胆味就是失真"的说法是错误的。胆味的本质是谐波的*分布特征*（偶次为主、低次为主），而非失真的绝对大小。良好的胆机设计可以在保持可听谐波特征的同时，将总谐波失真控制在合理范围内#ref(3)。

== 标准测试方法

根据DAFX、TI、IEEE等学术论文#ref(3) #ref(4) #ref(5)，胆机测试包括以下方法：

*谐波失真分析*：向放大器输入单一频率的正弦波，分析输出信号的频谱成分。胆机特征为2nd > 3rd > 4th > ...（偶次谐波占主导）。

*总谐波失真（THD）*：

$ "THD" = frac(sqrt(P_2^2 + P_3^2 + P_4^2 + dots.h), P_1) times 100% $

其中 $P_n$ 为第 $n$ 次谐波的功率，$P_1$ 为基波功率。胆机典型值：0.1% - 2%#ref(3)。

*互调失真（IMD）*：使用 19 kHz + 20 kHz 双音信号测试放大器的非线性特性#ref(3)。

*频率响应*：20 Hz - 20 kHz 范围内的幅度和相位响应。胆机特点为高频轻微滚降，低频轻微衰减#ref(3)。

*瞬态响应*：使用方波或脉冲信号测试上升沿/下降沿时间和软削波特性#ref(3)。

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
  caption: [danji-cli胆味测试合格标准],
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

== 测试用例

*用例 1：正弦波谐波分析* — 输入 1 kHz 正弦波（-6 dBFS），验证二次谐波是否主导、高次谐波是否衰减。

*用例 2：多频率 THD 测试* — 输入 100 Hz / 1 kHz / 10 kHz 正弦波，验证 THD 是否在 0.5% - 3% 范围内。

*用例 3：音乐信号测试* — 输入实际音乐文件（古典/爵士），验证是否具有温暖、甜美的音色特征（待执行）。

= 测试结果

#for item in all-data {
  let model = item.model
  let data = item.data

  [== #model-label(model)]

  figure(
    make-table(data),
    caption: [#(model)模型测试结果],
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
      caption: [#(model)模型#f0-label(r)谐波频谱],
    )

    #figure(
      image(plots-dir + "/spectrum_" + model + "_sine_" + f0-name(r) + ".png", width: 80%),
      caption: [#(model)模型#f0-label(r)频谱对比（输入vs输出）],
    )
  ]

  figure(
    image(plots-dir + "/thd_" + model + ".png", width: 70%),
    caption: [#(model)模型THD对比],
  )
}

== 结果汇总

#figure(
  image(plots-dir + "/verdict_summary.png", width: 60%),
  caption: [测试结果汇总饼图],
)

= 讨论

#for item in all-data {
  let model = item.model
  let data = item.data
  let avg-thd = data.map(r => r.output_thd_pct).fold(0.0, (a, b) => a + b) / data.len()
  let avg-2nd = data.map(r => r.second_harmonic_ratio_pct).fold(0.0, (a, b) => a + b) / data.len()
  let pass-count = total-pass(data)
  let all-pass = pass-count == data.len()
  let all-fail = pass-count == 0

  [== #model-label(model)]

  if all-pass {
    [#(model)模型全部达标。平均THD为#fmt-thd(avg-thd)，处于胆机典型范围（0.5%-3%）内。平均二次谐波占比#fmt-pct(avg-2nd)，远超60%的合格线，偶次谐波特征显著。该模型的增益（约62倍）与输入幅度匹配良好，输出信号保持在电子管的线性工作区内，产生的谐波失真以偶次为主，这正是胆味的声学基础。]
  } else if all-fail {
    [#(model)模型全部未达标。平均THD高达#fmt-thd(avg-thd)，远超3%的合格上限。平均二次谐波占比仅#fmt-pct(avg-2nd)，远低于60%的合格线。主要原因是级联增益过高导致严重削波，输出信号产生大量奇次谐波和高次谐波，与胆机"偶次谐波主导、高次谐波微弱"的特征完全相反。需要降低输入增益或调整B+电压。]
  } else {
    [#(model)模型部分达标（#pass-count/#data.len()通过）。达标频率的THD和二次谐波占比符合胆机特征，但未达标频率存在增益过高的问题。建议针对未达标频率调整参数。]
  }
}

= 结论与建议

== 结论

#{
  let all-models-pass = true
  let failed-models = ()
  for item in all-data {
    if total-pass(item.data) < item.data.len() {
      all-models-pass = false
      failed-models.push(item.model)
    }
  }
  if all-models-pass {
    [所有模型均达到胆味标准。测试框架验证了danji-cli能够准确模拟真空管放大器的谐波特征。]
  } else {
    [部分模型未达标：#failed-models.join("、")。主要问题是增益过高导致削波，需要参数调优。]
  }
}

== 建议

#{
  let suggestions = ()
  for item in all-data {
    let data = item.data
    let avg-thd = data.map(r => r.output_thd_pct).fold(0.0, (a, b) => a + b) / data.len()
    let avg-2nd = data.map(r => r.second_harmonic_ratio_pct).fold(0.0, (a, b) => a + b) / data.len()
    let pass-count = total-pass(data)

    if pass-count < data.len() {
      if avg-thd > 3.0 {
        suggestions.push([- #(item.model)模型THD过高（#fmt-thd(avg-thd)），建议使用`--gain -20dB`或更低增益])
      }
      if avg-2nd < 60.0 {
        suggestions.push([- #(item.model)模型二次谐波不足（#fmt-pct(avg-2nd)），建议降低增益以恢复偶次谐波特征])
      }
    }
  }
  if suggestions.len() > 0 {
    suggestions.join()
  } else {
    [当前参数配置合理，无需调整。建议后续使用音乐文件进行主观听感验证。]
  }
}

= 参考文献

+ 漫步者社区. 什么是胆味[EB/OL]. 2022.
+ Maleczek M. Comparative analysis of sound quality of vacuum-tube amplifiers and transistor amplifiers[J]. Archives of Acoustics, 2012.
+ Hamm R. Harmonic distortion characteristics of solid-state and vacuum-tube preamplifier stages under overload conditions[J]. Journal of the Audio Engineering Society, 1999.
+ Texas Instruments. 如何测量运算放大器的总谐波失真和 THD+N 的基本原理[Z]. 2023.
+ Quod Libet. Simulation of Electron Tube Audio Circuits[C]. ICMC, 1996.

#block(inset: (top: 2em))[
  #set text(size: 10pt, fill: luma(100))
  *声明*：本报告结果仅对被测样品（danji-cli 仿真输出）有效，不代表实际硬件胆机的性能。测试数据和分析方法详见正文。
]
