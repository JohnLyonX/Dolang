# Dolang Documentation Issues

面向用户的文档改进追踪。所有 issue 均基于对 `docs/guide/` 及相关文档的全面审计（2026-03-28）。

## 优先级说明

| 级别 | 含义 |
|------|------|
| P0 | 文档内容有矛盾或误导，用户按文档操作会出错 |
| P1 | 重要功能缺失文档，用户无法独立完成基本任务 |
| P2 | 章节内容薄弱，无法支撑学习目标 |
| P3 | 覆盖不完整，影响体验但不阻塞使用 |

---

## Issue 列表

### P0 — 内容矛盾，有误导风险

| Issue | 标题 |
|-------|------|
| [DOC-001](DOC-001-file-read-api-inconsistency.md) | `$<<FILE()` API 在 reference 和 examples.md 中描述不一致 |
| [DOC-002](DOC-002-wildcard-import-behavior-contradiction.md) | 通配符导入 `$mod math.*` 的行为在两处文档描述矛盾 |
| [DOC-003](DOC-003-list-return-type-enforcement-undocumented.md) | `-> List` 裸类型声明已被强制拒绝，但文档未反映此变更 |

### P1 — 重要功能无文档

| Issue | 标题 |
|-------|------|
| [DOC-004](DOC-004-map-write-syntax-undocumented.md) | Map 写入/赋值语法在所有文档中均未说明 |
| [DOC-005](DOC-005-catch-err-type-undocumented.md) | `$catch err` 中 `err` 的类型与可用字段从未说明 |
| [DOC-006](DOC-006-main-entrypoint-undocumented.md) | `$main() {}` 入口点没有专门解释，仅在示例中隐式出现 |
| [DOC-007](DOC-007-param-type-annotation-undocumented.md) | 函数参数类型注解语法在所有章节中均未涉及 |
| [DOC-008](DOC-008-runtime-error-catchability-undocumented.md) | 哪些运行时错误可以被 `$catch` 捕获，文档没有说明 |

### P2 — 章节内容薄弱

| Issue | 标题 |
|-------|------|
| [DOC-009](DOC-009-chapter17-testing-needs-rewrite.md) | 第 17 章（测试与调试）是全书最薄弱章节，缺乏实质内容 |
| [DOC-010](DOC-010-chapter18-patterns-is-skeleton.md) | 第 18 章（模式与配方）是占位骨架，缺少完整可运行示例 |
| [DOC-011](DOC-011-syntax-cheatsheet-incomplete.md) | 语法速查表（附录 A）覆盖不足 30% 的语法关键字 |
| [DOC-012](DOC-012-stdlib-api-reference-gap.md) | 旧 `stdlib.md` 是目前最完整的 API 参考，但处于迁移期，新文档未补齐 |

### P3 — 覆盖不完整

| Issue | 标题 |
|-------|------|
| [DOC-013](DOC-013-chapter03-missing-verification-steps.md) | 第 3 章第一个程序缺少 curl / 浏览器验证步骤 |
| [DOC-014](DOC-014-chapter08-collection-gaps.md) | 第 8 章集合缺少 Map 写入、索引越界行为、flatten/sort_desc 说明 |
| [DOC-015](DOC-015-chapter12-config-non-serve-behavior.md) | `$<<CONFIG()` 在非 serve 模式下的行为未文档化 |
| [DOC-016](DOC-016-chapter15-html-link-and-patch-body.md) | 第 15 章缺少 `$HTML().link()` 用法和 PATCH/PUT body 注入说明 |
| [DOC-017](DOC-017-chapter16-route-conflict-behavior.md) | 第 16 章未说明路由冲突（同路径两个 handler）的行为 |
| [DOC-018](DOC-018-old-guide-pages-migration-status.md) | 旧版 guide 页面（concepts/getting-started/web.md 等）迁移状态不清晰 |
