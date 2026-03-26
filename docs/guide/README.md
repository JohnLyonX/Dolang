# Dolang Guide

本页既是 `docs/guide/` 的入口，也是这一轮文档重构的章节目录。

当前策略：

- 旧的 guide 页面先保留，不删不减
- 新增按编号组织的章节
- 后续逐步把旧内容迁移、扩写、重组到新章节

这一版目录只基于当前仓库已经实现并可从 spec、实现、fixture、example 中交叉确认的能力，不把尚未稳定或尚未真正落地的设计放进主线。

## 编写原则

- 先讲“怎么用”，再补“为什么这样设计”
- 每章尽量只覆盖一个主题，正文配最小可运行示例
- Guide 讲学习路径，Reference 讲查表，Spec 讲当前实现边界
- 以当前实现为准，不提前写未来版本语法
- Web、项目系统、标准库是 Dolang 的用户主线，不应只放在零散参考文档里

## 章节目录

```text
docs/guide/
├── README.md
├── 01-introduction.md
├── 02-installation-and-cli.md
├── 03-first-program.md
├── 04-basic-syntax.md
├── 05-values-and-variables.md
├── 06-control-flow.md
├── 07-functions.md
├── 08-collections-and-methods.md
├── 09-gradual-typing.md
├── 10-error-handling.md
├── 11-modules.md
├── 12-projects-and-package.md
├── 13-stdlib-overview.md
├── 14-io-env-config.md
├── 15-http-basics.md
├── 16-http-organization.md
├── 17-testing-and-debugging.md
├── 18-patterns-and-recipes.md
└── appendix/
    ├── syntax-cheatsheet.md
    ├── cli-cheatsheet.md
    ├── module-resolution.md
    └── common-errors.md
```

## 开始阅读

- [01-introduction.md](01-introduction.md)
- [02-installation-and-cli.md](02-installation-and-cli.md)
- [03-first-program.md](03-first-program.md)
- [04-basic-syntax.md](04-basic-syntax.md)
- [05-values-and-variables.md](05-values-and-variables.md)
- [06-control-flow.md](06-control-flow.md)
- [07-functions.md](07-functions.md)
- [08-collections-and-methods.md](08-collections-and-methods.md)
- [09-gradual-typing.md](09-gradual-typing.md)
- [10-error-handling.md](10-error-handling.md)
- [11-modules.md](11-modules.md)
- [12-projects-and-package.md](12-projects-and-package.md)
- [13-stdlib-overview.md](13-stdlib-overview.md)
- [14-io-env-config.md](14-io-env-config.md)
- [15-http-basics.md](15-http-basics.md)
- [16-http-organization.md](16-http-organization.md)
- [17-testing-and-debugging.md](17-testing-and-debugging.md)
- [18-patterns-and-recipes.md](18-patterns-and-recipes.md)

附录：

- [syntax-cheatsheet.md](appendix/syntax-cheatsheet.md)
- [cli-cheatsheet.md](appendix/cli-cheatsheet.md)
- [module-resolution.md](appendix/module-resolution.md)
- [common-errors.md](appendix/common-errors.md)

迁移期旧页面：

- [concepts.md](concepts.md)
- [getting-started.md](getting-started.md)
- [examples.md](examples.md)
- [modules.md](modules.md)
- [stdlib.md](stdlib.md)
- [types.md](types.md)
- [web.md](web.md)

## 主线章节

### Part 1 起步

#### 1. 认识 Dolang

- Dolang 的定位、适用场景、和其他语言的区别
- 解释执行、面向脚本和轻服务的心智模型
- Guide / Reference / Spec 三类文档分别解决什么问题

#### 2. 安装与 CLI

- 本地编译与运行方式
- `dolang` REPL、`dolang run <file>`、`dolang serve [path]`、`dolang test`
- `--routertab`、`test --route`、`test --body` 的使用场景

#### 3. 第一个程序

- Hello World
- 第一个 REPL 交互
- 第一个 HTTP 路由
- 单文件脚本与项目目录运行的区别

### Part 2 语言基础

#### 4. 基本语法

- 语句与分号
- 注释
- 标识符与 `$` 前缀关键字
- 字面量：数字、字符串、字符、布尔、`null`
- f-string 基本写法
- 表达式优先级的最小规则

#### 5. 值、变量与常量

- `$` 变量声明
- `$@` 常量声明
- 赋值与复合赋值
- `$>>` 输出、`$>>ERR(...)` 错误输出
- `$<<LINE(...)` 读入

#### 6. 控制流

- `$if / $elif / $else`
- `$while`
- `$loop`
- `$for item in iterable`
- C 风格 `$for init; condition; update`
- `$break`、`$continue`、`exit`
- truthy / falsy 规则

#### 7. 函数

- `$fn` 定义函数
- 参数、可变参数、返回值
- `$#` 返回
- 返回类型注解
- 匿名函数的当前能力边界
- `_$fn` 私有函数

#### 8. 集合与常用方法

- `List`、`Map`、字符串的基础操作
- 索引访问与方法调用
- 常用内建方法：`len`、`slice`、`contains`、`push`、`pop`、`keys`、`values`、`type`
- 什么时候用值方法，什么时候用 `std.*`

### Part 3 语言能力

#### 9. 渐进类型

- “默认动态，显式约束”的使用方式
- 变量/常量类型注解
- 当前真正支持的声明类型名
- 函数返回类型检查
- `JSON<User>` 与 `$Type` 的定位

#### 10. 错误处理

- 运行时错误是什么
- `$try / $catch`
- `$throw`
- 未定义变量、类型不匹配、模块加载失败等典型错误
- Guide 中如何引导读者读懂错误信息

#### 11. 模块系统

- `$mod a.b;`
- `$mod a.*;`
- 命名空间访问规则
- 公有函数与私有函数
- 普通模块与 `std.*` 的区别

#### 12. 项目系统

- `package.toml` 的当前字段
- `entry`
- `[server]`
- `[env]`
- `[dependencies]` 当前只是解析边界，不写成“已完整可用的包管理”
- 项目根与入口解析规则

### Part 4 标准库与宿主能力

#### 13. 标准库总览

- native 模块与纯 `.dol` 模块的区别
- 当前已可写入主线的模块：
  - `std.str`
  - `std.math`
  - `std.json`
  - `std.fs`
  - `std.env`
  - `std.time`
  - `std.uuid`
  - `std.http`
  - `std.core.iter`
  - `std.core.check`
  - `std.str.check`
  - `std.str.fmt`
  - `std.math.stats`
  - `std.math.trig`
  - `std.path`

#### 14. I/O、环境变量与配置

- `$<<ENV(...)`
- `$<<CONFIG(...)`
- `$<<FILE(...)` 与 `$>>FILE(...)`
- `std.fs` 与 `std.env`
- serve 模式与非 serve 模式下的能力差异

### Part 5 Web 开发

#### 15. HTTP 基础

- `$GET / $POST / $PUT / $DEL / $PATCH`
- handler 参数、路径参数、query 参数、body
- `$HDR(...)`
- `-> String`、`-> JSON`、`-> HTML`
- `$JSON { ... }`、`$HTML(...)`、`$RES(status, body)`
- `std.http` 的同步客户端调用

#### 16. HTTP 组织方式

- `$HTTP { ... }`
- `$HTTP("/prefix") { ... }`
- `$HTTP(...).link("module.path")`
- `$STATIC(...)`
- 路由拆分、静态资源、HTML link 的当前能力边界

### Part 6 工程化使用

#### 17. 测试与调试

- `dolang test`
- 针对单路由测试
- 如何最小化复现运行时错误
- REPL、打印输出、fixture 风格的调试方式

#### 18. 模式与实战配方

- 小脚本
- 带 `package.toml` 的项目
- 拆模块的服务
- 纯标准库数据处理
- “从单文件到小型 API” 的迁移路径

## 附录章节

### Appendix A 语法速查

- 一页式列出常见语法
- 面向已经学过一遍、需要回查的用户

### Appendix B CLI 速查

- run / serve / test / repl 的命令清单

### Appendix C 模块解析说明

- 当前目录、项目根、`modules/`、`stdlib/`、`deps/` 的查找顺序

### Appendix D 常见错误

- 解析错误
- 类型错误
- 模块错误
- 文件与环境变量读取错误

## 暂不进入 Guide 主线的主题

以下主题可以在设计文档或 reference 中保留，但不建议先写进 Guide 主线：

- 第三方依赖生态与真正的包管理工作流
- 尚未形成稳定用户故事的高级安全模型
- LSP 与编辑器集成细节
- 未经 spec/test 固化的新语法提案
- 把 `Any`、`Json`、`Response` 等写成“变量声明已完整支持”的教程

## 现有文档的重构建议

- `getting-started.md` 可拆入第 2、3 章
- `concepts.md` 可压缩为第 1 章的“定位与设计目标”
- `examples.md` 不再单独做成散装示例页，改为分散到各章与第 18 章
- `types.md` 需要继续和主线章节保持同步，避免旧页再次落后于实现
- `modules.md`、`stdlib.md`、`web.md` 适合分别并入第 11、13-16 章

## 下一步建议

按下面顺序开始扩写最稳妥：

1. 第 2 章 安装与 CLI
2. 第 3 章 第一个程序
3. 第 4-8 章 语言基础
4. 第 11-16 章 模块、项目系统、标准库、Web
5. 第 9-10、17-18 章 类型、错误处理、测试与实战
