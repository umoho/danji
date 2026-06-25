# Workspace 重组

将单 crate 项目拆分为 workspace 结构，为 danji-realtime 做准备。

## 变更

- 根 `Cargo.toml` 改为 `[workspace]` + `[package]`（danji library 本身作为 member）
- `src/bin/danji-cli.rs` 移出到独立 crate `danji-cli/`
- 新建 `danji-realtime/` crate（占位）
- 清理 library 的依赖：`hound`、`clap` 移到 danji-cli，`cpal` 从 dev-dependencies 移除

## 验证

- `cargo build` ✅
- `cargo test --lib`（18 passed）✅
- `cargo clippy --all-targets`（仅 pre-existing warnings）✅
- `cargo fmt` ✅
