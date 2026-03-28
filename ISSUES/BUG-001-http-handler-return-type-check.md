# BUG-001: HTTP Handler 返回类型运行时校验未实现

## 状态
已修复（`RuntimeMode::Test` 下已实现 bootstrap-time return type 校验）

## 现象

运行以下代码时，期望报错，但实际静默通过：

```dolang
$GET("/users/:id") get_user(id) -> Int {
    $# {"id": id};
}
```

**期望错误：**
```
http handler 'get_user' expects return type 'Int' but got 'Map'
```

**实际输出：**
```
[INFO] HTTP route registered: GET /users/:id
```
（无错误，程序正常退出）

## 失败测试

```
tests/spec/invalid/http/http_handler_return_type_mismatch.dol
```

原始复现命令：
```bash
cargo test invalid_spec_samples_fail_with_expected_error
```

当前说明：

- 本 bug 对应的 HTTP return-type 回归已修复
- 若全量 `invalid_spec_samples_fail_with_expected_error` 仍失败，需另查仓库内无关的 syntax fixtures（当前已知包含 `function_json_generic_return_type.dol`）

## 根本原因

`exec/http.rs` 在注册 HTTP handler 时，不执行 handler body，因此无法在 `dolang run` / Test 模式下检测到实际返回值与声明的 `-> ReturnType` 不符。

返回类型注解曾经对 HTTP handler 只起文档作用，运行时没有任何校验逻辑。

## 修复结果

- `RuntimeMode::Test` 下，HTTP handler 注册前会做一次 isolated probe
- probe 结果与普通 `$fn` 共用同一套返回类型验证器
- 现在可以稳定报出：
  - `http handler 'get_user' expects return type 'Int' but got 'Map'`
  - `http handler 'get_user' references unknown return type 'User'`
- `serve` 模式行为保持不变

## 涉及文件

- `crates/dolang-runtime/src/interpreter/exec/http.rs` — handler 注册与执行逻辑
- `tests/spec/invalid/http/http_handler_return_type_mismatch.dol` — spec 测试
- `tests/spec/invalid/http/http_handler_return_type_mismatch.dol.error` — 期望错误片段

## 实现方向

在 `RuntimeMode::Test` 下，handler 注册完成后，用 dummy 参数调用一次 handler body，比对实际返回值的类型与声明的 return type：

- 若类型不符 → 报错，格式：`http handler '{name}' expects return type '{declared}' but got '{actual}'`
- 若 return type 为用户自定义类型（如 `User`、`List<User>`），则校验 `DolangValue` 的 `type_name` 或 List 元素类型

## 优先级

低（不影响 serve 运行时行为，仅影响开发阶段的类型安全提示）
