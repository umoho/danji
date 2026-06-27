# 输出设备监听 bug 修复

## 做了什么

对输出设备自动切换功能进行了 4 轮 bug 修复，解决监听逻辑错误、设备引用失效、字节序不匹配等问题。

## Bug 修复

### 1. 虚拟设备过滤 (ee38c2f)

监听线程未过滤 BlackHole 等虚拟设备，导致系统默认输出切换到 BlackHole 时也发送 SwitchOutput，造成输入输出直连。

修复：在发送 SwitchOutput 前检查设备名是否包含 BlackHole/多输出/Aggregate。

### 2. 监听目标错误 (2afe5b5)

监听 `SYSTEM_DEFAULT_OUTPUT` 是错误的——系统输出应始终保持为 BlackHole。改为监听内置设备的 `kAudioDevicePropertyDataSource`，通过 FourCC 码检测耳机插拔（`ispk` → 内置扬声器，`hdpn` → 耳机）。

### 3. cpal Device 引用失效 (dee756c)

启动时存储的 `cpal::Device` 引用在耳机拔出后失效（设备名从"外置耳机"变为"MacBook Pro扬声器"，旧引用指向已断开设备）。

修复：改为存储 `AudioObjectID`（CoreAudio 硬件 ID，不随插拔变化）。ISPK 时按名字匹配内置设备，HDPN 时按排除法找外置设备。

### 4. FourCC 字节序错误 (08eb2e7)

`u32::from_be_bytes` 得到 big-endian 值，但 CoreAudio 在 little-endian 系统上返回 native-endian，永远不相等，监听从未触发。

修复：改为 `u32::from_ne_bytes`。**尚未验证。**

## 验证结果

- `cargo clippy --all-targets` 通过（0 warnings）
- `cargo fmt` 通过
- `cargo test --lib` 18 个测试全部通过

## 踩坑记录

6. **监听目标搞反**：应监听设备插拔（data source），而非系统默认输出切换。系统输出必须保持 BlackHole
7. **cpal Device 是引用，不是 ID**：设备拔出后旧引用失效。应使用 CoreAudio 的 AudioObjectID（硬件 ID 不变）
8. **FourCC 字节序**：CoreAudio 返回 native-endian，不是 big-endian。`from_be_bytes` 永远不匹配
