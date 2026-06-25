// danji-cli 胆味测试方法文档
// 基于学术论文和行业标准

#set document(
  title: "danji-cli 胆味测试方法",
  author: "danji 开发团队",
  date: datetime(year: 2026, month: 6, day: 25),
)

#set page(
  paper: "a4",
  margin: 2.5cm,
  header: [
    #set text(size: 8pt, fill: gray)
    #h(1fr) danji-cli 测试文档
  ],
  footer: [
    #set text(size: 8pt, fill: gray)
    #h(1fr) #counter(page).display("1 / 1", both: true)
  ],
)

#set text(
  font: ("Noto Serif CJK SC", "Noto Serif"),
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

= 引言

本文档定义了 danji-cli（胆机放大器仿真工具）的"胆味"测试方法。测试基于音频工程领域的学术论文和行业标准，旨在客观量化 danji-cli 是否达到真实胆机的音色特征。

= "胆味"的学术定义

== 定义

"胆味"（Tube Sound / Valve Sound）是电子管放大器特有的一种音色特征。根据学术文献，胆味的本质是：

#blockquote[
  "胆味是电子管放大器特有的一种音色，听起来悦耳、甜、平滑、泛音丰富。这种音色是一定量的 2 次谐波修饰造成的，加上大多数电子管放大器难以提供良好的线性，输出变压器铁芯的磁滞作用降低了瞬态响应，就提供了这种 2 次谐波造就的泛音。"
  —— 漫步者社区技术讨论
]

== 胆味的声学特征

根据频谱分析研究，胆机的谐波失真具有以下特征：

+ *偶次谐波主导*：二次谐波（2nd harmonic）幅度最强，四次、六次谐波次之
+ *低次谐波为主*：各次谐波幅度随阶数递减
+ *高次谐波微弱*：高阶谐波（5次以上）幅度很小
+ *软削波特性*：信号过载时产生平滑的削波，而非硬削波

== 胆味 vs 失真

重要澄清："胆味就是失真"的说法是错误的。胆味的本质是谐波的*分布特征*（偶次为主、低次为主），而非失真的绝对大小。良好的胆机设计可以在保持可听谐波特征的同时，将总谐波失真控制在合理范围内。

= 胆机的标准测试方法

根据 DAFX、TI、IEEE 等学术论文，胆机测试包括以下方法：

== 谐波失真分析（Harmonic Distortion Analysis）

*测试原理*：向放大器输入单一频率的正弦波，分析输出信号的频谱成分。

*测试信号*：1 kHz 正弦波（标准参考频率）

*测量内容*：
- 各次谐波的幅度（2nd, 3rd, 4th, 5th...）
- 谐波幅度与基波的比例
- 谐波分布曲线

*判断标准*：
- 胆机特征：2nd > 3rd > 4th > ...（偶次谐波占主导）
- 晶体机特征：奇次谐波（3rd, 5th）幅度较高

*参考文献*：#footnote[Hamm, "Harmonic distortion characteristics of solid-state and vacuum-tube preamplifier stages under overload conditions"]

== 总谐波失真（THD）

*计算公式*：

$ "THD" = frac(sqrt(P_2^2 + P_3^2 + P_4^2 + dots.h), P_1) times 100% $

其中 $P_n$ 为第 $n$ 次谐波的功率，$P_1$ 为基波功率。

*典型值*：
- 胆机：0.1% - 2%（较高，但听感悦耳）
- 晶体机：< 0.01%（极低，但听感生硬）

*参考文献*：#footnote[Texas Instruments, "如何测量运算放大器的总谐波失真和 THD+N 的基本原理", 2023]

== 互调失真（IMD）

*测试原理*：使用双音信号测试放大器的非线性特性。

*测试信号*：19 kHz + 20 kHz 双音信号（Leinonen et al. 方法）

*测量内容*：
- 差频成分（1 kHz）
- 互调失真分量

*参考文献*：#footnote[Leinonen et al., "A Measurement Technique for IMD Analysis"; DATK Software Tool]

== 频率响应

*测试范围*：20 Hz - 20 kHz

*测量内容*：
- 幅度响应曲线
- 相位响应曲线

*胆机特点*：
- 高频有轻微滚降（输出变压器特性）
- 低频有轻微衰减（RC 耦合网络）

*参考文献*：#footnote[Maleczek, "Comparative analysis of sound quality of vacuum-tube amplifiers and transistor amplifiers", Archives of Acoustics, 2012]

== 瞬态响应

*测试信号*：方波或脉冲信号

*测量内容*：
- 上升沿/下降沿时间
- 过冲和振铃
- 软削波特性

*参考文献*：#footnote[Quod Libet, "Simulation of Electron Tube Audio Circuits", ICMC 1996]

= danji-cli 测试计划

== 测试环境

- *测试工具*：danji-cli（Rust 实现的胆机仿真工具）
- *测试模型*：single / two-stage / chain（三种放大器模型）
- *输入格式*：16-bit PCM WAV, 44.1 kHz
- *分析工具*：Python (numpy, scipy, matplotlib) 或专业音频分析软件

== 测试用例

=== 用例 1：正弦波谐波分析
- *输入*：1 kHz 正弦波，-6 dBFS
- *输出*：FFT 频谱分析
- *验证*：二次谐波是否主导，高次谐波是否衰减

=== 用例 2：多频率 THD 测试
- *输入*：100 Hz, 1 kHz, 10 kHz 正弦波
- *输出*：THD 计算
- *验证*：THD 是否在 0.1% - 2% 范围内

=== 用例 3：音乐信号测试
- *输入*：实际音乐文件（古典/爵士）
- *输出*：主观听感 + 频谱对比
- *验证*：是否具有温暖、甜美的音色特征

== 预期结果

根据 danji-cli 的物理模型（基于 Child-Langmuir 定律和 Koren 改进模型），预期：

+ 二次谐波占主导（偶次谐波特征）
+ 谐波幅度随阶数递减
+ THD 在合理范围内（0.5% - 2%）
+ 高频有轻微滚降
+ 软削波特性明显

= 合格标准

根据学术文献和行业经验，danji-cli 达到"胆味"的合格标准如下：

== 客观指标

#figure(
  table(
    columns: (auto, auto, auto),
    align: (left, center, left),
    table.header(
      [*指标*], [*合格标准*], [*依据*],
    ),
    [二次谐波占比], [≥ 60%（在所有谐波中）], [胆机以偶次谐波为主],
    [谐波衰减], [随阶数递减（2nd > 3rd > 4th）], [胆机低次谐波特征],
    [THD], [0.5% - 3%], [胆机典型范围，过低无胆味],
    [五次以上谐波], [< 5%（占总谐波）], [高次谐波应微弱],
  ),
  caption: [danji-cli 胆味测试合格标准],
)

== 主观指标（辅助参考）

- 声音温暖、不刺耳
- 有"甜腻"感
- 高频平滑
- 无明显数字感或刺耳感

== 判定规则

- *合格*：所有客观指标均达标
- *基本合格*：3 项客观指标达标，1 项轻微不达标（偏差 < 10%）
- *不合格*：2 项及以上客观指标不达标

= 参考文献

#bibliography("references.bib", title: "参考文献")

= 附录：测试脚本模板

```python
# test_harmonic_distortion.py
import numpy as np
from scipy import fft
import soundfile as sf

def analyze_harmonics(input_file, output_file, fundamental_freq=1000):
    """分析输入/输出音频的谐波成分"""
    # 读取音频文件
    input_signal, sr = sf.read(input_file)
    output_signal, sr = sf.read(output_file)
    
    # FFT 分析
    N = len(input_signal)
    input_spectrum = np.abs(fft.fft(input_signal))[:N//2]
    output_spectrum = np.abs(fft.fft(output_signal))[:N//2]
    
    # 提取谐波幅度
    freqs = np.fft.fftfreq(N, 1/sr)[:N//2]
    harmonic_indices = []
    for i in range(1, 11):  # 1-10 次谐波
        idx = np.argmin(np.abs(freqs - fundamental_freq * i))
        harmonic_indices.append(idx)
    
    # 计算谐波比例
    fundamental_amp = output_spectrum[harmonic_indices[0]]
    harmonics = output_spectrum[harmonic_indices] / fundamental_amp
    
    return harmonics

def calculate_thd(harmonics):
    """计算总谐波失真"""
    thd = np.sqrt(np.sum(harmonics[1:]**2)) / harmonics[0]
    return thd * 100  # 百分比
```
