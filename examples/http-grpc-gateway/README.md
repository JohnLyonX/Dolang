# HTTP gRPC Gateway Example

这个示例展示 Dolang 作为 HTTP API 集成层时，如何通过 `std.grpc` 调用后端 gRPC 服务。

## 场景

- Dolang 接收 HTTP 请求
- Dolang 整理请求参数
- Dolang 调用远端 unary gRPC 方法
- Dolang 返回结构化 JSON 结果

## 入口

- `main.dol`

## 前置条件

示例中的后端服务和 descriptor 文件需要由你自己提供：

- gRPC 服务地址：`dns:///user-service:50051`
- descriptor 文件：`descriptors/user_service.pb`

可以先用 `protoc` 生成 descriptor：

```bash
protoc \
  --proto_path=proto \
  --descriptor_set_out=descriptors/user_service.pb \
  --include_imports \
  proto/user_service.proto
```

## 运行方式

```bash
cargo run -- serve examples/http-grpc-gateway
```

示例假定你已经把真实 gRPC 地址和 descriptor 文件路径改成自己的工程配置。

## 预期行为

- 启动 Dolang HTTP 服务
- 对 `/api/users/detail` 发起 HTTP 请求
- handler 内部通过 `std.grpc` 调用远端 unary 方法
- 返回结构化 JSON 结果，保留 `ok`、`status`、`body`、`metadata`、`error`

如果你只想复用代码结构，也可以把示例内容合并进自己的项目 `main.dol`，再通过 `serve` 模式运行。
