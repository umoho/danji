# danji-realtime + danji-ctrl: 实时胆机模拟

实现了基于 BlackHole + CPAL 的实时音频处理，以及 daemon + ctrl 两进程架构。

## 踩坑记录

### 1. BlackHole 输入音量

所有路由都对了，WAV 录制也有数据，但实时播放无声。原因是 BlackHole 的输入音量（Audio MIDI Setup 中 BlackHole 2ch 的 Input Volume）默认 0.573（-27.33 dB）。系统音频进入 BlackHole 时已被衰减 27dB，经 danji 处理后几乎不可闻。

调试音频通路时，应先录制 WAV + 分析波形确定信号量级，而非靠耳朵。

### 2. 采样率匹配

Bypass 模式有声音但伴随爆音。BlackHole 输入 48kHz，MacBook 扬声器默认 44.1kHz。`mpsc::sync_channel` 在两种速率间传递数据时输入快于输出，缓冲区溢出导致丢数据。强制输出 config 与输入采样率一致即可修复。

实时音频中采样率必须匹配，否则任何缓冲区方案都无法避免爆音。

### 3. 数据通道选择

尝试过 ringbuf、Arc<Mutex<Vec>>、mpsc::sync_channel。最终 mpsc 在同采样率下工作稳定。

### 4. 测试音失真

1kHz 测试音像方波——输出 callback 每次调用时相位重置，波形在 buffer 边界不连续。未修复（仅用于调试）。

## 架构演进

### 第一阶段：单进程直通

```
输入 callback (BlackHole) → mpsc channel → 输出 callback (扬声器)
```

### 第二阶段：daemon + ctrl 两进程

```
danji-realtime (daemon)
  ├── 音频引擎 (CPAL + Simulator)
  ├── 命令循环 (主线程)
  └── Unix socket server

danji-ctrl (egui GUI)
  └── 连接 socket → 发命令 → 收响应
```

热参数（bypass, volume, gain, bplus）通过 AtomicU32/AtomicBool 共享，音频 callback 无锁读取。
重载参数（tube, model）通过 mpsc channel 发往主线程，暂停流 → 重建引擎 → 重启流。

### 参数一览

| 参数 | 默认值 | 范围 | 切换方式 |
|------|--------|------|----------|
| volume | -12 dB | -60 ~ +12 | 热（atomic） |
| gain | -38 dB | -96 ~ +10 | 热（atomic） |
| bplus | 300 V | 50 ~ 600 | 热（atomic） |
| bypass | off | on/off | 热（atomic） |
| tube | 12AX7 | 8 种 | 热（atomic + Mutex 保护引擎） |
| model | single | single/two-stage/chain | 冷（重建流 + 引擎） |

### 电子管型号

12AX7, 12AU7, 12AT7, 6DJ8, 6L6GC, 6550, EL34, KT88

### 模型状态

- single：已实现
- two-stage：声明但未实现
- chain：声明但未实现

## 验证

- `cargo build -p danji-realtime` ✅
- `cargo build -p danji-ctrl` ✅
- `cargo test --lib`（18 passed）✅
- `cargo clippy --all-targets`（仅 pre-existing warnings）✅
- `cargo fmt` ✅
