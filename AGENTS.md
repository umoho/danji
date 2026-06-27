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

### 决策规则

| 项目 | 决策 |
|------|------|
| 语言 | 中文（技术术语保留英文） |
| 格式 | 段落对照（中文在前，英文在后，`---` 分隔） |
| 说明内容 | 含物理单位和取值范围（参数、字段等） |
| 变化规律 | 范围无单位的数值，需说明数值增大时的变化规律（如"越大耦合越强"） |
| Panic | `# Panics`（复数形式），明确写出条件 |
| 内联注释 | 保持英文，不翻译 |
| 私有项 | 全部添加双语注释 |
| 测试函数 | 不添加文档注释 |
| 代码示例 | 注释保持英文 |

### Mod（模块）

使用 `//!` 内部文档注释，段落对照：

```rust
//! 中文模块描述。
//!
//! 中文详细描述（可选）
//!
//! # 子模块
//!
//! - [`mod1`] - 模块1说明
//!
//! # 主要类型
//!
//! - [`Type1`] - 类型1说明
//!
//! ---
//!
//! English module description.
//!
//! English detailed description (optional)
//!
//! # Submodules
//!
//! - [`mod1`] - Module 1 description
//!
//! # Main Types
//!
//! - [`Type1`] - Type 1 description
```

### Struct

使用 `///` 外部文档注释，字段注释为双语：

```rust
/// 中文结构体描述。
///
/// 中文详细描述（可选）
///
/// ---
///
/// English struct description.
///
/// English detailed description (optional)
pub struct Example {
    /// 中文字段说明
    ///
    /// English field description
    pub field: Type,
}
```

### Enum

使用 `///` 外部文档注释，变体注释为双语：

```rust
/// 中文枚举描述。
///
/// ---
///
/// English enum description.
pub enum Example {
    /// 变体1说明
    ///
    /// Variant 1 description
    Variant1,

    /// 变体2说明
    ///
    /// Variant 2 description
    Variant2 {
        /// 字段说明
        ///
        /// Field description
        field: Type,
    },
}
```

### Fn（函数/方法）

使用 `///` 外部文档注释，段落对照：

```rust
/// 中文简短描述。
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
/// ---
///
/// English brief description.
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
pub fn example(param1: Type) -> ReturnType {
    // ...
}
```

### Const（常量）

使用 `///` 外部文档注释，简短描述：

```rust
/// 中文说明。
///
/// ---
///
/// English description.
const NAME: Type = value;
```
