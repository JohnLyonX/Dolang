# DOC-013: 第 3 章（第一个程序）缺少验证步骤

**优先级：** P3
**影响章节：** `docs/guide/03-first-program.md`

## 问题描述

第 3 章给出了三个最短示例（脚本、REPL、HTTP），但对于 HTTP 示例，文档只展示了如何写和启动，没有说明如何**验证它跑起来了**：

- `dolang serve main.dol` 启动后默认监听哪个端口？
- 如何用 curl 验证路由正常响应？
- 浏览器访问什么地址？
- 终端会打印什么信息？

对于第一次接触 dolang 的用户，这个验证环节至关重要——跑通第一个接口是建立信心的关键步骤。

## 期望改进

在第 3 章 HTTP 示例后补充：

```bash
# 启动服务
dolang serve main.dol

# 终端会输出：
# [INFO] HTTP route registered: GET /hello
# [INFO] Serving on http://0.0.0.0:3000
```

```bash
# 另一个终端验证
curl http://localhost:3000/hello
# 输出：{"message":"Hello, Dolang!"}
```

同时在第一个脚本示例后补充预期输出：

```dol
$>> "Hello, Dolang!";
```
```
# 运行：dolang run hello.dol
# 输出：Hello, Dolang!
```
