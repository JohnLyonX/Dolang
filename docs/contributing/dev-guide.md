# 开发者贡献指南

本文档面向希望为 Dolang 项目贡献代码的开发者。

## 项目结构

Dolang 核心实现采用模块化设计，代码结构如下：

```
src/
├── main.rs              # 程序入口
├── cli.rs               # CLI 参数处理
├── repl.rs              # REPL 交互逻辑
├── token.rs             # 词法单元定义
├── lexer.rs             # 词法分析器
├── parser/              # 语法解析器模块
│   ├── mod.rs           # 统一导出
│   ├── expr.rs          # 表达式解析
│   ├── stmt.rs          # 语句解析
│   └── literal.rs       # 字面量解析
├── ast.rs               # 抽象语法树
├── interpreter/          # 解释执行模块
│   ├── mod.rs           # 统一导出
│   ├── env.rs           # 环境类型管理
│   ├── eval.rs          # 表达式求值
│   └── exec.rs          # 语句执行
├── error.rs             # 错误类型定义
└── syntax.rs            # 语法验证
```

## 开发环境搭建

### 环境要求

- Rust 1.70+
- Cargo（Rust 包管理器）

### 克隆项目

```bash
git clone https://github.com/your-repo/daolang.git
cd daolang
```

## 编译运行

### 编译项目

```bash
cargo build
```

### 运行 REPL

```bash
cargo run
```

### 直接运行二进制

```bash
./target/debug/daolang
```

### 调试模式运行

```bash
cargo run -- your_script.dol
```

## 测试

### 运行所有测试

```bash
cargo test
```

### 运行特定测试

```bash
cargo test test_name
```

### 检查代码格式

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

## 代码风格

### 命名规范

- 变量和函数：使用小写下划线命名法（snake_case）
- 常量：使用大写下划线命名法（SCREAMING_SNAKE_CASE）
- 文件名：使用小写下划线命名法

### 提交规范

提交信息应清晰描述所做的更改：

```
feat: add for-in iteration syntax
fix: resolve variable shadowing issue
docs: update README with installation instructions
```

### 代码审查

- 确保所有测试通过
- 遵循现有代码风格
- 添加必要的注释说明复杂逻辑

## 模块说明

### 词法分析器 (lexer)

负责将源代码字符串转换为 token 序列。

### 语法解析器 (parser)

负责将 token 序列转换为抽象语法树（AST）。

### 解释器 (interpreter)

负责执行 AST，得到运行结果。

### 错误处理 (error)

统一定义和处理各类错误。

## 常见任务

### 添加新的数据类型

1. 在 `lexer.rs` 中添加新的字面量识别
2. 在 `parser/literal.rs` 中添加解析逻辑
3. 在 `ast.rs` 中定义新的 AST 节点
4. 在 `interpreter/eval.rs` 中添加求值逻辑
5. 在 `docs/reference/types.md` 中添加文档

### 添加新的运算符

1. 在 `lexer.rs` 中添加 token 定义
2. 在 `parser/expr.rs` 中添加解析
3. 在 `interpreter/eval.rs` 中添加求值逻辑
4. 在 `docs/reference/operators.md` 中添加文档

### 添加新的内置方法

1. 在 `interpreter/eval.rs` 中实现方法
2. 在 `docs/reference/types.md` 中添加文档

---

感谢你对 Dolang 项目的贡献！
