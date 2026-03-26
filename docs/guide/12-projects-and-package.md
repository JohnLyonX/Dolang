# 12. 项目系统与 package.toml

当脚本开始变大，通常就该进入项目目录组织。

## 一个最小项目

```text
my-app/
├── package.toml
├── main.dol
└── shared/
    └── helper.dol
```

## 最小 `package.toml`

```toml
name = "my-app"
version = "0.1.0"
entry = "main.dol"

[server]
host = "0.0.0.0"
port = 8080

[env]
APP_NAME = "My App"
```

这是一个“当前可用字段都比较明确”的最小项目配置。

## 当前重要字段

- `name`
- `version`
- `entry`
- `[server]`
- `[env]`
- `[dependencies]`

其中 `[dependencies]` 当前更适合理解为“模块解析边界的一部分”，不要直接写成成熟包管理工作流。

## 一个最小项目示例

`package.toml`：

```toml
name = "project-config"
version = "0.1.0"
entry = "main.dol"

[env]
APP_NAME = "Dolang"
```

`main.dol`：

```dol
$ app = $<<CONFIG("APP_NAME");
$>> app;
```

## 运行项目

```bash
dolang run main.dol
dolang serve .
```

当前 CLI 中：

- `run` 运行的是明确的 `.dol` 文件
- `serve` 可以直接接项目目录

当 `serve` 接收目录时，会先定位项目根，再读取 `package.toml` 的 `entry`。

如果没有 manifest，则会退回到默认入口逻辑。

## 配置读取

`[env]` 中的值可以通过 `$<<CONFIG(...)` 读取：

```dol
$ app = $<<CONFIG("APP_NAME");
$>> app;
```

这适合读取：

- 应用名
- 环境标记
- 默认端口相关配置

## `server` 配置

服务模式会用到 `[server]`：

```toml
[server]
host = "0.0.0.0"
port = 8080
```

当你执行：

```bash
dolang serve .
```

CLI 会加载项目配置，并据此启动服务。

## 项目和单文件的区别

单文件适合：

- demo
- 小工具
- 单路由原型

项目目录适合：

- 多模块
- 配置读取
- 服务模式
- 后续继续扩展

## 一个推荐的起步目录

```text
my-app/
├── package.toml
├── main.dol
└── shared/
    └── helper.dol
```

## 进一步参考

- `docs/reference/project-system.md`

## 下一章

继续看 [13-stdlib-overview.md](13-stdlib-overview.md)。
