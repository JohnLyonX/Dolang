# `$Type` 自定义类型系统总计划与当前状态

更新日期：2026-04-05

## 目标

把 Dolang 的 `$Type` 从“说明性结构”收紧成一套真正有 runtime 约束的名义自定义类型系统，并逐步统一：

- 构造期约束
- 字段赋值约束
- 变量注解约束
- `List<T>` 元素约束
- 函数 / HTTP 返回类型约束
- 类型表达模型统一到 `TypeExpr`

## 总体阶段

### 阶段 1：最小闭环

目标：

- `$Type` 构造时严格校验
- typed instance 字段赋值严格校验
- 普通函数和 HTTP handler 返回值强校验
- `run` / `serve` / `test` 语义一致

状态：已完成

已完成内容：

- `TypeName { ... }` 会检查未知字段、缺少必填字段、字段值类型错误
- typed instance 不允许通过赋值动态长出新字段
- 普通函数 `-> T` / `-> List<T>` 会强校验
- HTTP handler `-> T` / `-> List<T>` 会强校验
- 裸 `Map/JSON` 不能冒充 `$Type`

主要设计与计划文件：

- `docs/superpowers/specs/2026-04-04-type-system-tightening-design.md`
- `docs/superpowers/plans/2026-04-04-type-system-tightening.md`

### 阶段 2：`TypeExpr` + 字段/变量/List 索引约束

目标：

- `$Type` 字段支持 `User` / `User?` / `List<User>` / `List<User>?`
- 变量与常量注解支持 `T` / `List<T>`
- `?` 允许缺失字段，也允许显式 `null`
- `List<T>` 的索引赋值持续校验
- 统一引入 `TypeExpr`

状态：已完成

已完成内容：

- 前端 AST 已引入 `TypeExpr`
- `$Type` 字段类型已迁到 `TypeExpr`
- 变量 / 常量注解已迁到 `TypeExpr`
- `List<User?>` 仍然明确不支持
- `posts[0] = value` 会按 `List<T>` 约束元素类型
- shared validator 已按 `TypeExpr` 递归校验

主要设计与计划文件：

- `docs/superpowers/specs/2026-04-04-type-system-phase2-design.md`
- `docs/superpowers/plans/2026-04-04-type-system-phase2.md`

### 阶段 2.5：typed list 可变写入口

目标：

- 补上 `List<T>` 在 list mutator 上的主要写入破口

范围：

- 只覆盖 `push`
- 只覆盖 `insert`
- 不扩到 `pop` / `remove_at` / `clear` / `reverse` / `sort`

状态：已完成

已完成内容：

- `posts.push(value)` 会按 `List<T>` 元素类型校验
- `posts.insert(index, value)` 会按 `List<T>` 元素类型校验
- 未注解的动态 list 仍保持原有宽松行为

主要设计与计划文件：

- `docs/superpowers/specs/2026-04-05-typed-list-mutators-design.md`
- `docs/superpowers/plans/2026-04-05-typed-list-mutators.md`

### 阶段 2.6：函数 / HTTP 返回类型 AST 统一到 `TypeExpr`

目标：

- 把函数 / HTTP handler / 匿名函数的 `return_type` 从 `String` 迁到 `TypeExpr`
- 让返回类型与 `$Type` 字段、变量注解使用同一套结构化类型表达
- shared return validator 主路径直接消费 `TypeExpr`

状态：进行中，代码已完成到工作区，尚未提交

当前已完成内容：

- frontend AST 的函数 / HTTP / `FnLiteral` 返回类型已改为 `Option<TypeExpr>`
- parser 已复用 `parse_type_expr()` 解析返回类型
- runtime 的 shared return validator 已改为接收 `Option<&TypeExpr>`
- HTTP probe body 与 route metadata 已改为按 `TypeExpr` 工作
- CLI test/backend 的返回类型边界已适配 `TypeExpr`

当前通过的验证：

- `cargo test -p dolang-frontend -- --nocapture`
- `cargo test -p dolang-runtime shared_return_type_validator -- --nocapture`
- `cargo test -p dolang-runtime type_validation -- --nocapture`
- `cargo test --test integration_suite strict_type_construction_fixture_passes -- --nocapture`
- `cargo test --test integration_suite optional_null_and_list_construction_fixture_passes -- --nocapture`
- `cargo test --test integration_suite typed_list_ -- --nocapture`
- `cargo test --test integration_suite http_handler_user_return_type_bootstraps_in_test_mode -- --nocapture`
- `cargo test --test integration_suite http_handler_return_type_mismatch_fails_in_test_mode_bootstrap -- --nocapture`

对应设计与计划文件：

- `docs/superpowers/specs/2026-04-05-return-type-typeexpr-design.md`
- `docs/superpowers/plans/2026-04-05-return-type-typeexpr.md`

当前未完成动作：

- 提交这批返回类型 `TypeExpr` 迁移代码
- 如需要，补文档说明“返回类型内部已统一到 `TypeExpr`，但语言能力未扩张”

### 阶段 2.7：参数类型注解

目标：

- 普通函数参数支持 `TypeExpr`
- HTTP handler 参数支持 `TypeExpr`
- 匿名函数参数支持 `TypeExpr`
- 参数绑定时按声明类型做 runtime 校验

状态：进行中，代码已完成到工作区，尚未提交

当前已完成内容：

- 已引入共享参数模型 `FnParam { name, type_annotation }`
- 普通函数、HTTP handler、匿名函数参数已从 `Vec<String>` 迁到 `FnParam`
- 普通函数 / 模块函数 / 匿名函数参数绑定已接入共享参数类型校验
- HTTP handler 的真实执行路径和 test-mode probe 路径都已接入参数类型校验
- variadic 参数仍保持“只收集，不加类型注解”的保守边界

当前通过的验证：

- `cargo test -p dolang-frontend -- --nocapture`
- `cargo test --test integration_suite typed_function_parameter_ -- --nocapture`
- `cargo test --test integration_suite typed_module_function_parameter_rejects_wrong_value -- --nocapture`
- `cargo test --test integration_suite typed_http_handler_parameter_ -- --nocapture`
- `cargo test --test integration_suite typed_fn_literal_parameter_accepts_matching_value -- --nocapture`
- `cargo test --test integration_suite typed_list_ -- --nocapture`
- `cargo test --test integration_suite http_handler_user_return_type_bootstraps_in_test_mode -- --nocapture`
- `cargo test --test integration_suite http_handler_return_type_mismatch_fails_in_test_mode_bootstrap -- --nocapture`
- `cargo test -p dolang-runtime shared_return_type_validator -- --nocapture`
- `cargo test -p dolang-runtime type_validation -- --nocapture`

对应设计与计划文件：

- `docs/superpowers/specs/2026-04-05-parameter-type-annotations-design.md`
- `docs/superpowers/plans/2026-04-05-parameter-type-annotations.md`

当前未完成动作：

- 提交这批参数类型注解代码
- 如需要，补主线 guide/reference 对参数类型注解的文档口径

## 当前整体执行到哪里

如果按路线看，现在已经完成到：

1. 阶段 1：完成
2. 阶段 2：完成
3. 阶段 2.5：完成
4. 阶段 2.6：进行中，代码基本完成并通过 targeted 验证，但还未提交
5. 阶段 2.7：进行中，代码基本完成并通过 targeted 验证，但还未提交

也就是说，`$Type` 现在已经不是“可有可无”的说明性标签了，主链路已经具备：

- 显式构造约束
- 字段赋值约束
- 变量注解约束
- `List<T>` 索引赋值约束
- `List<T>` 的 `push` / `insert` 约束
- 函数 / HTTP 返回值强校验

## 已完成清单

- `$Type` 构造严格校验
- typed instance 字段赋值严格校验
- 普通函数返回值强校验
- HTTP handler 返回值强校验
- `$Type` 字段类型支持 `T` / `T?` / `List<T>` / `List<T>?`
- 变量 / 常量注解支持 `T` / `List<T>`
- `?` 允许 `null`
- `List<T>` 索引赋值校验
- `List<T>` 的 `push` / `insert` 校验
- 返回类型 AST 迁移到 `TypeExpr` 的主链代码已完成
- 参数类型注解主链代码已完成

## 还没有完成的部分

### A. 参数类型注解

还没有实现：

- 函数参数注解 `fn get_user(id: Int)`
- HTTP handler 参数注解
- 参数绑定时按 `TypeExpr` 校验

这是下一阶段最自然的延伸点之一。

### B. 其余 list 可变方法的类型约束

当前只覆盖：

- `push`
- `insert`

还没有覆盖：

- 如果未来新增 `append` / `extend`
- 以及其他会引入新元素的 mutator

当前已有的 `pop` / `remove_at` / `clear` / `reverse` / `sort` 不属于“新元素写入”，所以不在当前阻塞列表里。

### C. legacy string type helper 的进一步清理

`parse_runtime_type_expr` 仍保留在 runtime 里，作为过渡 helper。

当前情况是：

- 它已经不再承担函数 / HTTP 返回值主链
- 但还没有完全从代码库中退场

这不是功能缺失，更像技术债清理项。

### D. 更复杂的类型表达

明确还没有支持：

- `List<User?>`
- 元素级 optional 语义
- 联合类型
- 更复杂泛型
- 自动 `Map/JSON -> $Type` 转换

这些都属于后续扩展，不在当前主线必做范围。

## 建议的后续顺序

### 方案 A：先收尾当前进行中的 2.6

建议动作：

1. 提交返回类型 `TypeExpr` 迁移代码
2. 视需要补一小段文档
3. 再进入下一阶段

这是最稳的顺序。

### 方案 B：下一阶段做参数类型注解

建议目标：

- 函数参数支持 `TypeExpr`
- HTTP handler 参数支持 `TypeExpr`
- 参数绑定时强校验

这是把 `$Type` 从“返回/变量边界强约束”继续推进到“调用边界也强约束”的关键一步。

### 方案 C：继续做类型系统清理和统一

例如：

- 清理 legacy string type parsing
- 进一步统一 error message
- 统一 docs 里的返回类型教学口径

## 当前工作区状态

截至这份文件生成时，工作区里仍有一批未提交改动，主要就是“返回类型迁到 `TypeExpr`”这一轮的代码和计划文件。

当前未提交文件包括：

- `crates/dolang-cli/src/backends/axum_backend.rs`
- `crates/dolang-cli/src/test.rs`
- `crates/dolang-frontend/src/ast/ast.rs`
- `crates/dolang-frontend/src/parser/literal.rs`
- `crates/dolang-frontend/src/parser/stmt/functions.rs`
- `crates/dolang-frontend/src/parser/stmt/http.rs`
- `crates/dolang-frontend/src/parser/stmt/mod.rs`
- `crates/dolang-runtime/src/interpreter/exec/functions.rs`
- `crates/dolang-runtime/src/interpreter/exec/http.rs`
- `crates/dolang-runtime/src/interpreter/mod.rs`
- `crates/dolang-runtime/src/interpreter/value.rs`
- `crates/dolang-runtime/src/runtime/context.rs`
- `crates/dolang-runtime/src/runtime/http.rs`
- `docs/superpowers/plans/2026-04-05-return-type-typeexpr.md`

## 一句话总结

`$Type` 主线已经完成了“构造 + 字段 + 变量 + List 索引 + push/insert + 返回值”的核心约束闭环；当前正处在“把返回类型内部表示彻底统一到 `TypeExpr`”这一步，代码已基本完成，下一步最合理的是先提交这一轮，再决定是否进入“参数类型注解”阶段。

## TODO List

### P0: 先收尾当前进行中的返回类型迁移

- [ ] 提交“函数 / HTTP 返回类型 AST 迁到 `TypeExpr`”这一轮代码
- [ ] 复核当前 working tree 中与返回类型迁移相关的文件是否都应进入同一提交
- [ ] 视需要补一小段文档，说明“返回类型内部表示已统一到 `TypeExpr`，但语言能力没有新增”

### P1: 参数类型注解

- [x] 为普通函数参数引入 `TypeExpr` 注解能力
- [x] 为 HTTP handler 参数引入 `TypeExpr` 注解能力
- [x] 为匿名函数参数引入 `TypeExpr` 注解能力
- [x] 在参数绑定阶段按 `TypeExpr` 做 runtime 校验
- [x] 统一参数类型错误消息格式
- [x] 增加普通函数参数类型成功 / 失败用例
- [x] 增加 HTTP handler 参数类型成功 / 失败用例
- [x] 增加匿名函数参数类型成功用例
- [x] 增加 `$Type` 参数拒绝裸 `Map` 的用例
- [ ] 提交参数类型注解这一轮代码
- [x] 如需要，补 guide/reference 对参数类型注解的主线文档口径

### P2: 继续补齐容器写入口

- [ ] 如果未来新增 `append` / `extend` / 类似 list mutator，接入 `List<T>` 元素类型校验
- [ ] 明确哪些 mutator 属于“写入新元素”，哪些不需要类型准入检查
- [ ] 为新增 mutator 补 targeted integration tests

### P3: 清理 legacy string type helper

- [x] 盘点仍然依赖字符串类型表达的内部 helper
- [x] 评估 `parse_runtime_type_expr` 还剩哪些真实调用场景
- [x] 把非必要的字符串桥接路径逐步迁到 `TypeExpr`
- [x] 删除已经不再需要的 legacy helper

### P4: 更复杂类型表达的后续评估项

- [ ] 评估是否需要支持 `List<User?>`
- [ ] 评估是否需要支持参数 optional 语义
- [ ] 评估是否需要支持联合类型
- [ ] 评估是否需要支持更复杂泛型
- [ ] 明确是否永远禁止自动 `Map/JSON -> $Type` 转换

### 推荐执行顺序

- [ ] 第一步：提交当前返回类型 `TypeExpr` 迁移
- [ ] 第二步：提交参数类型注解与返回类型 `TypeExpr` 两轮仍在工作区内的代码
- [x] 第三步：清理 legacy string type helper
- [ ] 第四步：最后再评估更复杂类型表达是否进入主线
T
