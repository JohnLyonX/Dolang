# Runtime Intrinsics

本页记录 Dolang runtime intrinsic 层的设计目标和当前实现边界。

## 目标

runtime intrinsic 层用于承接所有直接触达宿主 OS / 进程环境的能力，避免这些逻辑继续散落在：

- `eval/exec`
- builtin 方法
- 其他 runtime 子模块

这层不是面向普通脚本用户的公开 API，而是未来 `std.*` 的底座。

## 当前结构

当前实现包含：

- `IntrinsicId`
  - 使用枚举标识 intrinsic，避免字符串漂移
- `IntrinsicCall`
  - 统一调用签名：`&[DolangValue] -> Result<DolangValue, Error>`
- `IntrinsicRegistry`
  - 负责注册和分发 intrinsic
- `RuntimePolicy`
  - 负责允许或拒绝某个 intrinsic

## 当前首批 intrinsic

- `FsReadText`
- `FsReadLines`
- `FsWriteText`
- `FsAppendText`
- `FsDelete`
- `FsExists`
- `FsSize`
- `FsIsDir`
- `EnvGet`
- `ConfigGet`

## 当前调用原则

- 旧语法和 builtin 表面保持不变
- `$<<FILE(...)`、`$>>FILE(...)`、`File.*`、`$<<ENV(...)`、`$<<CONFIG(...)` 的底层实现改走 intrinsic
- `RuntimePolicy` 当前默认 allow-all，不改变现有用户行为

## 与标准库的关系

本层不直接等于标准库。

关系是：

```text
language surface / builtin
  -> runtime intrinsic
  -> host OS / process environment
```

未来标准库将建立在这层之上，例如：

- `std.fs.read_text()` -> `FsReadText`
- `std.env.get()` -> `EnvGet`
- `std.config.get()` -> `ConfigGet`

## 变更规则

后续如果新增宿主能力：

1. 先进入 intrinsic 层
2. 再决定是否公开为 `std.*`
3. 同步更新安全模型与开发文档
