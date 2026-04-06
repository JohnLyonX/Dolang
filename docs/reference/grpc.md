# gRPC Client 参考

Dolang 当前提供的是 `std.grpc` 出站客户端能力。

定位：

- 面向 HTTP API 编排、BFF、集成网关
- 首版只支持 unary
- Dolang 侧请求与响应使用 `Map/JSON`
- gRPC 契约通过 descriptor 驱动

当前不提供：

- gRPC 服务端
- streaming
- 代码生成
- Dolang 原生 proto 类型系统

## 模块导入

```dol
$mod std.grpc;
```

## `grpc.client(config)`

创建一个 gRPC client。

### 参数

`config` 必须是 `Map` 或 `Json`。

支持字段：

- `target: String`
- `descriptor: String`
- `proto: String`
- `timeout_ms: Int`
- `metadata: Map<String, String>`

约束：

- `target` 必填
- `descriptor` 和 `proto` 必须二选一
- 当前版本稳定支持 `descriptor`
- 当前版本传入 `proto` 会得到显式未实现错误

### 示例

```dol
$mod std.grpc;

$ user_svc = grpc.client({
  "target": "dns:///user-service:50051",
  "descriptor": "descriptors/user_service.pb",
  "timeout_ms": 1500,
  "metadata": {
    "x-client": "dolang-gateway"
  }
});
```

## `GrpcClient.call(config)`

执行一次 unary gRPC 调用。

### 参数

`config` 必须是 `Map` 或 `Json`。

支持字段：

- `service: String`
- `method: String`
- `body: Map | Json`
- `timeout_ms: Int`
- `metadata: Map<String, String>`

约束：

- `service` 必填
- `method` 必填
- `body` 必填
- 调用级 `timeout_ms` 优先于 client 默认配置
- 调用级 `metadata` 会覆盖同名默认 metadata

### 示例

```dol
$ result = user_svc.call({
  "service": "user.v1.UserService",
  "method": "GetUser",
  "body": {
    "id": body["id"]
  },
  "metadata": {
    "x-request-id": $HDR("X-Request-Id")
  }
});
```

## 返回结构

### 成功

```dol
{
  "ok": true,
  "status": 0,
  "status_name": "OK",
  "body": {
    "message": "pong"
  },
  "metadata": {
    "x-grpc-test": "echo-ok"
  },
  "error": null
}
```

### 远端非 OK

```dol
{
  "ok": false,
  "status": 5,
  "status_name": "NotFound",
  "body": null,
  "metadata": {
    "x-grpc-error-source": "echo-backend"
  },
  "error": {
    "code": 5,
    "name": "NotFound",
    "message": "resource not found",
    "details": []
  }
}
```

## 错误模型

分两类：

### 1. 本地错误，直接抛运行时错误

包括：

- 缺少 `target`
- 缺少 `descriptor/proto`
- descriptor 文件不存在
- descriptor 中找不到 service 或 method
- `body` 与 message schema 不匹配
- 无法连接目标地址

### 2. 远端 gRPC 非 OK，返回结构化结果

包括：

- `NOT_FOUND`
- `INVALID_ARGUMENT`
- `PERMISSION_DENIED`
- `UNAVAILABLE`
- `DEADLINE_EXCEEDED`

返回值里的 `metadata` 字段承载远端响应 metadata；当远端返回非 `OK` 时，这里承载的是 status metadata。

## 目标地址格式

当前支持：

- `http://127.0.0.1:50051`
- `https://service.example.com:443`
- `dns:///user-service:50051`

其中 `dns:///host:port` 会被当前实现归一化为普通 HTTP/2 目标地址。

## Descriptor 输入

首版主路径是 protobuf descriptor set 文件，例如：

```bash
protoc \
  --proto_path=proto \
  --descriptor_set_out=descriptors/user_service.pb \
  --include_imports \
  proto/user_service.proto
```

然后在 Dolang 里引用：

```dol
$ client = grpc.client({
  "target": "dns:///user-service:50051",
  "descriptor": "descriptors/user_service.pb"
});
```

## 当前已实现的映射范围

当前已接通的基础映射包括：

- `bool`
- `int32/int64`
- `uint32/uint64`
- `float/double`
- `string`
- `bytes`
- `enum`
- `repeated`
- `map<K,V>`
- 嵌套 message
- `optional` / presence 的基础语义

仍需继续补强的部分：

- 更完整的 optional/presence 边界行为
- 更多 map key/value 组合与类型覆盖

当前明确未支持：

- `oneof`
- `Any`
- 复杂 well-known types
- streaming

其中 `oneof` 当前会在 Dolang 本地映射阶段直接返回显式错误，不会静默降级。
`google.protobuf.Any`、`Timestamp`、`Duration`、`Struct`、`Value`、`ListValue`、`FieldMask` 等复杂 well-known types 当前也会直接返回显式错误。

## 推荐用法

推荐把 Dolang 放在 HTTP 入站和 gRPC 后端之间：

1. HTTP handler 接收请求
2. Dolang 做参数整理和基础聚合
3. Dolang 使用 `std.grpc` 调用后端服务
4. 把 gRPC 结果重新映射成 HTTP JSON

这比在 Dolang 里直接承载复杂业务逻辑，更符合当前 `std.grpc` 的设计定位。
