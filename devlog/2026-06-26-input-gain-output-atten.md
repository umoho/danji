# 修复输入增益 + 输出衰减

## 问题

`danji-realtime` 中：

1. **`gain` 参数从未被使用**。`engine.rs` 的音频回调中读取了 `volume`，
   但遗漏了 `gain`，导致输入信号未经缩放直接送入管子。两级 12AX7
   增益 ~25600x，0dBFS 输入直接削波。

2. **缺少输出固定衰减**。屏极 AC 摆动 ~200Vpk，未经衰减直接 × Volume
   后 clamp[-1,1]，Volume 同时扮演输出衰减和音量控制两个角色。
   调小 Volume 可以减少削波但不是正确做法。

## 修复

在 `danji-realtime/src/engine.rs` 的音频回调中：

```rust
// 前
let l_in = frame[0];
let l_out = (l_raw * vol).clamp(-1.0, 1.0);

// 后
let l_in = frame[0] * gain;                    // 输入增益
let output_atten = 1.0 / 300.0;                 // 固定输出衰减
let l_out = (l_raw * output_atten * vol).clamp(-1.0, 1.0);
```

## 信号链变化

```
前: l_in → 胆机 raw → DcBlocker → × vol → clamp
后: l_in × gain → 胆机 raw → DcBlocker → × output_atten(1/300) → × vol → clamp
```

- `gain` 默认 -38dB（≈0.013），可通过 ctrl 调节
- `output_atten = 1/300` 模拟输出变压器降压，固定不暴露

## 效果

削波明显减少，Volume 回归单纯的音量控制。不涉及库代码。
