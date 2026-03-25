# Issue 014：用户文档重构

**类型**: Docs
**所属 Epic**: —
**优先级**: P1
**状态**: Closed ✅

## 背景

当前 `docs/guide/` 下有基础教程，但存在两个问题：
1. **内容过时**：`examples.md` 中的 `$elif`、C 风格 `$for` 等语法已不准确
2. **覆盖不足**：模块系统、标准库、Web 服务、文件运行等核心功能完全没有教程

需要在 E01 标准库基础设施完善后，以最新的语言能力为基准，重写面向用户的教程文档。

## 需要新增/重写的内容

### 重写

| 文件 | 问题 |
|------|------|
| `docs/guide/getting-started.md` | 只有 REPL，缺少 `dolang run file.dol` 基本用法 |
| `docs/guide/examples.md` | `$elif` / C 风格 for 等语法已过时，需对照实际行为修正 |

### 新增

| 文件 | 内容 |
|------|------|
| `docs/guide/stdlib.md` | 标准库使用教程（str / math / fs / json / env / path） |
| `docs/guide/modules.md` | 模块系统：`$mod`、公有/私有函数、`_$fn` vs `$fn` |
| `docs/guide/web.md` | Web 服务：`$http GET "/path"`、路由、返回值 |
| `docs/guide/types.md` | 渐进类型注解：`-> String`、`-> Int`、参数类型 |

## 前置条件

- Epic E01 全部 Issues 完成（008-013）
- 标准库 API 稳定，不再大幅变动

## 验收标准

一个从未接触过 Dolang 的开发者，按照文档从头读到尾，能够：
1. 安装并运行第一个 `.dol` 文件
2. 使用 `std.fs` 读写文件
3. 写一个带路由的 HTTP 服务
4. 引用自定义模块
