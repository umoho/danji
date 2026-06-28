# 2026-06-28 — 清理 Capacitor.v_prev 死字段

## 做了什么

删除 `Capacitor` 结构体中的死字段 `v_prev: f64`，及其所有写入点。

## 背景

`Capacitor.v_prev` 在初始提交 `c308fc3` 就存在，但求解器从第一天起就使用自己的 `Solver.v_prev[]` 数组（`solver.rs:197–198`），从未读取 `Capacitor.v_prev`。该字段仅被写入（`simulator.rs:772` reset 清零，`simulator.rs:881` 回写电压），属于死存储。

## 改动文件

| 文件 | 改动 |
|------|------|
| `src/circuit/element.rs` | 删除字段定义及 `new()` 中初始化 |
| `src/simulator.rs` | 删除 `reset()` 和 `process_sample()` 中的写入循环 |
| `DESIGN.md` | 更新结构体示例 |

共删除约 14 行，不影响仿真行为。

## 验证结果

- `cargo clippy --all-targets` ✅
- `cargo fmt && cargo test --lib` ✅（18 passed）
