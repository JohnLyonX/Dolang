# DOC-010: 第 18 章（模式与配方）是占位骨架，缺少完整示例

**优先级：** P2
**影响章节：** `docs/guide/18-patterns-and-recipes.md`

## 问题描述

第 18 章本身已说明"后续继续扩写"，目前每个配方只有 3-10 行示例代码，没有完整可运行的 demo。

已列出但未实质填充的配方：
- 单文件脚本（仅 3 行示例）
- 标准库驱动小工具（仅 5 行示例）
- 带 package 小项目（仅目录结构，无实际代码）
- 拆模块 HTTP 服务（仅 4 行示例）
- 从脚本迁移到服务（每步只有标题，无代码演示）

完全缺失的高频场景：
- JSON 处理脚本（从文件读 JSON，处理后写回）
- 调用外部 API 后处理结果（`std.http` 客户端实战）
- 文件批处理（遍历目录、读取、转换）
- 错误优雅处理（try/catch 在实际服务里的用法）
- 多人协作的模块目录组织

## 期望改进

为每个配方提供**完整可粘贴运行**的示例，包含：
- 文件结构
- 完整代码
- 运行命令
- 预期输出

优先补充以下两个最高频场景：

### 配方：调用外部 API
```dol
$mod std.http;
$mod std.json;

$GET("/weather/:city") get_weather(city) -> JSON {
    $ resp = http.get("https://api.example.com/weather?q=" + city);
    $if resp["status"] != 200 {
        $# $RES(502, { "error": "upstream failed" });
    }
    $ data = json.parse(resp["body"]);
    $# $JSON { "city": city, "temp": data["temp"] };
}
```

### 配方：读取并处理 JSON 文件
```dol
$mod std.fs;
$mod std.json;

$ raw = fs.read_text("data.json");
$ records = json.parse(raw);
$for item in records {
    $>> item["name"] + ": " + item["score"];
}
```
