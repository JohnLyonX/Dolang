# STD-002 std.uuid 模块

**Epic**: [EPIC-01](EPIC-01-stdlib-enhancement.md)
**优先级**: P2
**状态**: Done

---

## 问题概述

生成唯一 ID 是后端接口的基础需求：数据库主键、日志追踪 ID、请求去重 Token 等场景均依赖 UUID。
目前 Dolang 无任何唯一 ID 生成能力，开发者只能用 `time.now_ms()` 拼接，不符合标准。

本 Issue 实现 `std.uuid` 模块，提供 UUID v4 生成与格式校验能力。

---

## TODO 清单

**依赖准备**
* 在 `crates/dolang-runtime/Cargo.toml` 添加 `uuid = { version = "1", features = ["v4"] }`

**核心实现**
* 新建 `crates/dolang-runtime/src/stdlib_native/uuid.rs`，实现：
  - `uuid.v4()` → String（生成随机 UUID v4，如 `"550e8400-e29b-41d4-a716-446655440000"`）
  - `uuid.is_valid(s: String)` → Bool（校验字符串是否符合 UUID 格式）

**注册**
* 在 `crates/dolang-runtime/src/stdlib_native/mod.rs` 中：
  - `mod uuid;`
  - `register_uuid` 函数注册到 `"std.uuid"` 模块

**测试**
* 新建 `tests/spec/valid/stdlib/uuid_functions.dol`，覆盖：
  - `uuid.v4()` 返回长度为 36 的字符串
  - `uuid.is_valid(uuid.v4())` 返回 `true`
  - `uuid.is_valid("not-a-uuid")` 返回 `false`
  - `uuid.is_valid("")` 返回 `false`
  - 连续调用 `uuid.v4()` 两次结果不相同

**文档**
* 更新 `docs/CHANGELOG.md` Unreleased/Added 区域

---

## 验收标准

* `cargo build` 编译通过
* `cargo test` 全部通过
* `uuid.v4()` 返回格式为 `xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx` 的字符串（v4 特征）
* 连续两次调用 `uuid.v4()` 结果不同
* `uuid.is_valid("550e8400-e29b-41d4-a716-446655440000")` 返回 `true`
* `uuid.is_valid("invalid")` 返回 `false`
* 参数类型错误时报运行时错误，可被 `$try/$catch` 捕获

---

## 评论

<!-- 由 @用户 填写，记录决策、阻塞、进展 -->
