# 18. 模式与实战配方

这一章收集“怎么组合使用 Dolang”的实际路径。

## 配方 1：单文件脚本

适合：

- 快速数据处理
- 小工具
- 一次性脚本

一个最小例子：

```dol
$ nums = [1, 2, 3, 4, 5];
$ total = 0;

$for n in nums {
    total += n;
}

$>> total;
```

## 配方 2：标准库驱动的小工具

典型组合：

- `std.str`
- `std.json`
- `std.path`
- `std.fs`

例如：

```dol
$mod std.fs;
$mod std.json;

$ raw = fs.read_text("data.json");
$ obj = json.parse(raw);
$>> json.stringify(obj);
```

## 配方 3：带 package 的小项目

适合：

- 有配置项
- 有多个模块
- 需要服务模式

最小结构：

```text
my-app/
├── package.toml
├── main.dol
└── shared/
    └── helper.dol
```

## 配方 4：拆模块的 HTTP 服务

推荐组合：

- `package.toml`
- `$mod` 普通模块
- `$HTTP(...).link(...)`

入口文件可以像这样：

```dol
$HTTP("/api").link("routers.api");
$STATIC("/static", "static");
```

## 配方 5：从脚本迁移到服务

常见演进顺序：

1. 从单文件脚本开始
2. 抽出模块
3. 引入 `package.toml`
4. 暴露 HTTP 路由
5. 用 `dolang test` 验证路由

## 一套实用的成长路径

如果你是第一次系统写 Dolang，建议按这个顺序练习：

1. 先写只包含变量、循环、函数的脚本
2. 再引入 `std.str`、`std.math`、`std.json`
3. 再把脚本拆成两个模块
4. 再加 `package.toml`
5. 最后再进入 HTTP 服务

## 这一章后续会继续扩写什么

优先级最高的补充方向：

- 文件处理脚本实战
- JSON API 小项目
- 多模块目录组织模板
- 服务入口与路由拆分模板
