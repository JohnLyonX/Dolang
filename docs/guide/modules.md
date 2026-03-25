# 模块系统

Dolang 的模块系统分为三层：**native 内置模块**、**纯 .dol 标准库模块**、**项目自定义模块**。

---

## 导入模块

### 导入标准库模块

```dolang
$mod std.math;
$mod std.fs;
$mod std.json;
```

导入后，模块以最后一级名称作为命名空间前缀使用：

```dolang
$mod std.math;
$>> math.sqrt(9.0);    // 3.0

$mod std.fs;
$ ok = fs.exists("./data.txt");
```

### 导入子模块

```dolang
$mod std.math.stats;   // 对应 stdlib/math/stats.dol
$mod std.str.fmt;      // 对应 stdlib/str/fmt.dol
$mod std.core.iter;    // 对应 stdlib/core/iter.dol
```

```dolang
$mod std.core.iter;
$for i in iter.range(0, 5) {
    $>> i;             // 0 1 2 3 4
}
```

---

## 自定义模块

在项目目录中创建 `.dol` 文件，即可作为模块使用。

### 项目结构

```
my-app/
├── package.toml
├── main.dol
└── shared/
    └── helper.dol
```

### `shared/helper.dol`

```dolang
$fn greet(name) -> String {
    $# f"Hello, {name}!";
}

// 私有函数，不可被外部调用
_$fn internal_helper(x) {
    $# x * 2;
}
```

### `main.dol`

```dolang
$mod shared.helper;

$>> helper.greet("Dolang");    // Hello, Dolang!
```

---

## 公有函数与私有函数

| 声明方式 | 可见性 | 说明 |
|----------|--------|------|
| `$fn name(...) { }` | 公有 | 可被外部模块调用 |
| `_$fn name(...) { }` | 私有 | 仅限当前文件内部使用 |

```dolang
// utils.dol
$fn public_api(x) {
    $# _transform(x) * 2;
}

_$fn _transform(x) {    // 私有，外部不可访问
    $# x + 1;
}
```

---

## 模块解析顺序

当你写 `$mod foo.bar;` 时，解释器按以下顺序查找：

1. **Native 注册表** — Rust 实现的内置模块（如 `std.math`、`std.fs`）
2. **`stdlib/` 目录** — 纯 .dol 实现的标准库（如 `std.core.iter` → `stdlib/core/iter.dol`）
3. **项目本地文件** — 当前项目中的 `.dol` 文件（如 `shared.helper` → `shared/helper.dol`）

`std` 前缀由解释器特殊处理，会映射到标准库目录。

---

## 项目配置文件

在项目根目录创建 `package.toml` 可以配置项目元信息、环境变量和服务器参数：

```toml
name = "my-app"
version = "0.1.0"
entry = "main.dol"

[env]
APP_NAME = "My Dolang App"
DB_HOST = "localhost"

[server]
port = 3000
host = "0.0.0.0"
```

通过 `$<<CONFIG("KEY")` 读取 `[env]` 中的值：

```dolang
$ app_name = $<<CONFIG("APP_NAME");
$>> app_name;    // My Dolang App
```

运行项目（自动加载 `package.toml`）：

```bash
dolang run .
dolang serve .
```

---

## 通配符导入

使用 `.*` 导入模块中所有公有函数到当前命名空间（不带前缀）：

```dolang
$mod std.core.iter.*;

$for i in range(0, 3) {    // 直接使用 range，无需 iter. 前缀
    $>> i;
}
```
