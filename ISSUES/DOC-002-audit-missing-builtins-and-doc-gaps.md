# DOC-002 核对代码后发现的文档遗漏与偏差

## 背景

基于当前实现，对以下范围做了交叉核对：

- 内置方法实现：
  - `crates/dolang-runtime/src/interpreter/builtins/str_methods.rs`
  - `crates/dolang-runtime/src/interpreter/builtins/list.rs`
  - `crates/dolang-runtime/src/interpreter/builtins/map.rs`
  - `crates/dolang-runtime/src/interpreter/builtins/number.rs`
  - `crates/dolang-runtime/src/interpreter/builtins/bool_methods.rs`
  - `crates/dolang-runtime/src/interpreter/builtins/file.rs`
  - `crates/dolang-runtime/src/interpreter/builtins/json.rs`
  - `crates/dolang-runtime/src/interpreter/builtins/html.rs`
- native stdlib 实现：
  - `crates/dolang-runtime/src/stdlib_native/*.rs`
- 当前文档：
  - `docs/reference/*`
  - `docs/spec/*`
  - `docs/guide/*`

目标不是修复，而是记录“文档还没覆盖到的能力”以及“文档和实现不一致的地方”。

---

## 结论概览

这次核对发现两类问题：

1. **代码里已有能力，但文档没有正式说明**
2. **文档写法与当前实现不一致**

### 本次测试记录

- 脚本模式：`cargo run -- run func_test/run/doc002_run.dol`
- REPL 纯内建：`cargo run < func_test/repl/doc002_repl.dol`
- REPL `std.*` 模块：`cargo run < func_test/repl/doc002_repl_std_modules.dol`
- serve 模式：在 `func_test/serve/` 下执行 `cargo run -- serve .`
- serve 路由探针：`cargo run -- run func_test/run/doc002_serve_probe.dol`

对应测试文件：

- `func_test/run/doc002_run.dol`
- `func_test/run/doc002_serve_probe.dol`
- `func_test/repl/doc002_repl.dol`
- `func_test/repl/doc002_repl_std_modules.dol`
- `func_test/serve/main.dol`

---

## 问题一：`File` / `Json` / `Html` 值的方法几乎没有正式文档 ✅（脚本模式、serve 路由、REPL 纯内建均验证可用；`File.exists/size/is_dir/read/read_lines/type`、`Json.type/to_str`、`Html.type/to_str/link` 都有测试覆盖）

### 代码证据

- `File` 值支持：
  - `exists`
  - `size`
  - `is_dir`
  - `read`
  - `read_lines`
  - `content`
  - `type`
- 见：`crates/dolang-runtime/src/interpreter/builtins/file.rs`

- `Json` 值支持：
  - `to_str`
  - `type`
- 见：`crates/dolang-runtime/src/interpreter/builtins/json.rs`

- `Html` 值支持：
  - `link`
  - `to_str`
  - `type`
- 见：`crates/dolang-runtime/src/interpreter/builtins/html.rs`

### 当前文档状态

- 文档里基本只零散提到：
  - `$<<FILE("path")` 会返回 `File`
  - `$HTML().link(...)`
  - security model 里提过 `File` 读写类方法
- 但没有一页正式列出这些值可用的方法、返回值和行为边界。

### 建议后续补写

- `docs/reference/` 需要新增或扩充“特殊值类型 / 内置方法”说明：
  - `File` 方法表
  - `Json` 方法表
  - `Html` 方法表
- `guide` 里至少应补一个“这些值也有方法，不只是 String/List/Map”的入口说明。

---

## 问题二：`std.time` 的函数级文档覆盖明显不完整 ❌（能力本身在脚本模式与 serve 路由可用；但 REPL 下 `$mod std.time;` 直接报 `module not found`，REPL 模式不可达）

### 代码证据

`std.time` 当前导出：

- `now`
- `now_ms`
- `format`
- `parse`
- `year`
- `month`
- `day`
- `hour`
- `minute`
- `second`
- `weekday`
- `add_days`
- `diff_days`

见：`crates/dolang-runtime/src/stdlib_native/time.rs`

### 当前文档状态

- 在主线 guide 中只看到少量示例级提及：
  - `time.format(...)`
  - `time.year(...)`
- 在 `docs/guide/stdlib.md` 中，没有 `std.time` 独立章节和函数表。

### 建议后续补写

- 在 `docs/guide/stdlib.md` 或新的 reference 页面中补齐 `std.time` 全量函数表。
- 尤其需要补清楚：
  - `now` vs `now_ms`
  - `parse` / `format` 的格式字符串约定
  - `weekday` 返回值形式
  - `add_days` / `diff_days` 的单位和 UTC 语义

---

## 问题三：`std.http` 的函数级文档覆盖不完整 ❌（能力本身在脚本模式已验证 `get/post/put/delete/request` 都可用；但 REPL 下 `$mod std.http;` 报 `module not found`。serve 模式里的自调用探针返回 502，并伴随 tokio runtime drop panic，这更像同进程回环测试的副作用，不能单独当成功能缺失证据）

### 代码证据

`std.http` 当前导出：

- `get`
- `post`
- `put`
- `delete`
- `request`

见：`crates/dolang-runtime/src/stdlib_native/http_client.rs`

返回值统一是：

- `{ status, body, headers }`

### 当前文档状态

- `security model` 里能看到：
  - `http.get(...)`
  - `http.post(...)`
  - `http.put(...)`
  - `http.delete(...)`
  - `http.request(...)`
- 但正式用户文档没有完整函数表。
- `guide` 里现在对 `std.http` 的说明不稳定，尤其缺少：
  - `delete`
  - `request`
  - headers 参数位置
  - body 必须是 `Map` / `Json`
  - 统一返回结构说明

### 建议后续补写

- 给 `std.http` 单独一节，补全签名、参数位置和返回结构。
- `request(method, url, body, headers)` 应明确说明是“全量入口”，不是只支持 GET/POST。

---

## 问题四：`std.uuid` 文档没有覆盖 `uuid.v4()` ❌（能力本身在脚本模式与 serve 路由可用；但 REPL 下 `$mod std.uuid;` 报 `module not found`，REPL 模式不可达）

### 代码证据

`std.uuid` 当前导出：

- `v4`
- `is_valid`

见：`crates/dolang-runtime/src/stdlib_native/uuid.rs`

### 当前文档状态

- 文档里能搜到 `uuid.is_valid(...)`
- 但没有看到 `uuid.v4()` 的正式说明

### 建议后续补写

- 在 `std.uuid` 相关文档里补上：
  - `uuid.v4() -> String`
  - `uuid.is_valid(s) -> Bool`

---

## 问题五：`length()` 别名没有文档 ✅（`String/List/Map.length()` 在脚本模式、serve 路由、REPL 纯内建里都返回了预期结果）

### 代码证据

以下类型同时支持 `len` 和 `length`：

- `String`
- `List`
- `Map`

见：

- `crates/dolang-runtime/src/interpreter/builtins/str_methods.rs`
- `crates/dolang-runtime/src/interpreter/builtins/list.rs`
- `crates/dolang-runtime/src/interpreter/builtins/map.rs`

### 当前文档状态

- 文档统一只写了 `.len()`
- 没有说明 `.length()` 也是可用别名

### 备注

这个问题优先级不如前面几项高，但如果要写“完整方法表”，应该一起补齐。

---

## 问题六：`String.slice()` 文档写“支持负索引”，实现并不支持 ❌（脚本模式、serve 路由、REPL 纯内建下 `"hello".slice(-1, 5)` 都报 `invalid slice indices`）

### 代码证据

`String.slice(start, end)` 当前实现直接把参数当 `usize`：

- 负数不会按“从尾部倒数”处理
- 超界会报 `invalid slice indices`

见：`crates/dolang-runtime/src/interpreter/builtins/str_methods.rs`

### 当前文档状态

`docs/guide/stdlib.md` 当前写法：

> `s.slice(start, end)` | `String` | 切片，支持负索引

### 影响

- 这是**文档与实现不一致**
- 容易误导用户把 `String.slice()` 当成 `List.slice()` 一样使用

### 建议后续补写

- 要么把文档改成“不支持负索引”
- 要么未来实现补上负索引，再保留现有文案

---

## 问题七：`s.len()` 与 `str.len(s)` 的长度语义未区分 ❌（脚本模式与 serve 路由都验证到 `"你a"` 的 `s.len()` 为 4、`str.len(s)` 为 2；但 REPL 下 `$mod std.str;` 报 `module not found`，REPL 里无法完整复现对照）

### 代码证据

- `s.len()` 使用 `s.len()`，按 **byte 长度** 计算
  - 见：`crates/dolang-runtime/src/interpreter/builtins/str_methods.rs`
- `str.len(s)` 使用 `chars().count()`，按 **字符数** 计算
  - 见：`crates/dolang-runtime/src/stdlib_native/str.rs`

### 当前文档状态

- 两边文档都只写“长度”
- 没有说明两者在多字节字符场景下可能不同

### 影响

- 这是高价值的语义缺口
- 对中文、emoji、多字节文本处理会直接造成认知偏差

### 建议后续补写

- 至少在 `docs/guide/stdlib.md` 和 reference 中明确区分：
  - `s.len()` 的当前实现语义
  - `str.len(s)` 的当前实现语义

---

## 优先级建议

### P0

- `String.slice()` “支持负索引”的错误文案
- `s.len()` vs `str.len(s)` 未区分语义

### P1

- `File` / `Json` / `Html` 值方法缺失正式文档
- `std.time` 函数表缺失
- `std.http` 函数表缺失
- `std.uuid.v4()` 缺失

### P2

- `length()` 别名未写入文档
