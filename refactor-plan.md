# Dolang Refactor Execution Plan

## 1. 计划目标

这份计划不是方向说明，而是面向当前仓库的执行清单。目标只有四个：

1. 把 Dolang 从“可运行原型”整理成“可维护项目”
2. 把核心架构整理到适合开源协作的状态
3. 为未来标准库建设建立稳定边界
4. 在重构过程中尽量不破坏现有语言行为

---

## 2. 当前仓库的关键问题

结合当前代码状态，后续重构必须先解决这些问题：

### 2.1 运行时全局状态过多

当前全局状态分散在：

- `src/interpreter/mod.rs`
- `src/config.rs`

包括：

- HTTP 路由注册表
- 静态资源路由注册表
- 当前执行文件
- serve 配置

这会直接带来三个问题：

- `run / repl / serve / test` 的执行路径无法完全统一
- 单元测试很难隔离
- 后续标准库和 LSP 很难共享同一套上下文

### 2.2 错误处理模型不统一

当前存在两类错误传递方式混用：

- 正常的 `Result<_, Error>`
- 字符串协议式错误，例如 `"[ERROR] ..."`、`"__DIVZERO__"`、`"__MODZERO__"`

这会让：

- 运行时逻辑难维护
- 测试难断言
- 报错信息难统一
- 后续诊断系统和 IDE 支持难落地

### 2.3 启动流程重复

`run_file`、`serve`、`test` 都在做类似的事情：

- 定位文件
- 读取源码
- parse
- 构造 env/fns/type_env/const_env
- 执行

这些逻辑现在分散在多个入口里，后续功能一改就要同时改多处。

### 2.4 代码文件过大

当前最需要拆分的文件：

- `src/parser/stmt.rs`
- `src/parser/expr.rs`
- `src/interpreter/exec.rs`
- `src/interpreter/eval.rs`
- `src/server.rs`
- `src/repl.rs`

这类大文件会让社区贡献者很难定位修改边界。

### 2.5 工程护栏不足

当前状态：

- `cargo check` 可通过
- `cargo test` 可通过，但测试数为 `0`
- `cargo fmt --check` 不通过
- `cargo clippy` 有较多 warning
- 缺少 CI
- 缺少根目录开源协作文件

这意味着现在还不适合直接开放大规模社区协作。

### 2.6 项目系统和标准库边界未定

当前已有：

- `package.toml`
- 模块导入
- builtins
- HTTP 相关内建能力

但还没有正式定义：

- 项目清单规范
- 模块解析优先级
- 标准库命名空间
- 内建能力和标准库的职责边界

---

## 3. 执行原则

整个重构必须遵守下面 6 条原则：

### 3.1 不做一次性大重写

必须按 Phase 分步推进，每个 Phase 都要能单独合并。

### 3.2 先稳单体，再拆 workspace

不要一开始就拆成很多 crate。先把单体仓库里的边界理顺，再做 workspace 级拆分。

### 3.3 每个 Phase 都要有验收标准

没有验收标准的阶段不能算完成。

### 3.4 重构必须伴随测试落地

每拆一层，都要补对应测试。不能只做代码搬迁。

### 3.5 先解决公共路径，再动周边功能

优先整理 `run / serve / test / repl` 的共同执行链，不优先扩语法。

### 3.6 标准库建设必须晚于模块系统稳定

模块规则没定之前，不适合让社区大规模提交标准库。

---

## 4. 推荐执行方式

建议按“小步 PR”推进，每个 Phase 拆成 2 到 5 个 PR。

每个 PR 应遵守：

- 只解决一个明确问题
- 带测试
- 不同时引入多种架构变化
- 可回滚

建议分支命名方式：

- `codex/refactor-phase0-ci`
- `codex/refactor-phase1-context`
- `codex/refactor-phase2-diagnostics`

---

## 5. Phase 0 - 建立工程基线

## 5.1 目标

先把仓库变成“可以稳定重构”的状态。

## 5.2 完成标准

满足以下条件才算完成：

- `cargo fmt --check` 通过
- `cargo clippy` 通过或仅剩明确接受的 warning
- `cargo test` 不再是 0 个测试
- 有 CI
- 有基础开源协作文档

## 5.3 执行步骤

### Step 0.1 整理根目录工程文件

新增：

- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`
- `SECURITY.md`
- `.editorconfig`
- `rustfmt.toml`

说明：

- `CONTRIBUTING.md` 只写当前有效流程，不写未来规划
- `CODE_OF_CONDUCT.md` 直接采用成熟模板即可
- `SECURITY.md` 明确漏洞提交流程

### Step 0.2 增加 CI

新增：

- `.github/workflows/ci.yml`

CI 最小内容：

- Rust stable
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features`
- `cargo test`

### Step 0.3 统一格式与 warning 基线

执行：

- `cargo fmt`
- `cargo clippy --fix` 可自动修复的部分先处理

本步骤只做：

- 格式统一
- module inception 这类明显结构 warning 的整理
- 明显可收敛的命名和导入清理

本步骤不要做：

- 大规模逻辑重写
- 架构搬迁

### Step 0.4 建立最小测试集

新增目录：

- `tests/`
- `tests/fixtures/`

第一批测试必须覆盖：

1. 最小运行脚本
2. 变量声明与赋值
3. 函数定义与调用
4. 基础表达式求值
5. HTTP 路由注册最小样例

### Step 0.5 修正文档与真实结构不一致的问题

需要更新：

- `README`
- `docs/contributing/dev-guide.md`

确保文档里写的命令、目录、开发方式和真实项目一致。

## 5.4 产出物

- 可工作的 CI
- 最小测试集
- 一致的代码风格
- 可读的根目录协作文档

## 5.5 风险控制

- 本阶段不要移动核心逻辑文件
- 本阶段不要引入新的 crate
- 所有修改优先保证现有行为不变

---

## 6. Phase 1 - 提取统一执行上下文

## 6.1 目标

把解释器从“依赖全局状态”改成“显式接收上下文”。

这是整个重构里最关键的一步，也是后续所有 Phase 的前提。

## 6.2 完成标准

满足以下条件才算完成：

- 不再由解释器直接依赖全局 `CURRENT_FILE`
- 不再由解释器直接依赖全局路由注册表
- `run / serve / test / repl` 使用统一的装载接口
- 核心执行路径可通过构造上下文独立测试

## 6.3 新增结构

建议先在当前单 crate 内新增以下模块，而不是马上拆 crate：

```text
src/
├── runtime/
│   ├── mod.rs
│   ├── context.rs
│   ├── session.rs
│   ├── loader.rs
│   └── routes.rs
```

### 其中建议职责如下

- `context.rs`
  - 维护运行模式
  - 当前文件
  - 项目根目录
  - 项目配置
  - 路由注册表
  - 静态资源注册表

- `loader.rs`
  - 统一负责读取文件
  - parse
  - 入口文件定位
  - 模块查找

- `routes.rs`
  - 统一路由数据结构
  - 统一路由注册

## 6.4 执行步骤

### Step 1.1 定义 `RuntimeContext`

新增 `RuntimeContext`：

- `current_file`
- `project_root`
- `mode`
- `project_config`
- `http_routes`
- `static_routes`

注意：

- 这里先不追求设计最终完美
- 先做到“替代全局状态”

### Step 1.2 提取 `ProgramState`

把当前执行时临时构造的内容统一收敛，例如：

- `env`
- `type_env`
- `const_env`
- `fns`

建议定义：

```text
ProgramState {
  env,
  type_env,
  const_env,
  fns,
}
```

### Step 1.3 修改 `exec` / `eval` 签名

逐步把以下函数改成显式接收上下文：

- `exec`
- `exec_with_writer`
- `exec_inner`
- `eval_expr`
- `exec_http_handler`

目标方向：

- 运行时状态放进 `ProgramState`
- 执行环境信息放进 `RuntimeContext`

### Step 1.4 去掉解释器模块里的全局路由状态

当前的：

- `HTTP_ROUTES`
- `STATIC_ROUTES`
- `CURRENT_FILE`

改为：

- `RuntimeContext` 成员

### Step 1.5 去掉配置模块里的全局 serve 配置

当前：

- `SERVE_CONFIG`

改为：

- 项目配置从 `RuntimeContext` 读取

### Step 1.6 提取统一装载流程

新增统一入口函数，例如：

- `load_program_from_file`
- `execute_program`
- `build_program_state`

让以下调用路径统一：

- `run_file`
- `run_serve`
- `run_test`
- REPL 内执行单段代码

### Step 1.7 让 `serve` 和 `test` 共用路由执行逻辑

当前 `server.rs` 和 `test.rs` 都在做：

- 路由参数提取
- body 注入
- headers 注入
- handler 执行
- response 构建

应该提取出公共层，例如：

- `http/request_context.rs`
- `http/handler_runner.rs`

## 6.5 本阶段必须补的测试

- 构造一个独立 `RuntimeContext` 执行脚本
- 验证多次执行之间上下文不串
- 验证 `serve` 和 `test` 共用同一套路由执行逻辑后行为不变

## 6.6 风险控制

- 先保留旧 API 的薄封装，避免一次性改太多调用点
- 每移除一个全局状态，就补一个回归测试

---

## 7. Phase 2 - 统一错误系统与诊断模型

## 7.1 目标

彻底停止字符串协议式报错，把错误统一成结构化类型。

## 7.2 完成标准

满足以下条件才算完成：

- 不再使用 `"[ERROR] ..."` 作为执行错误协议
- 不再使用 `__DIVZERO__`、`__MODZERO__`
- 词法、语法、运行时错误都有统一结构
- CLI 能输出统一格式错误

## 7.3 新增结构

建议新增：

```text
src/
├── diagnostics/
│   ├── mod.rs
│   ├── diagnostic.rs
│   ├── codes.rs
│   └── render.rs
```

## 7.4 执行步骤

### Step 2.1 定义统一诊断类型

最小结构建议：

- `code`
- `message`
- `severity`
- `file`
- `span`
- `line`
- `column`

### Step 2.2 统一 lexer 错误输出

把：

- `Result<Vec<Token>, String>`

改成：

- `Result<Vec<Token>, Diagnostic>`

### Step 2.3 统一 parser 错误输出

把：

- `ParseError`
- `Error::Parse`

重新整理为结构化诊断。

重点处理：

- 表达式解析时 source context 丢失的问题
- `calc_line_col` 的使用一致性

### Step 2.4 改造 `eval_expr`

把：

- `Option<DolangValue>`

改成：

- `Result<DolangValue, RuntimeDiagnostic>`

本步骤是关键，不允许再继续返回字符串错误。

### Step 2.5 改造 `exec_inner`

把 `Flow::Err(Error)` 保留或升级都可以，但要做到：

- 错误只有一条通道
- 错误不是值的一种伪装形式

### Step 2.6 调整 CLI 错误展示

在：

- `run`
- `serve`
- `test`
- `repl`

统一输出格式，例如：

- 文件
- 行列
- 错误码
- 摘要信息

## 7.5 本阶段必须补的测试

- 词法错误定位
- 语法错误定位
- 运行时除零错误
- 未定义变量错误
- 模块导入失败错误

## 7.6 风险控制

- 本阶段不要同时做模块系统重构
- 优先解决错误传递通道，不先追求报错美化

---

## 8. Phase 3 - 拆分超大模块，明确职责边界

## 8.1 目标

把当前“超大文件承载多个职责”的结构拆开，让每类贡献者都能只关注自己的修改面。

## 8.2 完成标准

满足以下条件才算完成：

- `parser/stmt.rs` 被拆成按语句类别组织的模块
- `parser/expr.rs` 被拆成按优先级或能力组织的模块
- `interpreter/exec.rs` 被拆成按 statement family 组织的模块
- `interpreter/eval.rs` 被拆成按 expression family 组织的模块

## 8.3 推荐拆分方式

### parser 部分

```text
src/parser/
├── mod.rs
├── common.rs
├── expr/
│   ├── mod.rs
│   ├── precedence.rs
│   ├── literals.rs
│   ├── postfix.rs
│   └── constructors.rs
└── stmt/
    ├── mod.rs
    ├── control_flow.rs
    ├── declarations.rs
    ├── functions.rs
    ├── http.rs
    └── modules.rs
```

### interpreter 部分

```text
src/interpreter/
├── mod.rs
├── env.rs
├── value.rs
├── exec/
│   ├── mod.rs
│   ├── control_flow.rs
│   ├── variables.rs
│   ├── functions.rs
│   ├── http.rs
│   ├── io.rs
│   └── modules.rs
└── eval/
    ├── mod.rs
    ├── literals.rs
    ├── operators.rs
    ├── calls.rs
    ├── constructors.rs
    └── io.rs
```

## 8.4 执行步骤

### Step 3.1 先拆 parser

原因：

- parser 纯度更高
- 副作用更少
- 拆分风险低于 runtime

### Step 3.2 再拆 exec

按语句家族拆：

- 变量/常量
- 控制流
- 函数
- HTTP
- 模块
- 文件/输入输出

### Step 3.3 最后拆 eval

按表达式家族拆：

- 字面量
- 运算符
- 函数调用
- 方法调用
- 读写表达式
- HTML/JSON/RES 构造器

### Step 3.4 清理 module inception

例如：

- `src/ast/mod.rs -> src/ast.rs` 或更清晰命名
- `src/token/mod.rs -> src/token.rs`
- `src/lexer/mod.rs -> src/lexer.rs`

目标是让模块命名与目录命名直观统一。

## 8.5 本阶段必须补的测试

- parser 拆分后回归测试
- exec/eval 拆分后行为一致性测试
- builtins 方法分发回归测试

## 8.6 风险控制

- 每次只拆一个大文件
- 每拆一个文件先保留旧接口 re-export，最后再统一清理

---

## 9. Phase 4 - 标准化项目系统与模块解析规则

## 9.1 目标

正式定义 Dolang 项目和模块系统，为标准库和第三方包留出稳定边界。

## 9.2 完成标准

满足以下条件才算完成：

- `package.toml` 有正式 schema
- 模块解析顺序明确定义
- 保留标准库命名空间
- 项目入口、相对导入、标准库导入有文档和测试

## 9.3 执行步骤

### Step 4.1 替换手写配置解析

当前 `package.toml` 解析过于简化，建议引入正式解析方案：

- `serde`
- `toml`

### Step 4.2 定义 manifest schema

先只定义当前真正需要的字段：

- `name`
- `version`
- `entry`
- `[server]`
- `[env]`
- `[dependencies]`

不要一开始设计过多未来字段。

### Step 4.3 定义模块解析顺序

必须明确并写入文档：

1. 当前文件相对路径
2. 项目根目录模块
3. `modules/` 目录
4. `stdlib/`
5. 第三方依赖目录

### Step 4.4 引入 `ModuleResolver`

建议新增：

```text
src/module/
├── mod.rs
├── resolver.rs
├── manifest.rs
└── paths.rs
```

由它统一负责：

- 文件查找
- 模块定位
- manifest 读取
- 项目根判断

### Step 4.5 预留标准库命名空间

建议统一保留：

- `std.*`

例如：

- `std.io`
- `std.http`
- `std.json`
- `std.str`

### Step 4.6 重新划分 builtin 与 stdlib 的边界

建议规则：

- 语言执行不可缺的能力保留为 builtin
- 可作为库形式提供的能力下沉到 stdlib

建议保留 builtin 的内容：

- 基础类型
- 基础运算
- 核心控制结构
- 最小 IO 原语

建议逐步迁移到 stdlib 的内容：

- JSON 辅助方法
- 字符串扩展能力
- HTTP 辅助封装
- 文件工具层

## 9.4 本阶段必须补的测试

- manifest 解析测试
- 模块解析优先级测试
- 相对导入测试
- `std.*` 解析测试

## 9.5 风险控制

- 标准库命名空间先保留，不要一次性塞满内容
- 本阶段先把规则定下来，再逐步迁移实现

---

## 10. Phase 5 - 建立完整测试矩阵

## 10.1 目标

让 Dolang 的语言行为进入“可回归、可验证、可协作”的状态。

## 10.2 完成标准

满足以下条件才算完成：

- 核心子系统都有测试
- 有 fixture 目录
- 有 `.dol` 级别集成测试
- 每个 bug 修复都能沉淀为回归测试

## 10.3 测试层次设计

### 单元测试

覆盖：

- lexer
- parser
- diagnostics
- value
- builtins

### 集成测试

覆盖：

- 运行 `.dol` 文件
- `serve` 模式路由行为
- `test` 模式行为
- 模块导入
- 项目配置读取

### 规范样例测试

建立：

- `tests/spec/valid/`
- `tests/spec/invalid/`

每个样例只验证一个语言行为。

### 回归测试

每修一个 bug，都新增一个对应 fixture。

## 10.4 执行步骤

### Step 5.1 建立 fixture 目录规范

建议：

```text
tests/
├── fixtures/
│   ├── lexer/
│   ├── parser/
│   ├── runtime/
│   ├── http/
│   └── modules/
├── integration/
└── spec/
```

### Step 5.2 建立脚本级测试工具

建议实现统一辅助函数，例如：

- 读取 `.dol`
- 执行
- 捕获 stdout/stderr
- 比较返回值或错误

### Step 5.3 建立最小语义兼容集

至少覆盖：

- 变量/常量
- 类型注解
- 函数
- 控制流
- 列表/映射
- builtins
- HTTP
- 模块导入

### Step 5.4 建立 bug 回归策略

规则：

- 修 bug 必须带测试
- 改语言行为必须更新 spec 用例

## 10.5 风险控制

- 不追求一开始测试覆盖率数字
- 先覆盖关键路径，再逐步补全

---

## 11. Phase 6 - 拆分 workspace crate

## 11.1 前置条件

只有在下面条件满足后，才进入本阶段：

- Phase 0 完成
- Phase 1 完成
- Phase 2 完成
- Phase 3 完成
- Phase 4 完成
- Phase 5 有基本测试矩阵

如果前置条件没完成，不建议提前拆 crate。

## 11.2 目标

把现在的单 crate 结构拆成职责更稳定的 workspace。

## 11.3 完成标准

满足以下条件才算完成：

- frontend 独立可编译
- runtime 独立可编译
- cli 只依赖 frontend + runtime
- lsp 可直接依赖 frontend

## 11.4 推荐拆分结果

```text
crates/
├── dolang-frontend
├── dolang-runtime
├── dolang-cli
└── dolang-lsp
```

### 职责

- `dolang-frontend`
  - token
  - lexer
  - parser
  - ast
  - diagnostics

- `dolang-runtime`
  - value
  - env
  - exec
  - eval
  - builtins
  - module resolver bridge

- `dolang-cli`
  - 命令行参数
  - run
  - repl
  - serve
  - test

- `dolang-lsp`
  - 基于 frontend 的诊断和语义能力

## 11.5 执行步骤

### Step 6.1 先抽 frontend

因为 frontend 副作用最少，拆分成本最低。

### Step 6.2 再抽 runtime

把对 frontend 的依赖限定为 AST 和 diagnostics。

### Step 6.3 最后整理 cli

让二进制入口只负责：

- 参数解析
- 调用 runtime
- 呈现输出

### Step 6.4 更新 workspace 文档和 CI

让 CI 覆盖：

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets`

## 11.6 风险控制

- 每拆一个 crate 先保持 API 简单，不要立刻做大规模抽象
- 优先确保 workspace 拆分不改变语言行为

---

## 12. Phase 7 - 开源协作与标准库建设准备

## 12.1 目标

让社区成员能低成本参与语言核心与标准库建设。

## 12.2 完成标准

满足以下条件才算完成：

- 有标准库贡献指南
- 有语言行为规范文档
- 有示例项目
- 有 issue / PR 模板
- LSP 不再是空壳

## 12.3 执行步骤

### Step 7.1 建立标准库目录

建议：

```text
stdlib/
├── core/
├── io/
├── str/
├── json/
└── http/
```

### Step 7.2 写标准库贡献规则

必须明确：

- 命名规则
- 模块组织方式
- 文档要求
- 测试要求
- 向后兼容要求

### Step 7.3 增加 examples

至少提供：

- hello world
- basic repl script
- http hello api
- module import
- project config
- stdlib usage

### Step 7.4 推进 `dolang-lsp`

最小可交付目标：

- 打开文件能 parse
- 能返回 syntax diagnostics

本阶段不要求一步到位实现完整语言服务。

### Step 7.5 建立 RFC 流程

适合后续社区讨论：

- 新语法
- 标准库 API
- 模块系统规则
- 兼容性策略

---

## 13. 推荐 PR 拆分顺序

这是最适合当前仓库的推荐推进顺序：

### PR 1

- 加 `CONTRIBUTING.md`
- 加 `.editorconfig`
- 加 `rustfmt.toml`
- 加 CI

### PR 2

- 全仓 `cargo fmt`
- 清理基础 clippy warning

### PR 3

- 新增 `tests/`
- 建立最小测试集

### PR 4

- 引入 `RuntimeContext`
- 先迁移 `CURRENT_FILE`

### PR 5

- 迁移 HTTP/STATIC 路由状态

### PR 6

- 迁移 `SERVE_CONFIG`
- 统一装载流程

### PR 7

- 提取 `serve/test` 公共 handler 执行逻辑

### PR 8

- 引入结构化 diagnostics
- 改 lexer/parser 报错

### PR 9

- 改 `eval_expr` 与运行时错误通道

### PR 10

- 拆 parser 大文件

### PR 11

- 拆 interpreter 大文件

### PR 12

- 引入正式 manifest schema
- 加 `ModuleResolver`

### PR 13

- 建立 `stdlib/`
- 预留 `std.*`

### PR 14

- workspace crate 拆分

### PR 15

- 推进 `dolang-lsp` 最小可用版本

---

## 14. 明确不做的事情

为了保证可执行性，这份计划明确不把下面这些事情放进当前重构主线：

- 重写语言语法风格
- 引入复杂 JIT 或字节码系统
- 一开始就做完整包管理器
- 在架构没稳定前开放大规模标准库贡献
- 在测试体系不完整前频繁扩展语言表面积

---

## 15. 整个计划的最终验收标准

整个重构完成后，Dolang 至少应该达到下面状态：

- 没有关键运行时全局状态耦合
- 错误系统统一且结构化
- `run / repl / serve / test` 共享装载与执行底层
- parser/runtime 模块边界清晰
- 模块系统和项目系统有正式规则
- 有基础标准库目录
- 有完整测试矩阵
- 有 CI 和开源协作文档
- 有正式语言规范文档
- 有版本与兼容策略
- 有标准库治理规则
- 有安全模型文档
- 有最小 benchmark 基线
- 新贡献者可以根据文档独立完成一个小型功能 PR

---

## 16. Phase 8 - 语言规范文档正式化

## 16.1 目标

把 Dolang 从“实现优先”提升为“规范与实现并行”。

## 16.2 完成标准

满足以下条件才算完成：

- Dolang 有独立 `docs/spec/` 目录
- parser/runtime 的关键行为都能在 spec 里找到定义
- `tests/spec` 与规范文档建立一一对应关系
- 新贡献者不需要先读实现也能理解语言行为

## 16.3 必须新增的文档

- `docs/spec/README.md`
- `docs/spec/lexical.md`
- `docs/spec/syntax.md`
- `docs/spec/semantics.md`
- `docs/spec/modules.md`
- `docs/spec/runtime-errors.md`

## 16.4 本阶段必须定义的内容

- token 规则
- 注释规则
- 字符串与 f-string 规则
- 表达式优先级和结合性
- 作用域规则
- truthy/falsy 规则
- 类型注解的运行时语义
- `return / break / continue / exit` 行为
- 模块解析顺序
- 标准错误类别与错误码

## 16.5 执行步骤

### Step 8.1 建立 `docs/spec/` 主索引

在 `docs/spec/README.md` 中明确：

- 语言规范的适用范围
- 规范与实现的关系
- 文档目录结构
- 规范变更的维护规则

### Step 8.2 先记录当前实现行为

本阶段先从当前实现提取真实行为，不先重新设计语言规则。

要求：

- 词法规则以当前 lexer 为准
- 语法规则以当前 parser 为准
- 运行时语义以当前 exec/eval 行为为准

### Step 8.3 分主题写规范

每个 spec 文档都必须包含：

- 规则描述
- 最小正确示例
- 最小错误示例
- 对应测试位置

### Step 8.4 建立 spec 与测试映射

在 `tests/spec` 中按文档主题组织用例，要求：

- 每类语义至少有一个 valid case
- 每类错误至少有一个 invalid case
- 文档中示例可以在测试中找到对应样例

### Step 8.5 建立变更联动规则

后续任何语言行为改动，必须同时更新：

- `docs/spec/*`
- `tests/spec/*`
- 如涉及用户可见变化，再更新 `CHANGELOG`

## 16.6 风险控制

- 本阶段不重写语言设计
- 允许记录“当前实现如此，但未来可能调整”的注记
- 规范文档先保证准确，再追求美观和完整度

---

## 17. Phase 9 - 兼容性、版本与弃用策略

## 17.1 目标

给语言核心和标准库建立稳定演进规则，避免后续开源后频繁破坏兼容。

## 17.2 完成标准

满足以下条件才算完成：

- contributor 能判断一个改动是否破坏兼容
- `CHANGELOG` 和版本规则一致
- 语言核心与标准库不再随意改变对外行为
- 未来 breaking change 有统一处理入口

## 17.3 必须新增的文档

- `docs/spec/versioning.md`
- `docs/spec/compatibility.md`
- `docs/spec/deprecation.md`

## 17.4 本阶段必须定义的规则

- 项目版本策略采用 SemVer 风格
- 哪些改动算 breaking change
- 哪些改动允许作为 minor 增量
- 标准库 API 的弃用周期
- 语言语法弃用的公告方式
- 旧行为保留窗口和移除窗口

## 17.5 执行步骤

### Step 9.1 定义兼容边界

先明确两套边界：

- 语言核心兼容边界
- 标准库 API 兼容边界

### Step 9.2 定义对外承诺

至少明确：

- parser 行为
- runtime 语义
- CLI 命令
- `std.*` 命名空间

### Step 9.3 标准化变更记录

把 `docs/CHANGELOG.md` 调整为统一模板，固定栏目：

- Added
- Changed
- Deprecated
- Removed
- Fixed

### Step 9.4 建立弃用流程

任何弃用至少要写清：

- 当前行为
- 替代行为
- 起始版本
- 计划移除版本

### Step 9.5 建立 breaking change 门槛

未来 breaking change 必须附带：

- RFC
  或
- 兼容说明文档

## 17.6 风险控制

- 本阶段先建立规则，不强行回溯重写旧历史
- 对尚未稳定的能力可以先标记为 experimental，降低兼容承诺范围

---

## 18. Phase 10 - 标准库治理与稳定级别

## 18.1 目标

防止未来 `stdlib/` 变成无边界堆叠，建立标准库准入和稳定性规则。

## 18.2 完成标准

满足以下条件才算完成：

- 社区能明确知道一个能力该做成 builtin 还是 stdlib
- 标准库模块不会无约束扩张
- 标准库评审有统一口径
- 每个 stdlib 模块都有最小交付要求

## 18.3 必须新增的文档

- `docs/stdlib/README.md`
- `docs/stdlib/contributing.md`
- `docs/stdlib/stability-levels.md`

## 18.4 本阶段必须定义的规则

- builtin 与 stdlib 的准入边界
- stdlib 模块命名规范
- 每个 stdlib 模块必须具备文档、示例、测试、稳定级别标记
- 稳定级别分为 `experimental`、`preview`、`stable`

## 18.5 执行步骤

### Step 10.1 建立 `stdlib/` 目录模板

在 `stdlib/` 根目录预留标准模块模板，至少包含：

- 源码目录
- README
- 示例
- 测试

### Step 10.2 定义模块最低准入要求

一个 stdlib 模块最少必须包含：

- 代码
- README
- 示例
- 集成测试

### Step 10.3 划分 builtin 与 stdlib 边界

保留 builtin：

- 基础类型
- 基础控制结构
- 最小 IO 原语

优先放 stdlib：

- JSON 辅助
- 文件工具
- HTTP 辅助封装
- 字符串扩展工具

### Step 10.4 建立稳定级别升级规则

默认规则：

- 新 stdlib 模块默认 `experimental`
- 文档、测试、兼容策略齐全后可升 `preview`
- 经过稳定使用和兼容承诺后可升 `stable`

### Step 10.5 建立标准库评审清单

评审至少检查：

- 是否应属于 builtin
- 是否已有重复能力
- 是否有文档和示例
- 是否有测试
- 是否声明稳定级别

## 18.6 风险控制

- 本阶段先建治理规则，不要求一次性填满 stdlib
- 对争议模块先保持 `experimental`

---

## 19. Phase 11 - 安全模型与副作用边界

## 19.1 目标

给 Dolang 的文件、环境变量、HTTP 能力建立最小安全模型，让它更像正式语言项目，而不是仅有功能集合。

## 19.2 完成标准

满足以下条件才算完成：

- Dolang 的副作用边界有正式文字定义
- 社区在做 stdlib 和 runtime 扩展时不会绕过安全边界讨论
- 后续做受限执行环境时有文档落点

## 19.3 必须新增的文档

- `docs/spec/security-model.md`

## 19.4 本阶段必须定义的能力边界

- 文件读取/写入行为
- 环境变量访问行为
- HTTP handler 默认可访问能力
- 模块加载的文件系统边界
- 未来是否支持 sandbox / capability-based runtime

## 19.5 执行步骤

### Step 11.1 记录当前副作用入口

先梳理当前真实存在的副作用入口：

- 文件 I/O
- ENV 读取
- 网络服务暴露
- 模块文件加载

### Step 11.2 定义当前默认能力模型

当前默认模型明确写入文档：

- 默认信任本地执行环境
- 没有沙箱
- 没有权限系统

### Step 11.3 标出未来可受限的能力

在文档中显式区分：

- 当前默认开放能力
- 未来可能改成受限能力的入口

### Step 11.4 预留未来策略接口

在重构计划里为后续能力控制预留扩展方向：

- 可选 runtime policy
- 可选文件系统限制
- 可选环境变量白名单

### Step 11.5 建立安全讨论入口

后续涉及副作用能力新增或放宽时，必须同步更新：

- `docs/spec/security-model.md`
- 相关 spec
- 相关测试

## 19.6 风险控制

- 本阶段只文档化当前行为，不立即引入沙箱实现
- 不用在当前阶段承诺完整安全隔离能力

---

## 20. Phase 12 - 基准测试与性能回归基线

## 20.1 目标

在不把性能优化放到主线的前提下，建立最小 benchmark 基线，防止重构后性能长期无感退化。

## 20.2 完成标准

满足以下条件才算完成：

- 至少有一组可重复运行的 benchmark
- 重构前后可做横向比较
- 项目不再只有功能回归，没有性能回归观察

## 20.3 建议新增内容

- `benches/`
- `docs/spec/performance-baseline.md`

## 20.4 第一批 benchmark 覆盖范围

- lexer 处理小型脚本
- parser 处理小型脚本
- runtime 执行基础表达式
- 函数调用
- 模块加载
- HTTP 路由 handler 执行最小样例

## 20.5 执行步骤

### Step 12.1 选定 benchmark 框架

要求：

- 不引入复杂依赖链
- 能稳定重复运行
- 能输出最小基线数据

### Step 12.2 固定输入样例

为 benchmark 建立固定样例集，避免目标漂移。

样例必须记录：

- 输入规模
- 使用场景
- 预期关注点

### Step 12.3 建立基线记录方式

每次记录至少包含：

- 输入规模
- 执行时间
- 测试环境说明
- 对应版本号

### Step 12.4 建立运行规则

benchmark 不作为主线阻塞项，但要求：

- 大重构前后必须运行一次
- parser/runtime 重构 PR 要附带“无明显性能退化”说明

### Step 12.5 文档化性能目标

在 `docs/spec/performance-baseline.md` 中明确：

- benchmark 的用途是观察退化，不是当前阶段强行做性能优化
- 允许小幅波动，但不能长期无记录退化

## 20.6 风险控制

- 不把 benchmark 结果直接作为合并门槛
- 先建立基线，再逐步收紧要求

---

## 21. 结论

这个重构不应该被理解为“整理代码风格”，而应该被理解为：

把 Dolang 的核心从“作者自己能持续推进”升级到“团队和社区也能持续推进”。

正确顺序必须是：

1. 先建立工程基线
2. 再提取统一上下文
3. 再统一错误系统
4. 再拆模块
5. 再定义项目和模块规则
6. 再补完整测试
7. 再拆 workspace
8. 再正式化语言规范文档
9. 再补版本、兼容和弃用策略
10. 再建立标准库治理规则
11. 再定义安全模型与副作用边界
12. 最后补 benchmark 基线、推进标准库和 LSP

按这个顺序走，风险最低，返工最少，也最符合 Dolang 当前阶段的现实需要。
