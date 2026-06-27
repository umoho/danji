# 输出设备监听调试与最终方案

## 调试过程

通过临时调试程序 `monitor_test.rs`（197 行）逐步排查 CoreAudio 设备行为，最终发现正确的监听方式。

## 关键发现

1. **耳机接口是动态设备**：插入时出现新设备，拔出时设备消失。设备 ID 每次都变（128→135→142→149）
2. **新设备的 data source 是 0**，不是 `hdpn`。无法通过 data source 判断是否为耳机
3. **扬声器设备的 data source 不变**：始终为 `ispk`，不随插拔变化
4. **不需要检查 FourCC 或 transport type**：只看设备列表增删即可

## 最终方案

监听 `kAudioHardwarePropertyDevices` 设备列表变化：
- 设备增加 → 耳机插入 → 切到外置设备
- 设备减少 → 耳机拔出 → 切到内置扬声器

设备匹配增加 `default_output_config().is_ok()` 过滤，排除输入设备（麦克风）。

## 调试程序

`danji-realtime/src/bin/monitor_test.rs`（197 行），已单独提交到 git 历史。

## 调试输出示例

```
[initial] 3 devices:
  ID=86 [other] ds=0x00000000 (    ) [none]
  ID=81 [builtin] ds=0x00000000 (    ) [none]
  ID=74 [builtin] ds=0x00000000 (    ) [none]

[  4271ms] tick=21 DEVICE LIST CHANGED: +1 -0
  + ID=128 [builtin] ds=0x00000000 (    ) [none]
  Headphone present: false
[  6514ms] tick=32 DEVICE LIST CHANGED: +0 -1
  - ID=128
```

## 验证结果

- `cargo clippy --all-targets` 通过（0 warnings）
- `cargo fmt` 通过
- `cargo test --lib` 18 个测试全部通过
- 实际插拔测试：danji-realtime 正确切换到 "MacBook Pro扬声器" / "外置耳机"
