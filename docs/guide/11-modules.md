# 11. 模块系统

Dolang 通过 `$mod` 组织可复用代码。只要脚本开始变大，你几乎一定会进入模块拆分。

## 导入模块

```dol
$mod shared.helper;
$>> helper.answer();
```

模块路径使用点号分隔，对应文件系统中的 `.dol` 文件。

例如下面的目录：

```text
app/
├── main.dol
└── shared/
    └── helper.dol
```

`main.dol`：

```dol
$mod shared.helper;
$>> helper.answer();
```

`shared/helper.dol`：

```dol
$fn answer() -> Int {
    $# 42;
}
```

## 命名空间访问

导入后，不会把函数直接铺平到全局；通常通过模块命名空间访问：

```dol
$mod helper;
$>> helper.answer();
```

也就是说，`$mod helper;` 导入后，你写的是 `helper.answer()`，而不是直接写 `answer()`。

## 通配符模块导入

当前实现支持：

```dol
$mod math.*;
```

它表示导入某个模块目录下的直接子模块。

例如目录：

```text
math/
├── add.dol
└── mul.dol
```

导入：

```dol
$mod math.*;
```

这时会把 `math/` 目录下的直接子模块注册为命名空间，而不是把函数平铺进当前文件。

## 公有函数与私有函数

```dol
$fn public_api() -> Int {
    $# 1;
}

_$fn hidden() -> Int {
    $# 2;
}
```

`$mod` 只暴露公开函数。

可以把规则记成：

- `$fn` 默认公开
- `_$fn` 默认私有
- 私有函数只给当前模块内部使用

例如：

```dol
_$fn twice(x) -> Int {
    $# x * 2;
}

$fn calc(x) -> Int {
    $# twice(x) + 1;
}
```

## `std.*` 命名空间

标准库模块也通过 `$mod` 导入：

```dol
$mod std.math;
$mod std.str;
```

但 `std.*` 是保留命名空间，它的解析规则和普通项目模块不同。

常见写法：

```dol
$mod std.math;
$mod std.str;

$>> math.sqrt(16.0);
$>> str.trim("  hello  ");
```

## 当前模块解析顺序

普通模块的主顺序可以先记成：

1. 当前文件所在目录
2. 当前项目根目录
3. 项目根目录下的 `modules/`
4. 如果声明了依赖，再查 `deps/`

`std.*` 则只走标准库解析，不和普通项目模块混用。

## 一个完整例子

目录：

```text
demo/
├── main.dol
└── shared/
    └── helper.dol
```

`shared/helper.dol`：

```dol
_$fn twice(x) -> Int {
    $# x * 2;
}

$fn answer() -> Int {
    $# twice(21);
}
```

`main.dol`：

```dol
$mod shared.helper;
$>> helper.answer();
```

## 迁移期参考

- [modules.md](modules.md)
- `docs/spec/modules.md`

## 下一章

继续看 [12-projects-and-package.md](12-projects-and-package.md)。
