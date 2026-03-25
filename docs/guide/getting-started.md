# 快速开始

本指南帮助你在 5 分钟内上手 Dolang。

---

## 安装

```bash
# 克隆项目
git clone https://github.com/your-repo/dolang.git
cd dolang

# 编译
cargo build --release

# 将二进制加入 PATH（可选）
export PATH="$PATH:$(pwd)/target/release"
```

---

## 运行你的第一个程序

创建文件 `hello.dol`：

```dolang
$>> "Hello, Dolang!";
```

运行：

```bash
dolang run hello.dol
# 输出: Hello, Dolang!
```

---

## 使用 REPL

REPL 是交互式解释器，适合快速验证代码：

```bash
dolang repl
```

```
Dolang REPL v2026
>> $>> "Hello!";
Hello!
>> $ x = 1 + 2;
>> $>> x;
3
>> exit
```

**REPL 快捷键**

| 快捷键 | 功能 |
|--------|------|
| `↑` / `↓` | 历史记录 |
| `Ctrl+A` | 跳到行首 |
| `Ctrl+E` | 跳到行末 |
| `Ctrl+C` | 取消当前输入 |
| `Ctrl+D` | 退出（输入为空时） |

输入 `exit` 或 `quit` 也可退出。

---

## 基本语法速览

### 变量与常量

```dolang
$ name = "Dolang";       // 变量（可修改）
$@ PI = 3.14159;         // 常量（不可修改）
```

### 输出

```dolang
$>> "Hello, World!";
$>> 42;
$>> name;
```

### 注释

```dolang
// 这是单行注释
```

### 函数

```dolang
$fn add(a, b) {
    $# a + b;            // $# 是 return
}

$>> add(3, 4);           // 7
```

### 条件

```dolang
$ score = 85;
$if score >= 90 {
    $>> "A";
} $else $if score >= 80 {
    $>> "B";
} $else {
    $>> "C";
}
```

### 循环

```dolang
// for-in 遍历列表
$for item in [1, 2, 3] {
    $>> item;
}

// while 循环
$ i = 0;
$while i < 3 {
    $>> i;
    i += 1;
}
```

---

## 下一步

- [实用示例](examples.md) — 更多代码示例
- [标准库参考](stdlib.md) — 内置方法与标准库模块
- [模块系统](modules.md) — 导入和组织代码
- [Web 服务](web.md) — 构建 HTTP API
