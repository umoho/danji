# danji-realtime: 实时音频处理

实现了基于 BlackHole + CPAL 的实时音频处理，系统声音经过 12AX7 单级模拟后播放。

## 踩坑记录

### 1. 音量坑（耗时最久）

**现象**：所有路由都对了，WAV 录制也有数据，但实时播放无声。

**原因**：BlackHole 的**输入音量**（Audio MIDI Setup 中选中 BlackHole 2ch，下方 Input Volume 滑块）默认 0.573（-27.33 dB）。系统音频进入 BlackHole 时已被衰减 27dB，再经 danji 处理后几乎不可闻。

**教训**：调试音频通路时，应先通过录制 WAV + 分析波形确认信号量级，而不是靠耳朵。

### 2. 采样率匹配

**现象**：bypass 模式有声音但伴随爆音/哒哒声（"拍拍音"）。

**原因**：BlackHole 输入 48kHz，MacBook 扬声器默认 44.1kHz。用 `mpsc::sync_channel` 在两种速率间传递数据，输入产生快于输出消耗 → 缓冲区溢出 → 丢数据 → 爆音。

**修复**：强制输出 config 与输入采样率一致（48kHz）。扬声器支持 48kHz。

**教训**：实时音频中采样率必须匹配，否则任何缓冲区方案都无法避免问题。

### 3. 数据通道选择

**历程**：
- `ringbuf` crate：先弃用（调试复杂，不确定是否真的有问题）
- `Arc<Mutex<Vec>>`：输入/输出 callback 竞争锁导致数据丢失
- `mpsc::sync_channel`：最终方案，在采样率匹配后工作稳定

**教训**：标准库 `mpsc` 虽不是为实时音频设计，但在同速率下工作良好。`ringbuf` 可能本也可以，但调试成本太高，先求可行再求最优。

### 4. 测试音正弦波失真

**现象**：1kHz 测试音听起来像方波/锯齿波。

**原因**：输出 callback 每次调用时 `phase` 重置为 0，导致波形在每个 buffer 边界不连续。

**未修复**：测试音仅用于调试，不影响主要功能。

## 现状架构

```
输入 callback (BlackHole 48kHz)
  → Simulator::process_sample (12AX7 × 2, L/R)
  → DcBlocker (10Hz HPF 去屏极 DC)
  → volume scaling + clamp
  → mpsc::sync_channel
  → 输出 callback (扬声器 48kHz)
```

## 命令行

- `cargo run -p danji-realtime`：默认模式（12AX7 处理，-12dB）
- `--bypass`：直通模式（无电子管处理）
- `--test-tone`：1kHz 正弦波测试
- `--capture <file> <secs>`：录制 WAV 到文件
- `--analyze <file>`：分析 WAV 文件统计信息

## 验证

- `cargo build` ✅
- `cargo clippy --all-targets`（仅 pre-existing warnings）✅
- `cargo test --lib`（18 passed）✅
- `cargo fmt` ✅

# 后续架构演进：daemon + ctrl

## 架构

```
danji-realtime (daemon)
  ├── 音频引擎 (CPAL + Simulator)
  ├── Unix socket server (/tmp/danji.sock)
  └── 命令循环 (主线程)

danji-ctrl (egui GUI)
  └── 连接 socket → 发命令 → 收响应
```

## daemon 侧

- **热参数**（bypass, volume, gain, bplus）：通过 AtomicU32/AtomicBool 共享，音频 callback 无锁读取
- **重载参数**（tube, model）：通过 mpsc channel 发送到主线程，主线程暂停流 → 重建引擎 → 重启流
- **Socket 协议**：纯文本，一行一个命令，响应 `OK\n` 或 `ERROR <msg>\n`

## ctrl 侧

- egui 调音台界面
- 滑块：Volume (-60~+12 dB), Gain (-96~+10 dB), B+ (50~600 V)
- 下拉菜单：Model (single/two-stage/chain), Tube (8 种型号)
- 按钮：Bypass on/off, Stop Daemon
- 后台线程定时拉取 status 保持界面同步

## 命令协议

```
bypass on|off          ← bypass 绕过
volume <dB>            ← 输出音量 (-60 ~ +12)
gain <dB>              ← 输入增益 (-96 ~ +10)
bplus <V>              ← B+ 电压 (50 ~ 600)
tube <type>            ← 换管 (12AX7/12AU7/...)
model <name>           ← 换模型 (single/two-stage/chain)
status                 ← 返回当前所有参数
stop                   ← 停止 daemon
```

## 验证

- `cargo build -p danji-realtime` ✅
- `cargo build -p danji-ctrl` ✅
- `cargo build -p danji-cli` ✅
- `cargo test --lib`（18 passed）✅
