# 2026-06-27 — 清理未使用依赖 & 提升 workspace deps

## 做了什么

1. 分析全 workspace 依赖使用情况，找出未使用的依赖
2. 清理根 crate `danji` 的 7 个未使用依赖
3. 清理 `danji-realtime` 的 1 个未使用依赖
4. 将 4 个跨 crate 复用的依赖提升到 `[workspace.dependencies]`

## 清理的未使用依赖

### 根 crate `danji`
- `wgpu`, `pollster`, `bytemuck` — GPU 计算占位，代码从未引用
- `thiserror` — error.rs 是手写 Display + Error impl
- `serde`, `serde_json` + `serde-config` feature — 未使用
- `plotters` (dev-dep) — examples 和 src 中均无引用

### `danji-realtime`
- `ctrlc` — Cargo.toml 声明但源码无引用

## 提升到 `[workspace.dependencies]` 的依赖

```toml
[workspace.dependencies]
log = "0.4"
env_logger = "0.11"
clap = { version = "4", features = ["derive"] }
hound = "3"
```

- `log` — `danji` + `danji-realtime` 共用
- `env_logger` — `danji` (dev-dep) + `danji-realtime` (dep) 共用
- `clap` — `danji-realtime` + `danji-cli` 共用
- `hound` — `danji-realtime` + `danji-cli` 共用

未提升的：`cpal`（仅 realtime）、`eframe`（仅 ctrl）、`danji = { path = ".." }`（path 依赖保持本地）

## 验证结果

- `cargo check --workspace` ✅
- `cargo test --workspace` ✅ (18 tests passed)
- `cargo clippy --all-targets` ✅ (无 warning)
- `cargo fmt --check` ✅

## 关键决策

- `env_logger` 在根 crate 是 dev-dep 因为只在 examples/ 中使用，Cargo 对 dev target 可见
- workspace.dependencies 只是版本注册表，不会导致未引用的包被编译
- 手写 Error impl 比 thiserror 更轻量，适合当前简单场景
