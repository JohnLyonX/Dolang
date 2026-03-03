# 快速开始

本指南将帮助你在 5 分钟内掌握 Dolang 的基本用法。

## 安装

```bash
# 克隆项目
git clone https://github.com/your-repo/daolang.git
cd daolang

# 编译
cargo build

# 运行 REPL
cargo run
```

## 你的第一个程序

```
Dolang REPL v1.6
>> $>> "Hello, Dolang!";
Hello, Dolang!
>>
```

## REPL 使用说明

REPL（Read-Eval-Print Loop）是交互式解释器，你可以输入代码并立即看到结果。

### 基本操作

- **输入代码**：输入代码后按回车执行
- **历史记录**：使用上下方向键遍历历史命令
- **光标编辑**：使用左右方向键移动光标
- **快捷键**：
  - Ctrl+A：跳转行首
  - Ctrl+E：跳转行末
  - Ctrl+C：取消当前行输入
  - Ctrl+D：退出程序（当输入为空时）

### 多行输入

REPL 支持多行输入。当输入未闭合的括号时，会自动继续读取：

```
>> $fn fib(n) {
>>     $if n <= 1 {
>>         $# n;
>>     } $else {
>>         $# fib(n - 1) + fib(n - 2);
>>     }
>> }
```

输入 `}` 闭合代码块后按回车执行。

## 退出方式

有以下几种方式退出 REPL：

- 输入 `exit`
- 输入 `exit()`
- 输入 `quit`
- 快捷键 Ctrl+D（当输入为空时）

## 第一个程序示例

### Hello World

```dao
$>> "Hello, Dolang!";
```

### 变量声明与使用

```dao
$ name = "Dolang";
$>> "Hello, " + name;
```

### 简单计算

```dao
$>> 1 + 2 * 3;
7

$>> (1 + 2) * 3;
9
```

### 条件判断

```dao
$ x = 10;
$if x > 5 {
    $>> "big";
} $else {
    $>> "small";
}
```

---

现在你已经掌握了 Dolang 的基本用法，可以开始编写自己的脚本了。
