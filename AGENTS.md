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

使用 Conventional Commits，**提交消息必须使用英文**：

```
<type>: <description>

- bullet points describing specific changes
- key data or results
```

`type` 使用 `feat` / `fix` / `chore` 即可，不必太细分。

## 文档注释格式

所有代码的文档注释必须遵循以下格式规范。

### 基本结构

使用段落对照方式（中文在前，英文在后），以 `---` 分隔：

```rust
/// 中文简短描述
///
/// 中文详细描述（可选）
///
/// # 参数
///
/// * `param1` - 参数1说明（单位：xxx，范围：xxx ~ xxx）
///
/// # 返回值
///
/// 返回值说明
///
/// # Panics
///
/// 可能 panic 的条件说明
///
/// # 示例
///
/// ```
/// // 注释保持英文
/// ```
///
/// ---
///
/// English brief description
///
/// English detailed description (optional)
///
/// # Arguments
///
/// * `param1` - Parameter 1 description (unit: xxx, range: xxx ~ xxx)
///
/// # Returns
///
/// Return value description
///
/// # Panics
///
/// Conditions that may cause panic
///
/// # Examples
///
/// ```
/// // Comments stay in English
/// ```
```

### 章节标题

使用 Rust 标准章节（复数形式）：

| 章节 | 用途 |
|------|------|
| `# 参数` | 参数说明（含物理单位和取值范围） |
| `# 返回值` | 返回值说明 |
| `# Panics` | 可能 panic 的条件（复数形式） |
| `# 示例` | 代码示例 |

### 决策规则

| 项目 | 决策 |
|------|------|
| 语言 | 中文（技术术语保留英文） |
| 格式 | 段落对照（中文在前，英文在后，`---` 分隔） |
| 参数说明 | 含物理单位和取值范围 |
| Panic | `# Panics`（复数形式），明确写出条件 |
| 内联注释 | 保持英文，不翻译 |
| 私有函数 | 全部添加双语注释 |
| 测试函数 | 不添加文档注释 |
| 示例代码 | 注释保持英文 |

### 示例

```rust
/// 添加电阻元件。
///
/// 在节点 `a` 和 `b` 之间添加一个电阻值为 `ohms` 的电阻。
///
/// # 参数
///
/// * `a` - 节点 A
/// * `b` - 节点 B
/// * `ohms` - 电阻值（单位：欧姆，范围：0.0 ~ 1e9）
///
/// # 返回值
///
/// 返回自身引用，支持链式调用
///
/// ---
///
/// Add a resistor element.
///
/// Adds a resistor with value `ohms` between nodes `a` and `b`.
///
/// # Arguments
///
/// * `a` - Node A
/// * `b` - Node B
/// * `ohms` - Resistance (unit: ohms, range: 0.0 ~ 1e9)
///
/// # Returns
///
/// Returns self reference for method chaining
pub fn add_resistor(&mut self, a: NodeId, b: NodeId, ohms: f64) -> &mut Self {
    self.resistors.push(Resistor::new(a, b, ohms));
    self
}
```
