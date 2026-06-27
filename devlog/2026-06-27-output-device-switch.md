# 输出设备自动切换

## 做了什么

为 danji-realtime 添加了输出设备自动切换功能，支持耳机插拔时自动切换音频输出。

## 关键改动

### 1. 依赖 (Cargo.toml)

新增 `coreaudio-sys` 依赖（仅 macOS）：

```toml
[target.'cfg(target_os = "macos")'.dependencies]
coreaudio-sys = "0.2"
```

最初尝试使用高层封装 `coreaudio` crate，但存在 `kAudioFormatAPAC` 编译错误（coreaudio-sys 未导出该符号），最终直接使用 `coreaudio-sys`。

### 2. 命令变体 (params.rs)

`MainCommand` 新增 `SwitchOutput` 变体：

```rust
SwitchOutput { device: cpal::Device }
```

fire-and-forget 模式，无需响应通道。

### 3. 设备选择改进 (engine.rs)

`output_device()` 改为优先使用 `host.default_output_device()`，保留 BlackHole/多输出/Aggregate 排除过滤。回退到遍历所有设备。

### 4. 流切换 (engine.rs)

`run_engine()` inner loop 新增 `SwitchOutput` 处理：
- Drop 旧 output stream
- 用新 device 重建 output stream
- 不 break 外层循环，引擎继续运行

为支持流重建，`rx` 改为 `Arc<Mutex<Receiver>>` 共享。

### 5. 设备监听线程 (main.rs)

新增 `monitor_default_output()` 函数：
- 使用 `coreaudio-sys` 的 `AudioObjectGetPropertyData` 轮询 `kAudioHardwarePropertyDefaultOutputDevice`
- 每 200ms 检查一次系统默认输出设备 ID
- 变化时通过 `cmd_tx` 发送 `SwitchOutput` 命令
- 非 macOS 平台为空实现

原计划使用 `coreaudio` crate 的 `add_listener(SYSTEM_DEFAULT_OUTPUT)` 做事件驱动，但该 crate 存在编译问题（见踩坑 #1），降级为轮询方案。200ms 间隔对耳机插拔场景足够，CPU 开销可忽略。

### 6. 文档注释

AGENTS.md 更新后，为新增代码补齐双语文档注释：
- `MainCommand::SwitchOutput` 变体
- `monitor_default_output()` 函数
- `get_default_output_id()` 内部函数
- 非 macOS 空实现 stub

## 验证结果

- `cargo clippy --all-targets` 通过（0 warnings）
- `cargo fmt` 通过
- `cargo test --lib` 18 个测试全部通过

## 踩坑记录

1. **coreaudio crate 编译失败**：v0.2.17~0.2.19 均引用不存在的 `kAudioFormatAPAC`，这是 crate 与 coreaudio-sys 的兼容性 bug。最终直接使用 `coreaudio-sys`
2. **rx 所有权**：output stream 闭包 move 了 `rx`，SwitchOutput 重建流时无法复用。解决方案：`Arc<Mutex<Receiver>>`
3. **cmd_tx 所有权**：socket 线程 move 了 `cmd_tx`，监听线程无法再 clone。解决方案：先 clone 再分别 move
4. **AudioObjectGetPropertyData 参数类型**：需要 `*mut c_void` 而非 `&mut AudioObjectID`
5. **cpal `name()` 已弃用**：需改用 `description().name()` 获取设备名称
