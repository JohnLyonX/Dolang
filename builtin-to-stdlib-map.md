# Dolang Builtin To Stdlib Map

本文件记录 Dolang 当前能力应落在哪一层：

- `core`
- `runtime intrinsic`
- `future stdlib`

目标不是立刻实现完整标准库，而是明确哪些能力已经具备宿主基础，哪些能力后续应通过 `std.*` 暴露。

## Core

这些能力属于语言底座，继续保留在解释器核心：

- 词法、语法、AST、诊断
- 变量、常量、作用域、控制流
- 函数声明与调用
- `$mod`
- HTTP 路由语法：`$GET/$POST/...`、`$HTTP`
- 基础值类型：`Int/Float/String/Bool/List/Map/Json/Html/Response`
- `File` 作为 runtime handle

## Runtime Intrinsic

这些能力触达宿主 OS / 进程环境，应统一收敛到 intrinsic 层：

| 当前能力 | intrinsic | 说明 |
| --- | --- | --- |
| `$<<FILE(...)` / `File.read()` | `FsReadText` / `FsReadLines` | 文件读取原语 |
| `$>>FILE(...)` / `File.content(...)` | `FsWriteText` / `FsAppendText` | 文件写入与追加 |
| `DEL` 删除文件 | `FsDelete` | 文件删除 |
| `File.exists()` | `FsExists` | 文件存在性 |
| `File.size()` | `FsSize` | 文件大小 |
| `File.is_dir()` | `FsIsDir` | 文件夹判断 |
| `$<<ENV("KEY")` | `EnvGet` | 环境变量读取 |
| `$<<CONFIG("KEY")` | `ConfigGet` | serve 模式项目配置读取 |

## Future Stdlib

这些能力未来应优先通过 `std.*` 暴露，而不是直接扩散 builtin 表面：

| future stdlib API | intrinsic 依赖 |
| --- | --- |
| `std.fs.read_text(path)` | `FsReadText` |
| `std.fs.read_lines(path)` | `FsReadLines` |
| `std.fs.write_text(path, content)` | `FsWriteText` |
| `std.fs.append_text(path, content)` | `FsAppendText` |
| `std.fs.exists(path)` | `FsExists` |
| `std.fs.size(path)` | `FsSize` |
| `std.fs.is_dir(path)` | `FsIsDir` |
| `std.fs.remove(path)` | `FsDelete` |
| `std.env.get(key)` | `EnvGet` |
| `std.config.get(key)` | `ConfigGet` |

## Deferred

这些能力还没有在本阶段进入完整 intrinsic / stdlib 设计：

- `Json` 的 parse / serialize API
- HTTP request / response helper API
- HTML 资源链接能力
- 时间、随机数、路径工具

原因：

- 当前值模型或行为边界还不够稳定
- 适合在 fs/env/config intrinsic 层稳定后再推进
