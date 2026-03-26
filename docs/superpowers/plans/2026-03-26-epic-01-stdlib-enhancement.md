# EPIC-01 Stdlib Enhancement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `std.time`, `std.uuid`, and `std.http` native modules to Dolang with spec coverage, runtime-safe error behavior, and the required changelog/security doc updates.

**Architecture:** Extend the existing native stdlib registry in `crates/dolang-runtime/src/stdlib_native/` with three focused Rust modules. Land deterministic modules first (`std.time`, `std.uuid`), then add `std.http` with explicit request/response normalization and local-only verification so the spec suite stays stable.

**Tech Stack:** Rust workspace, `chrono`, `uuid`, `reqwest::blocking`, Dolang spec fixtures under `tests/spec`, runtime docs under `docs/spec` and `docs/CHANGELOG.md`

---

### Task 1: Add Time Module Dependencies And Deterministic Spec Coverage

**Files:**
- Modify: `crates/dolang-runtime/Cargo.toml`
- Create: `tests/spec/valid/stdlib/time_functions.dol`
- Create: `tests/spec/valid/stdlib/time_functions.dol.stdout`
- Create: `tests/spec/valid/stdlib/time_errors.dol`
- Create: `tests/spec/valid/stdlib/time_errors.dol.stdout`

- [ ] **Step 1: Write the failing spec fixtures for `std.time`**

```dol
$mod std.time;

$>> time.format(0, "%Y-%m-%d");
$>> time.parse("1970-01-01", "%Y-%m-%d");
$>> time.year(0);
$>> time.month(0);
$>> time.day(0);
$>> time.hour(0);
$>> time.minute(0);
$>> time.second(0);
$>> time.weekday(0);
$>> time.add_days(0, 1);
$>> time.diff_days(86400, 0);

$ now = time.now();
$ now_ms = time.now_ms();
$>> now > 0;
$>> now_ms >= now * 1000;
```

```text
1970-01-01
0
1970
1
1
0
0
0
Thursday
86400
1
true
true
```

```dol
$mod std.time;

$try {
    $>> time.year("bad");
} $catch err {
    $>> "caught-year";
}

$try {
    $>> time.parse("bad-date", "%Y-%m-%d");
} $catch err {
    $>> "caught-parse";
}
```

```text
caught-year
caught-parse
```

- [ ] **Step 2: Run the new specs to verify they fail before implementation**

Run: `cargo test spec_suite -- --nocapture`
Expected: `tests/spec/valid/stdlib/time_functions.dol` and `time_errors.dol` fail because `std.time` is not registered.

- [ ] **Step 3: Add the time dependency**

```toml
[dependencies]
dolang-frontend = { path = "../dolang-frontend" }
indexmap = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
rand = "0.8"
chrono = { version = "0.4", features = ["clock"] }
```

- [ ] **Step 4: Run a focused build after adding the dependency**

Run: `cargo build -p dolang-runtime`
Expected: build succeeds and downloads/compiles `chrono`.

- [ ] **Step 5: Commit the fixture and dependency baseline**

```bash
git add crates/dolang-runtime/Cargo.toml tests/spec/valid/stdlib/time_functions.dol tests/spec/valid/stdlib/time_functions.dol.stdout tests/spec/valid/stdlib/time_errors.dol tests/spec/valid/stdlib/time_errors.dol.stdout
git commit -m "test: add std.time spec fixtures"
```

### Task 2: Implement `std.time` And Register It

**Files:**
- Create: `crates/dolang-runtime/src/stdlib_native/time.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/mod.rs`

- [ ] **Step 1: Write a focused Rust unit test in the new module**

```rust
#[cfg(test)]
mod tests {
    use super::{seconds_to_utc, timestamp_from_format};

    #[test]
    fn parses_epoch_date_as_zero() {
        assert_eq!(timestamp_from_format("1970-01-01", "%Y-%m-%d").unwrap(), 0);
    }

    #[test]
    fn exposes_epoch_components_in_utc() {
        let dt = seconds_to_utc(0).unwrap();
        assert_eq!(dt.year(), 1970);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }
}
```

- [ ] **Step 2: Run the focused runtime test and confirm it fails**

Run: `cargo test -p dolang-runtime time::tests -- --nocapture`
Expected: fail because `time.rs` and helper functions do not exist yet.

- [ ] **Step 3: Implement the module and helper conversions**

```rust
use std::sync::Arc;

use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike, Utc};

use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::{NativeFnMap, RuntimeContext};

pub fn register(context: &mut RuntimeContext) {
    let mut exports = NativeFnMap::new();
    exports.insert("now".into(), Arc::new(|_, _| Ok(DolangValue::Int(Utc::now().timestamp()))));
    exports.insert("now_ms".into(), Arc::new(|_, _| Ok(DolangValue::Int(Utc::now().timestamp_millis()))));
    exports.insert("format".into(), Arc::new(|args, _| {
        let ts = int_arg("time.format", args, 0)?;
        let fmt = string_arg("time.format", args, 1)?;
        Ok(DolangValue::Str(seconds_to_utc(ts)?.format(fmt).to_string()))
    }));
    // Continue with parse/year/month/day/hour/minute/second/weekday/add_days/diff_days
    context.register_native_module("std.time", exports);
}

fn int_arg(id: &str, args: &[DolangValue], index: usize) -> Result<i64, Error> {
    match args.get(index) {
        Some(DolangValue::Int(n)) => Ok(*n),
        Some(other) => Err(Error::Interpreter(format!("{id}: argument {index} must be Int, got {}", other.type_name()))),
        None => Err(Error::Interpreter(format!("{id}: missing argument {}", index))),
    }
}

fn string_arg<'a>(id: &str, args: &'a [DolangValue], index: usize) -> Result<&'a str, Error> {
    match args.get(index) {
        Some(DolangValue::Str(s)) => Ok(s),
        Some(other) => Err(Error::Interpreter(format!("{id}: argument {index} must be String, got {}", other.type_name()))),
        None => Err(Error::Interpreter(format!("{id}: missing argument {}", index))),
    }
}

fn seconds_to_utc(ts: i64) -> Result<DateTime<Utc>, Error> {
    Utc.timestamp_opt(ts, 0)
        .single()
        .ok_or_else(|| Error::Interpreter(format!("time: invalid timestamp {ts}")))
}

fn timestamp_from_format(input: &str, fmt: &str) -> Result<i64, Error> {
    let naive = NaiveDateTime::parse_from_str(input, fmt)
        .or_else(|_| chrono::NaiveDate::parse_from_str(input, fmt).map(|d| d.and_hms_opt(0, 0, 0).unwrap()))
        .map_err(|err| Error::Interpreter(format!("time.parse: {err}")))?;
    Ok(Utc.from_utc_datetime(&naive).timestamp())
}
```

- [ ] **Step 4: Register the new module in the stdlib registry**

```rust
mod env;
mod fs;
mod json;
mod math;
mod str;
mod time;

pub fn register_stdlib_native_modules(context: &mut RuntimeContext) {
    fs::register(context);
    env::register(context);
    str::register(context);
    math::register(context);
    json::register(context);
    time::register(context);
}
```

- [ ] **Step 5: Run the time-focused verification**

Run: `cargo test -p dolang-runtime time::tests -- --nocapture`
Expected: the new Rust unit tests pass.

Run: `cargo test spec_suite -- --nocapture`
Expected: `time_functions` and `time_errors` pass along with the existing spec suite.

- [ ] **Step 6: Commit the time module**

```bash
git add crates/dolang-runtime/src/stdlib_native/time.rs crates/dolang-runtime/src/stdlib_native/mod.rs
git add tests/spec/valid/stdlib/time_functions.dol tests/spec/valid/stdlib/time_functions.dol.stdout tests/spec/valid/stdlib/time_errors.dol tests/spec/valid/stdlib/time_errors.dol.stdout
git commit -m "feat: add std.time native module"
```

### Task 3: Add UUID Support And Spec Coverage

**Files:**
- Modify: `crates/dolang-runtime/Cargo.toml`
- Create: `crates/dolang-runtime/src/stdlib_native/uuid.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/mod.rs`
- Create: `tests/spec/valid/stdlib/uuid_functions.dol`
- Create: `tests/spec/valid/stdlib/uuid_functions.dol.stdout`
- Create: `tests/spec/valid/stdlib/uuid_errors.dol`
- Create: `tests/spec/valid/stdlib/uuid_errors.dol.stdout`

- [ ] **Step 1: Write the failing spec fixtures for `std.uuid`**

```dol
$mod std.uuid;

$ a = uuid.v4();
$ b = uuid.v4();
$>> a.len();
$>> uuid.is_valid(a);
$>> uuid.is_valid("550e8400-e29b-41d4-a716-446655440000");
$>> uuid.is_valid("not-a-uuid");
$>> a != b;
```

```text
36
true
true
false
true
```

```dol
$mod std.uuid;

$try {
    $>> uuid.is_valid(123);
} $catch err {
    $>> "caught-uuid";
}
```

```text
caught-uuid
```

- [ ] **Step 2: Run the spec suite to verify the new UUID fixtures fail**

Run: `cargo test spec_suite -- --nocapture`
Expected: UUID fixtures fail because `std.uuid` is not yet registered.

- [ ] **Step 3: Add the UUID dependency and implement the module**

```toml
uuid = { version = "1", features = ["v4"] }
```

```rust
use std::sync::Arc;

use crate::error::Error;
use crate::interpreter::DolangValue;
use crate::runtime::{NativeFnMap, RuntimeContext};

pub fn register(context: &mut RuntimeContext) {
    let mut exports = NativeFnMap::new();
    exports.insert(
        "v4".into(),
        Arc::new(|_, _| Ok(DolangValue::Str(uuid::Uuid::new_v4().to_string()))),
    );
    exports.insert(
        "is_valid".into(),
        Arc::new(|args, _| {
            let input = match args.first() {
                Some(DolangValue::Str(s)) => s,
                Some(other) => {
                    return Err(Error::Interpreter(format!(
                        "uuid.is_valid: first arg must be String, got {}",
                        other.type_name()
                    )))
                }
                None => return Err(Error::Interpreter("uuid.is_valid: missing first arg".into())),
            };
            Ok(DolangValue::Bool(uuid::Uuid::parse_str(input).is_ok()))
        }),
    );
    context.register_native_module("std.uuid", exports);
}
```

- [ ] **Step 4: Register `std.uuid` and add a focused unit test**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn recognizes_valid_uuid_strings() {
        assert!(uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }
}
```

Also update `stdlib_native/mod.rs` to call `uuid::register(context);`.

- [ ] **Step 5: Run UUID verification**

Run: `cargo test -p dolang-runtime uuid::tests -- --nocapture`
Expected: new UUID unit tests pass.

Run: `cargo test spec_suite -- --nocapture`
Expected: UUID fixtures pass and no existing spec regresses.

- [ ] **Step 6: Commit the UUID module**

```bash
git add crates/dolang-runtime/Cargo.toml crates/dolang-runtime/src/stdlib_native/uuid.rs crates/dolang-runtime/src/stdlib_native/mod.rs
git add tests/spec/valid/stdlib/uuid_functions.dol tests/spec/valid/stdlib/uuid_functions.dol.stdout tests/spec/valid/stdlib/uuid_errors.dol tests/spec/valid/stdlib/uuid_errors.dol.stdout
git commit -m "feat: add std.uuid native module"
```

### Task 4: Add HTTP Client Helpers, Local Unit Tests, And Failure Specs

**Files:**
- Modify: `crates/dolang-runtime/Cargo.toml`
- Create: `crates/dolang-runtime/src/stdlib_native/http_client.rs`
- Modify: `crates/dolang-runtime/src/stdlib_native/mod.rs`
- Create: `tests/spec/valid/stdlib/http_client_errors.dol`
- Create: `tests/spec/valid/stdlib/http_client_errors.dol.stdout`

- [ ] **Step 1: Add the failing Dolang error-path spec**

```dol
$mod std.http;

$try {
    $ res = http.get("not-a-url");
    $>> res["status"];
} $catch err {
    $>> "caught-http";
}
```

```text
caught-http
```

- [ ] **Step 2: Add focused Rust unit tests for request/response normalization**

```rust
#[cfg(test)]
mod tests {
    use super::{headers_to_map, response_to_dolang_map};
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    fn folds_response_headers_into_string_map() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let map = headers_to_map(&headers).unwrap();
        assert_eq!(map.get("content-type").unwrap().to_string(), "application/json");
    }
}
```

- [ ] **Step 3: Run the focused test target and confirm failure**

Run: `cargo test -p dolang-runtime http_client::tests -- --nocapture`
Expected: fail because `http_client.rs` is not implemented yet.

- [ ] **Step 4: Add the dependency and implement the blocking client module**

```toml
reqwest = { version = "0.12", features = ["blocking", "json"] }
```

```rust
use std::sync::Arc;
use std::time::Duration;

use indexmap::IndexMap;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;

use crate::error::Error;
use crate::interpreter::value::value_to_json;
use crate::interpreter::DolangValue;
use crate::runtime::{NativeFnMap, RuntimeContext};

pub fn register(context: &mut RuntimeContext) {
    let mut exports = NativeFnMap::new();
    exports.insert("get".into(), Arc::new(|args, _| get(args)));
    exports.insert("post".into(), Arc::new(|args, _| post(args)));
    exports.insert("put".into(), Arc::new(|args, _| put(args)));
    exports.insert("delete".into(), Arc::new(|args, _| delete(args)));
    exports.insert("request".into(), Arc::new(|args, _| request(args)));
    context.register_native_module("std.http", exports);
}

fn client() -> Result<Client, Error> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|err| Error::Interpreter(format!("http.request: {err}")))
}

fn response_to_dolang_map(status: u16, body: String, headers: HeaderMap) -> Result<DolangValue, Error> {
    let mut map = IndexMap::new();
    map.insert("status".into(), DolangValue::Int(status as i64));
    map.insert("body".into(), DolangValue::Str(body));
    map.insert("headers".into(), headers_to_map(&headers)?);
    Ok(DolangValue::Map(map))
}
```

Implement argument validation helpers for `String`, `Map`, and optional headers/body so:

- `http.get(url)`
- `http.get(url, headers)`
- `http.post(url, body)`
- `http.post(url, body, headers)`
- `http.put(url, body)`
- `http.delete(url)`
- `http.request(method, url, body, headers)`

all normalize into a single request path.

- [ ] **Step 5: Register `std.http` and run focused verification**

Update `stdlib_native/mod.rs` to call `http_client::register(context);`.

Run: `cargo test -p dolang-runtime http_client::tests -- --nocapture`
Expected: local unit tests pass.

Run: `cargo test spec_suite -- --nocapture`
Expected: `http_client_errors.dol` passes and no existing specs regress.

- [ ] **Step 6: Commit the HTTP client module**

```bash
git add crates/dolang-runtime/Cargo.toml crates/dolang-runtime/src/stdlib_native/http_client.rs crates/dolang-runtime/src/stdlib_native/mod.rs
git add tests/spec/valid/stdlib/http_client_errors.dol tests/spec/valid/stdlib/http_client_errors.dol.stdout
git commit -m "feat: add std.http native module"
```

### Task 5: Update Security Docs, Changelog, And Run Full Verification

**Files:**
- Modify: `docs/spec/security-model.md`
- Modify: `docs/CHANGELOG.md`

- [ ] **Step 1: Update the changelog**

```md
### Added

- 增加 `std.time` 原生模块，提供 Unix 时间戳、格式化、解析与日期偏移能力
- 增加 `std.uuid` 原生模块，提供 UUID v4 生成与格式校验能力
- 增加 `std.http` 原生模块，提供同步 HTTP 客户端能力
```

- [ ] **Step 2: Update the security model for outbound HTTP**

```md
## 当前副作用入口

当前仓库中已存在的主要副作用入口包括：

- 文件 I/O
- 环境变量读取
- 标准输入读取
- HTTP 服务暴露
- HTTP 出站请求
- 模块文件加载
```

Add a short subsection documenting that `std.http` issues blocking outbound HTTP requests using host network permissions and is currently allowed by default.

- [ ] **Step 3: Run full project verification**

Run: `cargo fmt --all`
Expected: formatting completes with no errors.

Run: `cargo build`
Expected: workspace build succeeds.

Run: `cargo test`
Expected: full workspace test suite passes.

- [ ] **Step 4: Commit docs and final verification**

```bash
git add docs/spec/security-model.md docs/CHANGELOG.md
git commit -m "docs: describe EPIC-01 stdlib additions"
```

- [ ] **Step 5: Capture final status**

Run: `git status --short`
Expected: clean worktree.

Run: `git log --oneline -n 5`
Expected: recent commits include the time, uuid, http, and docs checkpoints from this plan.
