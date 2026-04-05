# Dolang gRPC Client 设计

## 1. 背景与目标

Dolang 当前的核心定位，是快速生成轻量、快速的 HTTP 服务。用户既可以直接在 Dolang 中编写业务逻辑，也可以把 Dolang 作为 HTTP API 集成网关，把核心业务下沉到 Go 或其他语言实现。

这决定了 Dolang 对 gRPC 的需求，不是“完整实现一套 gRPC 框架”，而是：

`提供一个适合 HTTP API 编排、BFF、集成网关场景的 gRPC Client 能力`

该能力的主要用途包括：

- Dolang `serve` 入口接收 HTTP 请求
- Dolang 在 handler 中聚合、转换或校验请求数据
- Dolang 调用后端 Go/Java/Node 等服务暴露的 gRPC 接口
- Dolang 将 gRPC 结果重新映射成 HTTP JSON/HTML/String 响应

因此，本设计不追求把 Dolang 变成一个完整 RPC 平台，而是先把它做成一个可靠的 gRPC 调用层。

## 2. 首版定位

首版能力定义为：

`一个契约驱动、面向网关场景的 unary gRPC client`

其中“契约驱动”表示：

- Dolang 侧请求与响应继续使用 `Map/JSON`
- gRPC service、method、request、response 的结构由 proto 契约决定
- 运行时在调用前后依据契约完成校验、编码与解码

其中“unary”表示：

- 首版只支持单次请求、单次响应
- 不支持 streaming

这个范围对常见增删改查、查询聚合、权限校验、资料拼装等业务场景是足够的，也与 Dolang 当前的 HTTP/BFF 定位一致。

## 3. 设计原则

本次设计遵循以下原则：

1. Dolang 仍然以 HTTP 服务与 API 集成为主，不把 gRPC 做成新的语言中心能力。
2. 首版优先支持最常见的业务调用路径，不为少量高级场景引入过高实现成本。
3. Dolang 用户继续操作 `Map/JSON`，不要求生成代码，不强迫用户理解复杂 proto 运行时细节。
4. 运行时必须保留标准 gRPC 的关键边界，包括 `target`、`service`、`method`、`metadata`、`timeout/deadline`、`status`。
5. 远端业务失败不等同于 Dolang 运行时错误，错误模型必须适合网关场景。
6. 首版以标准库模块形式接入，不新增 gRPC 专属语言级语法。

## 4. 首版范围

### 4.1 In Scope

- 新增 `std.grpc` 标准库模块
- 支持对象式客户端创建：`grpc.client({...})`
- 支持 unary 调用：`client.call({...})`
- Dolang 侧请求与响应统一使用 `Map/JSON`
- 支持通过 descriptor 主导契约加载
- 支持 `.proto` 作为兼容入口
- 支持 metadata
- 支持 timeout / deadline 的基础能力
- 支持基础 TLS 配置
- 支持结构化调用结果返回
- 支持将远端 gRPC 非 OK 状态以结构化错误暴露给业务层

### 4.2 Out of Scope

- server streaming
- client streaming
- bidirectional streaming
- 代码生成
- Dolang 原生 proto 类型系统
- 新的语言级 RPC DSL，例如 `$GRPC { ... }`
- 完整服务发现、负载均衡、熔断、重试治理
- 深度耦合的网关声明式路由编排语法

## 5. 用户使用模型

首版采用对象式 API：

- `grpc.client({...})` 负责创建客户端
- `client.call({...})` 负责执行一次 unary RPC

### 5.1 推荐调用形态

```dol
$ grpc = import("std.grpc");

$ user_svc = grpc.client({
  "target": "dns:///user-service:50051",
  "descriptor": "descriptors/user_service.pb",
  "timeout_ms": 1500,
  "metadata": {
    "x-client": "dolang-gateway"
  }
});

$POST("/api/users/detail") user_detail() -> JSON {
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

  $# result;
}
```

### 5.2 为什么选择对象式

对象式而不是全局函数式 API，原因如下：

- `client` 可以承载连接级配置，避免每次调用都重复传入
- 请求级参数与连接级参数边界清晰
- 后续可自然扩展连接复用、默认 metadata、观测、拦截器等能力
- 符合 Dolang 现有通过标准库模块提供运行时能力的风格

## 6. 契约输入模型

### 6.1 支持两种输入

客户端初始化时，允许使用两种契约来源：

- `descriptor`
- `proto`

### 6.2 主路径与兼容路径

首版主推 `descriptor`，`.proto` 仅作为兼容入口。

原因：

- `.proto` 需要处理 import、依赖路径、well-known types、工程目录结构等问题
- descriptor 是已编译完成的契约结果，更适合运行时消费
- descriptor 更稳定，部署链路也更清晰

因此建议产品口径为：

- 开发态可接受 `.proto`
- 生产部署与正式接入主推 descriptor

### 6.3 约束

- `grpc.client({...})` 必须提供 `descriptor` 或 `proto` 二选一
- 如果同时提供，运行时需要定义优先级并检查冲突
- 文档与样例以 descriptor 为主

## 7. 消息映射模型

### 7.1 核心策略

`Dolang 内部继续使用 Map/JSON，gRPC 编码解码由契约驱动完成`

执行流程如下：

1. 通过 `service + method` 查找 method descriptor
2. 根据 input message schema 校验 Dolang `body`
3. 将 Dolang `Map/JSON` 编码为 protobuf message
4. 发起 gRPC unary 请求
5. 根据 output message schema 将 protobuf response 解码为 Dolang `Map/JSON`

### 7.2 首版建议支持的类型

- `string`
- `bool`
- `int32`
- `int64`
- `uint32`
- `uint64`
- `float`
- `double`
- `bytes`
- `repeated`
- `map<K,V>`
- 嵌套 message
- `enum`
- `optional` / presence 的基础语义

### 7.3 首版限制

首版不追求一次性覆盖 protobuf 的全部高级特性。以下能力建议暂不完整支持，或仅做受限支持：

- `oneof`
- `Any`
- 复杂 well-known types 的高级语义映射
- 自定义 option
- 细粒度 proto2/proto3 行为差异

遇到未支持特性时，运行时必须给出清晰错误，而不是静默降级。

## 8. 错误模型

首版错误分为两类：

- 本地开发期错误
- 远端调用结果错误

### 8.1 本地开发期错误

以下错误应直接抛出 Dolang 运行时错误：

- `grpc.client({...})` 配置不合法
- descriptor / proto 无法加载
- `service` 或 `method` 不存在
- 请求 `body` 与 schema 不匹配
- 本地编码失败
- 本地解码流程发生实现级异常

原因是这类错误说明程序、配置或契约本身存在问题，不应伪装成业务结果。

### 8.2 远端调用结果错误

以下情况不应直接抛出异常，而应返回结构化结果：

- 远端返回 `NOT_FOUND`
- 远端返回 `INVALID_ARGUMENT`
- 远端返回 `PERMISSION_DENIED`
- 远端返回 `UNAVAILABLE`
- 远端返回 `DEADLINE_EXCEEDED`
- 其他 gRPC 非 OK 状态

这是因为这些错误对网关层来说，是一次“已完成的远程调用结果”，不是 Dolang 运行时崩溃。

### 8.3 建议返回结构

```dol
{
  "ok": true,
  "status": 0,
  "status_name": "OK",
  "body": {
    "id": "u_123",
    "name": "Tom"
  },
  "metadata": {
    "x-request-id": "abc-123"
  },
  "error": null
}
```

失败时：

```dol
{
  "ok": false,
  "status": 5,
  "status_name": "NOT_FOUND",
  "body": null,
  "metadata": {},
  "error": {
    "code": 5,
    "name": "NOT_FOUND",
    "message": "user not found",
    "details": []
  }
}
```

### 8.4 设计结论

- 本地配置、契约、编码错误直接抛异常
- 远端 gRPC 非 OK 结果结构化返回

这是最适合 HTTP/BFF/集成网关场景的模型。

## 9. 与现有 HTTP 栈的关系

本设计不改变 Dolang 当前的主定位：

- `serve` 负责入站 HTTP
- `std.http` 负责通用 HTTP 出站
- `std.grpc` 负责契约驱动的 gRPC unary 出站

推荐的系统角色分工为：

1. HTTP 请求由 `serve` handler 接收
2. handler 提取 path/query/header/body
3. handler 组装成 `client.call({...})` 需要的 `body`
4. gRPC 返回结构化结果
5. handler 再决定将其映射成 HTTP JSON、HTML 或 String

因此，gRPC 在 Dolang 中应被视为“出站集成能力”，而不是“新的服务定义主线”。

## 10. 为什么不做语言级 DSL

首版明确不新增类似 `$GRPC { ... }` 的语法。

原因：

- 会扩大 parser、AST、diagnostics 的改动面
- 与当前需求不匹配，当前需求是客户端调用能力，不是声明式 gRPC 服务端框架
- 标准库模块更容易验证价值，也更容易调整 API
- 如果未来需要更高级的声明式集成能力，可以建立在 `std.grpc` 之上继续演进

这一决策有利于控制实现风险。

## 11. 首版内部架构拆分

为了避免把实现塞进单一模块，建议内部拆成以下 4 层：

### 11.1 Client Config

负责：

- 解析 `target`
- 解析 `descriptor` / `proto`
- 解析默认 `metadata`
- 解析 `timeout_ms`
- 解析 TLS 相关配置

该层只负责客户端初始化，不关心单次 method 调用。

### 11.2 Contract Loader

负责：

- 加载 descriptor 或 `.proto`
- 建立 service / method / message 查询索引
- 向上提供 method descriptor 查询接口

这一层是契约中枢，不应与传输层耦合。

### 11.3 Message Mapper

负责：

- Dolang `Map/JSON` 到 protobuf message 的转换
- protobuf response 到 Dolang `Map/JSON` 的转换
- 字段校验
- 基础类型转换
- 枚举映射
- 嵌套结构处理

这一层的重点是“契约驱动的数据转换”。

### 11.4 Transport Executor

负责：

- 发起 unary gRPC 调用
- 写入 metadata
- 控制 timeout / deadline
- 收集响应 metadata
- 收集 status / error

这一层只关心传输，不应承担复杂的 Dolang 业务结构理解。

## 12. API 草案

### 12.1 Client 创建

```dol
$ grpc = import("std.grpc");

$ client = grpc.client({
  "target": "dns:///inventory-service:50051",
  "descriptor": "descriptors/inventory.pb",
  "timeout_ms": 1200,
  "tls": {
    "enabled": true
  },
  "metadata": {
    "x-client": "dolang"
  }
});
```

建议支持字段：

- `target`
- `descriptor`
- `proto`
- `timeout_ms`
- `metadata`
- `tls`

### 12.2 Unary 调用

```dol
$ result = client.call({
  "service": "inventory.v1.InventoryService",
  "method": "GetStock",
  "body": {
    "sku": "SKU-1"
  },
  "metadata": {
    "x-request-id": "req-1"
  },
  "timeout_ms": 800
});
```

建议支持字段：

- `service`
- `method`
- `body`
- `metadata`
- `timeout_ms`

调用级配置覆盖 client 默认配置。

## 13. 首版测试策略

首版测试应覆盖以下层次：

### 13.1 Contract Loader 测试

- descriptor 加载成功
- `.proto` 加载成功
- service / method 查找成功
- service / method 不存在时报错清晰

### 13.2 Message Mapper 测试

- 基础标量类型编码解码
- 嵌套 message 编码解码
- repeated / map 编码解码
- enum 编码解码
- 缺字段、错字段、类型不匹配时错误清晰

### 13.3 Transport 测试

- unary 成功请求
- metadata 透传
- timeout 生效
- 非 OK status 返回结构化结果
- 连接失败、目标不可达的结果分类清晰

### 13.4 集成测试

- 在 `serve` handler 中调用 gRPC client
- 将 HTTP body 转为 gRPC request
- 将 gRPC response 转为 HTTP JSON response
- 验证网关场景的端到端行为

## 14. 演进路线

首版完成后，后续演进顺序建议如下：

### 14.1 Phase 2

- 更完整的 protobuf 类型支持
- 更好的 TLS 配置能力
- 更好的错误详情结构
- 更清晰的 metadata / trailer 暴露模型

### 14.2 Phase 3

- server streaming 设计
- 结果迭代器或流式消费模型
- 与 `serve` 长连接或流式 HTTP 的桥接策略

### 14.3 Phase 4

- 更高层的声明式远程服务绑定
- 网关治理能力
- 服务发现、重试、负载均衡等高级能力

在此之前，不建议首版把范围直接扩大。

## 15. 最终结论

Dolang 的 gRPC 方案，应当首先服务于它现有的产品定位：

`Dolang 作为轻量 HTTP 服务与 API 集成网关，通过 std.grpc 去调用后端 gRPC 服务。`

因此首版最合理的落点是：

- 不做完整 gRPC
- 不做服务端框架
- 不做 streaming
- 不做代码生成
- 先做一个契约驱动、对象式、unary、面向网关场景的 `std.grpc` client

这个方案在复杂度、产品价值和后续演进空间之间，取得了最稳妥的平衡。
