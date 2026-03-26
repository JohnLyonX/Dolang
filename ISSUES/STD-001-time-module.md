# STD-001 std.time 原生模块

**Epic**: [EPIC-01](EPIC-01-stdlib-enhancement.md)
**优先级**: P1
**状态**: Done

---

## 问题概述

Dolang 目前完全没有时间 / 日期 API。任何 Web 服务都需要：
- 获取当前时间戳（日志、数据库记录）
- 日期格式化（HTTP 响应、用户展示）
- 日期计算（过期时间、统计区间）

本 Issue 在 `std.time` 原生模块中实现 13 个函数，覆盖上述全部场景。

---

## TODO 清单

**依赖准备**
* 检查 `crates/dolang-runtime/Cargo.toml` 是否已有 `chrono`，若无则添加 `chrono = { version = "0.4", features = ["clock"] }`

**核心实现**
* 新建 `crates/dolang-runtime/src/stdlib_native/time.rs`，实现以下函数：
  - `time.now()` → Int（当前 Unix 秒级时间戳）
  - `time.now_ms()` → Int（当前 Unix 毫秒级时间戳）
  - `time.format(ts: Int, fmt: String)` → String（strftime 格式，如 `"%Y-%m-%d"`）
  - `time.parse(s: String, fmt: String)` → Int（字符串解析为 Unix 时间戳）
  - `time.year(ts: Int)` → Int
  - `time.month(ts: Int)` → Int（1–12）
  - `time.day(ts: Int)` → Int（1–31）
  - `time.hour(ts: Int)` → Int
  - `time.minute(ts: Int)` → Int
  - `time.second(ts: Int)` → Int
  - `time.weekday(ts: Int)` → String（`"Monday"` 等）
  - `time.add_days(ts: Int, n: Int)` → Int（偏移 n 天后的时间戳）
  - `time.diff_days(ts1: Int, ts2: Int)` → Int（两个时间戳相差天数，可负）

**注册**
* 在 `crates/dolang-runtime/src/stdlib_native/mod.rs` 中：
  - `mod time;`
  - `register_time` 函数注册到 `"std.time"` 模块

**测试**
* 新建 `tests/spec/valid/stdlib/time_functions.dol`，覆盖：
  - `time.now()` 返回正整数
  - `time.format(0, "%Y-%m-%d")` 返回 `"1970-01-01"`
  - `time.year(0)` 返回 `1970`
  - `time.add_days(0, 1)` 返回 `86400`
  - `time.diff_days(86400, 0)` 返回 `1`

**文档**
* 更新 `docs/CHANGELOG.md` Unreleased/Added 区域

---

## 验收标准

* `cargo build` 编译通过，无警告
* `cargo test` 全部通过
* `time.now()` 返回当前 Unix 时间戳（Int，正整数）
* `time.format(0, "%Y-%m-%d")` 返回字符串 `"1970-01-01"`
* `time.parse("1970-01-01", "%Y-%m-%d")` 返回 `0`（UTC）
* `time.add_days(ts, 1)` 等于 `ts + 86400`
* `time.diff_days(ts + 86400, ts)` 返回 `1`
* 传入非 Int 时间戳应报运行时错误，可被 `$try/$catch` 捕获

---

## 评论

<!-- 由 @用户 填写，记录决策、阻塞、进展 -->
