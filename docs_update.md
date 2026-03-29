# Dolang Docs Update Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 根据 `ISSUES/` 中已整理的文档问题，产出一轮面向 `docs/` 的系统修订，优先消除误导和缺口，再补齐主线 guide、reference 与迁移期页面的职责边界。

**Architecture:** 本计划按文档层级而不是按 issue 编排。先修入口与迁移状态说明，再集中处理 P0/P1 的事实性问题，随后补强 guide 弱章节、附录与 API 参考，最后统一收口旧页面的去留和互链。所有任务都是文档修改任务，不把实现变更或测试修复作为交付范围。

**Tech Stack:** Markdown, mdBook guide structure, existing `docs/guide/*`, `docs/reference/*`, `docs/spec/*`, `ISSUES/*`

---

## File Map

### Primary files to modify

- `docs/guide/README.md`
- `docs/guide/03-first-program.md`
- `docs/guide/07-functions.md`
- `docs/guide/08-collections-and-methods.md`
- `docs/guide/09-gradual-typing.md`
- `docs/guide/10-error-handling.md`
- `docs/guide/11-modules.md`
- `docs/guide/12-projects-and-package.md`
- `docs/guide/13-stdlib-overview.md`
- `docs/guide/14-io-env-config.md`
- `docs/guide/15-http-basics.md`
- `docs/guide/16-http-organization.md`
- `docs/guide/17-testing-and-debugging.md`
- `docs/guide/18-patterns-and-recipes.md`
- `docs/guide/appendix/syntax-cheatsheet.md`
- `docs/reference/errors.md`
- `docs/reference/syntax.md`

### Legacy guide files to update, deprecate, or relabel

- `docs/guide/concepts.md`
- `docs/guide/getting-started.md`
- `docs/guide/examples.md`
- `docs/guide/modules.md`
- `docs/guide/stdlib.md`
- `docs/guide/types.md`
- `docs/guide/web.md`

### New file to create

- `docs/reference/stdlib-api.md`

### Reference-only sources to consult while writing

- `ISSUES/README.md`
- `ISSUES/DOC-001-file-read-api-inconsistency.md`
- `ISSUES/DOC-002-wildcard-import-behavior-contradiction.md`
- `ISSUES/DOC-003-list-return-type-enforcement-undocumented.md`
- `ISSUES/DOC-004-map-write-syntax-undocumented.md`
- `ISSUES/DOC-005-catch-err-type-undocumented.md`
- `ISSUES/DOC-006-main-entrypoint-undocumented.md`
- `ISSUES/DOC-007-param-type-annotation-undocumented.md`
- `ISSUES/DOC-008-runtime-error-catchability-undocumented.md`
- `ISSUES/DOC-009-chapter17-testing-needs-rewrite.md`
- `ISSUES/DOC-010-chapter18-patterns-is-skeleton.md`
- `ISSUES/DOC-011-syntax-cheatsheet-incomplete.md`
- `ISSUES/DOC-012-stdlib-api-reference-gap.md`
- `ISSUES/DOC-013-chapter03-missing-verification-steps.md`
- `ISSUES/DOC-014-chapter08-collection-gaps.md`
- `ISSUES/DOC-015-chapter12-config-non-serve-behavior.md`
- `ISSUES/DOC-016-chapter15-html-link-and-patch-body.md`
- `ISSUES/DOC-017-chapter16-route-conflict-behavior.md`
- `ISSUES/DOC-018-old-guide-pages-migration-status.md`
- `docs/spec/README.md`
- `docs/spec/syntax.md`
- `docs/spec/semantics.md`
- `docs/spec/modules.md`
- `docs/spec/runtime-errors.md`

## Delivery Rules

- 只修改文档，不调整 Rust 实现、stdlib 代码或测试夹具。
- 文档中的行为描述以当前 `docs/spec/*` 与现有主线文档共识为准；必要时只做“写作前核实”，不把核实动作扩展成实现任务。
- 主线 guide 负责学习路径，reference 负责查表，spec 负责真实行为边界；任何新增内容都要遵守这三层职责。
- 迁移期旧页面必须显式标注状态：废弃、保留补充价值、或已迁移到新位置。

## Issue Coverage Matrix

- Task 1 覆盖：DOC-018
- Task 2 覆盖：DOC-001, DOC-002, DOC-003
- Task 3 覆盖：DOC-004, DOC-014
- Task 4 覆盖：DOC-005, DOC-007, DOC-008
- Task 5 覆盖：DOC-006, DOC-015, DOC-016, DOC-017, DOC-013
- Task 6 覆盖：DOC-009
- Task 7 覆盖：DOC-010
- Task 8 覆盖：DOC-011, DOC-012

### Task 1: Clarify Guide Entry Points And Legacy Page Status

**Files:**
- Modify: `docs/guide/README.md`
- Modify: `docs/guide/concepts.md`
- Modify: `docs/guide/getting-started.md`
- Modify: `docs/guide/examples.md`
- Modify: `docs/guide/modules.md`
- Modify: `docs/guide/stdlib.md`
- Modify: `docs/guide/types.md`
- Modify: `docs/guide/web.md`

- [ ] **Step 1: Add a migration-status policy block to the guide index**

在 `docs/guide/README.md` 的“迁移期旧页面”段落前后补一段状态说明，至少覆盖：
- 哪些旧页仍可阅读但不作为主线
- 哪些旧页仅保留独特补充信息
- 哪些旧页因内容冲突应视为废弃
- 用户遇到冲突时以编号章节和 `docs/spec/*` 为准

- [ ] **Step 2: Classify each legacy page inline**

为每个迁移期文件添加统一风格的页首说明块，建议只使用三类标签：
- `Deprecated`：如 `docs/guide/modules.md`
- `Supplemental`：如 `docs/guide/concepts.md`、`docs/guide/getting-started.md`
- `Migration Source`：如 `docs/guide/examples.md`、`docs/guide/stdlib.md`、`docs/guide/web.md`、`docs/guide/types.md`

说明块要包含：
- 当前状态
- 建议改去读哪一个新文档
- 本页仍保留的独特价值

- [ ] **Step 3: Mark conflicting old module guidance as deprecated**

在 `docs/guide/modules.md` 顶部写明该页已废弃，主线以 `docs/guide/11-modules.md` 为准，并显式提示通配符导入语义曾有旧说法，避免用户继续采信旧文。

- [ ] **Step 4: Mark high-value legacy pages as supplemental instead of silently leaving them**

对 `docs/guide/web.md`、`docs/guide/stdlib.md`、`docs/guide/examples.md`、`docs/guide/types.md` 补充“内容正在迁移到哪几章”的交叉链接，避免这些页面继续以“发现即使用”的状态悬空存在。

- [ ] **Step 5: Verify entry-point consistency**

验收点：
- `docs/guide/README.md` 已说明新旧文档关系
- 每个迁移期页面都有统一的状态头
- `modules.md` 的废弃状态足够醒目
- 高价值旧页都带有明确去向链接

### Task 2: Resolve P0 Contradictions In Core Guide Chapters

**Files:**
- Modify: `docs/guide/11-modules.md`
- Modify: `docs/guide/14-io-env-config.md`
- Modify: `docs/guide/15-http-basics.md`
- Modify: `docs/guide/examples.md`
- Modify: `docs/guide/modules.md`
- Modify: `docs/reference/syntax.md`
- Modify: `docs/guide/appendix/syntax-cheatsheet.md`

- [ ] **Step 1: Normalize `$<<FILE(...)` documentation across guide and reference**

在 `docs/guide/14-io-env-config.md` 新增一个独立小节，统一解释：
- 当前主线文档允许展示的 `$<<FILE(...)` 形式
- 返回值该如何表述
- 与 `std.fs.read_text`、`std.fs.read_lines` 的职责区分
- 当用户需要“按行读取”时应优先看哪种写法

同时把 `docs/reference/syntax.md` 中“返回 File 对象”这类表述改成与 guide 一致的说法；`docs/guide/examples.md` 中若仍保留双参数旧示例，要明确标注为迁移期旧写法，不可继续当主线 API 教学。

- [ ] **Step 2: Rewrite wildcard import behavior to one authoritative explanation**

在 `docs/guide/11-modules.md` 中加入一个最小可运行示例，对比：
- `$mod math.*;`
- `math.add.fn()` 或 `math.sqrt()` 这类命名空间访问方式
- 明确写出“不会平铺到当前作用域”

同时在 `docs/guide/modules.md` 的废弃说明里点名该矛盾已迁移，避免旧文继续传播“直接调用 `fn()`”的预期。

- [ ] **Step 3: Fix return-type guidance for `List<T>`**

在 `docs/guide/09-gradual-typing.md` 和 `docs/guide/15-http-basics.md` 中新增统一说明：
- 返回类型里推荐使用 `List<T>`，不要只写裸 `List`
- `List<User>` 这类写法应放在 `$Type User` 之后讲解
- HTTP handler 示例应给出一个完整的 `List<User>` 返回例子

并在 `docs/guide/appendix/syntax-cheatsheet.md` 增补“返回类型合法写法”示例，防止附录继续落后于正文。

- [ ] **Step 4: Remove contradiction sources from legacy pages**

对 `docs/guide/examples.md`、`docs/guide/modules.md` 中与当前主线冲突的段落，采用以下策略之一：
- 删除明显错误的旧示例
- 保留但在段首加“旧写法，勿用于当前主线”的说明

本任务目标不是保全旧文，而是消除继续误导用户的入口。

- [ ] **Step 5: Verify P0 closure**

验收点：
- `$<<FILE` 在 `guide` 与 `reference` 不再相互矛盾
- 通配符导入的访问方式只有一种说法
- `List<T>` 在类型章节、HTTP 章节、附录中的口径一致

### Task 3: Fill Collection And Map Usage Gaps In Learning Path

**Files:**
- Modify: `docs/guide/08-collections-and-methods.md`
- Modify: `docs/guide/13-stdlib-overview.md`
- Modify: `docs/guide/examples.md`

- [ ] **Step 1: Expand the Map section from read-only to read-write**

在 `docs/guide/08-collections-and-methods.md` 的 Map 章节补齐以下内容：
- 创建时初始化
- 运行时修改
- 如果语言层支持 `map["k"] = v`，给出直接示例
- 如果不适合作为主线推荐写法，则引导到 `std.json.set(...)`

说明中要显式区分“Map 原生操作”和“`std.json` 工具函数”。

- [ ] **Step 2: Add missing list and map behaviors**

在第 8 章补充以下项目的最小说明和示例：
- `flatten()`
- `sort_desc()`
- `count(x)` 的实际示例
- `map.len()`
- 索引越界行为说明
- `split()` 后继续做列表处理的短示例

如果 `map/filter` 这类高阶函数不适合作为当前主线内容，不要猜测支持度，只写“当前推荐用 `$for` 循环替代”的说明。

- [ ] **Step 3: Add cross-links from stdlib overview without duplicating chapter 8**

在 `docs/guide/13-stdlib-overview.md` 中只补“相关能力索引”，不要把第 8 章重复写一遍。目标是告诉读者：
- 哪些集合操作是值方法
- 哪些集合辅助能力在 `std.json` 或其他模块
- 完整 API 查哪里

- [ ] **Step 4: Clean up legacy example references**

如果 `docs/guide/examples.md` 中还保留对 `flatten()` 等旧示例的有效内容，统一给出“该示例的主线去向”链接，避免知识点散落。

- [ ] **Step 5: Verify collection chapter completeness**

验收点：
- 第 8 章不再只有 Map 读法没有写法
- issue 提到的集合缺口均能在正文找到入口
- `stdlib` 总览页只做索引，不与第 8 章冲突

### Task 4: Rework Type And Error Handling Guidance

**Files:**
- Modify: `docs/guide/07-functions.md`
- Modify: `docs/guide/09-gradual-typing.md`
- Modify: `docs/guide/10-error-handling.md`
- Modify: `docs/reference/errors.md`
- Modify: `docs/guide/types.md`
- Modify: `docs/guide/appendix/syntax-cheatsheet.md`

- [ ] **Step 1: Add a clear parameter type annotation stance**

在 `docs/guide/07-functions.md` 和 `docs/guide/09-gradual-typing.md` 统一补一段“参数类型注解现状”说明。写法要求：
- 只写当前主线允许承诺的结论
- 若主线暂不准备教学参数类型注解，则明确说明原因和替代手段
- 若决定收入口径，则给出最小函数签名示例

同时在 `docs/guide/types.md` 的迁移说明里同步口径，不允许旧页和新页一边讲支持、一边不讲。

- [ ] **Step 2: Make `err` observable and understandable**

在 `docs/guide/10-error-handling.md` 增加一个专门小节解释：
- `$catch err` 中 `err` 应如何理解
- 用户能安全做哪些操作
- 可展示哪些简单示例，例如打印、比较、类型判断

避免把这一节写成运行时内部实现细节说明；重点是用户如何判断和处理错误。

- [ ] **Step 3: Add a catchability matrix**

在 `docs/guide/10-error-handling.md` 和 `docs/reference/errors.md` 协同补一张表，覆盖：
- `$throw`
- 除零
- 未定义变量
- 类型不匹配
- 文件读取失败
- 模块加载失败
- 语法错误

guide 解释“怎么用”，reference 解释“查表时怎么看”；两边用词必须一致。

- [ ] **Step 4: Link type guidance and error guidance together**

在 `docs/guide/09-gradual-typing.md` 增加一段去 `10-error-handling.md` 的引导，解释类型注解失败属于哪类用户可见错误；在 `docs/reference/errors.md` 里反向链回 guide，避免两个页面各写各的。

- [ ] **Step 5: Update the cheatsheet for typed returns and error syntax**

在 `docs/guide/appendix/syntax-cheatsheet.md` 增补：
- `$try / $catch / $throw`
- `$>>ERR(...)`
- 类型注解最小形式
- `-> JSON<User>` / `-> List<User>` 等返回类型写法

- [ ] **Step 6: Verify terminology consistency**

验收点：
- 参数类型注解的态度在 3 个页面一致
- `err` 的说明不再缺失
- catchability 表已在 guide 和 reference 建立互链

### Task 5: Strengthen Project, I/O, And HTTP Operational Chapters

**Files:**
- Modify: `docs/guide/03-first-program.md`
- Modify: `docs/guide/12-projects-and-package.md`
- Modify: `docs/guide/14-io-env-config.md`
- Modify: `docs/guide/15-http-basics.md`
- Modify: `docs/guide/16-http-organization.md`
- Modify: `docs/guide/web.md`

- [ ] **Step 1: Add first-run verification steps to chapter 3**

在 `docs/guide/03-first-program.md` 的脚本示例和 HTTP 示例后面都补上“怎么确认成功”的段落，至少包括：
- 运行命令
- 预期输出
- HTTP 访问地址
- `curl` 验证方式

这一步只补验证指引，不扩写新概念。

- [ ] **Step 2: Document `$<<CONFIG(...)` usage boundary clearly**

在 `docs/guide/12-projects-and-package.md` 与 `docs/guide/14-io-env-config.md` 中统一写清：
- 它依赖项目配置上下文
- 非 `serve` 场景下不应被当作通用配置入口
- 推荐的跨模式兜底写法应该长什么样

目标是消除“在 `run` 模式里调用会怎样”的疑惑，即使不展开底层细节，也要给出用户可执行的推荐模式。

- [ ] **Step 3: Add a focused `$main()` section**

在 `docs/guide/12-projects-and-package.md` 或 `docs/guide/16-http-organization.md` 新增一个 `$main()` 专题段落，覆盖：
- 什么时候需要写
- 什么时候可以省略
- 它与全局初始化、全局 CORS 的关系
- 与 `entry` 字段的职责边界

同时把 `docs/guide/web.md` 里仍有独特价值的 `$main()` 说明迁移或互链过去。

- [ ] **Step 4: Fill HTTP basics gaps without bloating the chapter**

在 `docs/guide/15-http-basics.md` 增补最需要的实际用法：
- `$HTML().link(...)`
- `POST / PUT / PATCH` 的 body 注入共性
- `GET / DELETE` 不应承诺 body 注入
- 一个最小的 HTML 文件链接示例

这一步保持第 15 章“基础篇”定位，不把复杂路由组织塞进来。

- [ ] **Step 5: Document route conflict behavior in chapter 16**

在 `docs/guide/16-http-organization.md` 加入“路由冲突”提示段，告诉读者：
- 同路径同方法重复注册时要关注什么
- 当前文档建议如何避免冲突
- 到哪里继续查排障信息

如果不适合承诺底层细节，就把重点放在“避免冲突”和“排查顺序”，不要虚构机制。

- [ ] **Step 6: Verify project and HTTP chapters form one coherent path**

验收点：
- 第 3 章已有可执行验证步骤
- 第 12、14、15、16 章对项目入口、配置、HTTP 基础、路由组织的边界清楚
- `$main()` 与 `$HTML().link()` 都不再只出现在旧页面里

### Task 6: Rewrite Chapter 17 Into A Real Testing And Debugging Guide

**Files:**
- Modify: `docs/guide/17-testing-and-debugging.md`
- Modify: `docs/guide/appendix/common-errors.md`
- Modify: `docs/guide/README.md`

- [ ] **Step 1: Replace placeholder structure with a practical chapter outline**

把 `docs/guide/17-testing-and-debugging.md` 重构成至少 4 个部分：
- 断言与基本验证
- 测试文件组织与命名建议
- `dolang test` / `dolang test --route` 的使用方式
- 调试工作流与常见排障场景

- [ ] **Step 2: Add script tests and HTTP tests as parallel workflows**

章节中要同时覆盖：
- 纯脚本逻辑怎么验证
- HTTP handler 怎么验证

不要让整章只剩 `--route` 示例，否则仍然无法支撑“测试与调试”标题。

- [ ] **Step 3: Add realistic terminal snippets**

补入成功和失败两类终端输出示意，帮助用户理解：
- 一个测试通过时大概会看到什么
- 一个断言失败时会看到什么
- 如何从错误输出回到源码位置

- [ ] **Step 4: Add debugging recipes, not just principles**

至少补 3 类常见排障短场景：
- handler 返回空 body
- 模块函数没被调用
- 类型注解相关报错

每类场景都要有“观察现象 -> 检查点 -> 推荐动作”的结构。

- [ ] **Step 5: Verify chapter 17 is no longer thinner than its neighboring chapters**

验收点：
- 第 17 章已能独立指导新用户做最基本测试
- 至少同时覆盖脚本与 HTTP 两条验证路径
- 章节结构和信息密度明显高于当前占位稿

### Task 7: Rebuild Chapter 18 As Runnable Recipes

**Files:**
- Modify: `docs/guide/18-patterns-and-recipes.md`
- Modify: `docs/guide/examples.md`
- Modify: `docs/guide/getting-started.md`

- [ ] **Step 1: Decide the recipe set and keep it focused**

第 18 章不要追求一次装下所有场景。先锁定一组高频、可复制、与现有主线相连的配方，建议优先：
- 单文件脚本
- 标准库驱动小工具
- 带 `package.toml` 的小项目
- 模块化 HTTP 服务
- 读取并处理 JSON 文件
- 调用外部 API 后整形返回

- [ ] **Step 2: Rewrite each recipe to be copy-paste runnable**

每个配方至少包含：
- 文件结构
- 完整代码
- 运行命令
- 预期输出或请求结果

不得继续保留只有 3 到 10 行的占位式示例。

- [ ] **Step 3: Migrate surviving value from `examples.md` into chapter 18**

把旧 `docs/guide/examples.md` 里仍有价值的完整示例迁入第 18 章后，再把旧页改成：
- 迁移索引页
- 或精选补充示例页

不要再让 `examples.md` 与第 18 章长期并列承载同一职责。

- [ ] **Step 4: Add links back to the relevant learning chapters**

每个配方末尾都加“回看哪一章”的链接，比如：
- 集合处理链接回第 8 章
- 类型结构链接回第 9 章
- HTTP 配方链接回第 15、16 章

- [ ] **Step 5: Verify recipe chapter usefulness**

验收点：
- 第 18 章已从骨架变成可运行配方集
- 至少两个高频场景有完整、独立、可复制示例
- `examples.md` 的职责已被重新定义

### Task 8: Build A Complete Reference Layer For Syntax And Stdlib

**Files:**
- Create: `docs/reference/stdlib-api.md`
- Modify: `docs/guide/13-stdlib-overview.md`
- Modify: `docs/guide/appendix/syntax-cheatsheet.md`
- Modify: `docs/guide/stdlib.md`
- Modify: `docs/guide/README.md`
- Modify: `docs/README.md`

- [ ] **Step 1: Expand the syntax cheatsheet into a real appendix**

重写 `docs/guide/appendix/syntax-cheatsheet.md` 的结构，按功能分组至少覆盖：
- 声明与赋值
- 输入与输出
- 控制流
- 函数
- 错误处理
- 模块
- 类型系统
- HTTP handler
- HTTP 组织
- 构造器
- 注解
- 程序入口与退出

每个条目都只给最小可识别片段，避免写成长教程。

- [ ] **Step 2: Create `docs/reference/stdlib-api.md` as the authoritative API lookup page**

新文件建议结构：
- 使用说明与范围
- 值方法索引
- `std.*` 模块索引
- 按模块列 API
- 稳定性/迁移说明

目标不是把 guide 改成 API 表，而是给用户一个明确的“查完整方法列表”的地方。

- [ ] **Step 3: Turn chapter 13 into an overview that points to the new reference**

在 `docs/guide/13-stdlib-overview.md` 中加入“完整 API 参考”链接，并压实章节定位：
- 第 13 章负责认路
- `docs/reference/stdlib-api.md` 负责查表
- 旧 `docs/guide/stdlib.md` 仅作为迁移来源或补充页存在

- [ ] **Step 4: Relabel the old stdlib page**

在 `docs/guide/stdlib.md` 页首明确说明：
- 该页已迁移到新的 reference 位置
- 若仍保留内容，是为了迁移过渡，不应继续当主线入口

- [ ] **Step 5: Update top-level docs entry points**

在 `docs/guide/README.md` 和 `docs/README.md` 中补充对 `docs/reference/stdlib-api.md` 的入口链接，避免新 reference 建好后仍然没人能发现。

- [ ] **Step 6: Verify reference layer separation**

验收点：
- 附录 A 已能承担“语法速查”职责
- `stdlib` 完整 API 有独立 reference 页面
- guide 总览、旧 stdlib 页面、顶层 docs 入口都指向同一个新查表入口

## Final Review Checklist

- [ ] 所有 18 个 issue 都已映射到至少一个任务
- [ ] 没有把实现修改、运行时修复、测试夹具变更写进范围
- [ ] 任务顺序符合实际落地顺序：入口说明 -> 纠错 -> 补全 -> 重写弱章节 -> 收口 reference
- [ ] 每个任务都写明了具体文件和验收点
- [ ] guide / reference / spec / legacy 的职责边界清晰

## Suggested Execution Order

1. Task 1
2. Task 2
3. Task 4
4. Task 3
5. Task 5
6. Task 6
7. Task 7
8. Task 8

Plan complete and saved to `docs_update.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
