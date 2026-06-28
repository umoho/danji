# 2026-06-28 — 死代码清理：GpuInit & Capacitor.v_prev

## 做了什么

1. 删除 `DanjiError::GpuInit(String)` 死代码变体
2. 删除 `src/gpu/shaders` 空目录（连同空父目录 `src/gpu/`）

## 清理详情

### DanjiError::GpuInit

- `src/error.rs` — 删除变体定义（4 行文档 + 声明）及 `Display` 匹配分支
- `DESIGN.md` — 更新示例代码，删除 `GpuInit(String)` 行
- 阶段 1 GPU 依赖（wgpu/pollster/bytemuck）早已移除后的残留

## 验证结果

- `cargo clippy --all-targets` ✅（无 warning）
- `cargo fmt && cargo test --lib` ✅（18 tests passed）
