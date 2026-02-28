# Dolang

<p align="center">
  <img src="https://img.shields.io/badge/version-v1.1-blue" alt="Version">
  <img src="https://img.shields.io/badge/Rust-1.70+-orange" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
</p>

> *"写代码，就是赚钱。"*

Dolang 是面向独立开发者的快速变现脚本语言。

## 快速开始

### 安装

```bash
cargo install daolang
```

### 运行示例

```bash
# 运行示例程序
daolang run examples/hello.dao

# 进入 REPL
daolang
```

## 核心语法

### 变量声明

```dao
$ x = 10;
$@ CONST = 100;
```

### 函数定义

```dao
$fn add(a, b) {
    $# a + b;
}

$>> add(1, 2);
3
```

### 函数作为值

```dao
$result = $fn(x, y) {
    $# x + y;
};

$result(10, 20);
30
```

## 文档

详细文档请参阅 [docs.md](docs.md)。

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件。

---

**Dolang Team · v1.1**
