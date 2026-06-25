# AGENTS.md — danji 工作流程

## 提交前检查

但凡修改了 Rust 代码，提交前必须执行：

```bash
cargo clippy --all-targets
cargo fmt
```

- `cargo clippy`：检查代码质量，无 `warning` 才能提交（忽略 `block v0.1.6` 等第三方依赖警告）
- `cargo fmt`：格式化代码，确保风格一致
- `cargo test --lib`：确保所有单元测试通过

## Dev Log

每完成一段工作后，在 `devlog/` 目录下创建新的日志文件：

```
devlog/YYYY-MM-DD-slug.md
```

- 日期写当天日期
- slug 反映本次工作的主题
- 内容：做了什么、关键决策、验证结果、踩坑记录

## 提交风格

使用 Conventional Commits：

```
<type>: <description>

- bullet points 描述具体变更
- 关键数据或结果
```

`type` 使用 `feat` / `fix` / `chore` 即可，不必太细分。
