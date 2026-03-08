# Changelog

All notable changes to DaoLang will be documented in this file.

## [v1.7.0] - 2026-03-08

### Added
- **可变参数函数**：
  - 具名函数：`$fn add(...nums) { $# nums; }`
  - 匿名函数：`$ result = $fn(...args) { $# args; };`
  - 混合参数：`$fn greet(name, ...others) { $# [name, others]; }`
  - 使用 `.len()` 获取参数个数
- **文件读写**（拆分写法）：
  - `$>>FILE(path)` - 创建文件对象
  - `.content("内容")` - 链式写入内容
  - `$>>FILE(path, "W")` - 覆盖写入模式
  - `$>>FILE(path, "A")` - 追加写入模式
  - `$>>FILE(path, "DEL")` - 删除文件
  - `$<<FILE(path)` - 读取文件（返回 File 对象）
  - File 方法：`.exists()`、`.read()`、`.read_lines()`、`.size()`、`.is_dir()`
- **转义字符支持**：
  - 字符串：`\n`, `\t`, `\r`, `\0`, `\\`, `\"`
  - f-string 同样支持
- **项目系统**：
  - `dolang serve [path]` - 服务模式
  - `$mod 路径;` - 模块导入
  - `$main() { }` - 主入口声明
  - `$<<CONFIG("KEY")` - 配置读取（仅服务模式）
  - `package.toml` - 项目配置文件

## [v1.6] - 2026-03-02

### Added
- **标准输入**：
  - `$<<ENV("KEY")` - 读取环境变量
  - `$<<LINE()` - 读取 stdin 一行
  - `$<<LINE("提示")` - 读取 stdin（带提示）
- **方法调用必须使用括号**：如 `obj.method()`，不再支持 `obj.method`
- 错误信息改进：方法调用缺少括号时会给出明确的错误提示

### Fixed
- 修复：`$<<ENV()` 在未赋值情况下会自动打印值的 bug
- 修复：方法调用缺少括号时报错信息不准确的问题

## [v1.3] - 2026-03-01

### Added
- **渐进式类型系统**：支持类型注解（`$ x: Int = 30`）
- 动态模式（默认）：变量可随时改变类型
- 静态模式：使用类型注解后，类型检查生效
- 支持的类型注解：`Int`/`Integer`、`Float`、`String`/`Str`、`Bool`/`Boolean`
- 常量类型注解支持

### Changed
- **Breaking Change**：动态模式下重新赋值可以改变类型（之前会报错）
- 类型错误信息格式更新，更清晰易读

### Fixed
- 修复 Bug #4：`to_str()` 等方法返回值可直接赋值给原变量（动态模式下）

## [v1.2] - 2026-02-28

### Added
- 代码解耦、模块化重构
- 复合赋值运算符 (`+=`, `-=`, `*=`, `/=`, `%=`)
- 列表类型 (List)
- 字典类型 (Map)
- 方法调用语法 (`obj.method()`)
- for-in 遍历语法
- 字符串内置方法 (`len`, `upper`, `lower`, `contains`, `replace`, `trim`, `split`, etc.)
- 列表内置方法 (`push`, `pop`, `reverse`, `len`, `contains`, `join`)
- 字典内置方法 (`keys`, `values`, `len`, `contains_key`, `remove`)

### Changed
- 代码解耦：拆分 parser、interpreter、main 模块，提升代码可维护性

## [v1.1] - 2026-02-27

### Added
- 函数作为值（匿名函数赋值给变量）
- 改进解析错误消息显示

## [v1.0] - 2026-02-27

### Added
- 函数定义 (`$fn`)
- 返回语句 (`$#`)
- 函数返回类型检查
- 未定义变量检查

## [v0.9] - 2026-02-27

### Added
- 循环语句 (`$while`, `$loop`, `$for`)

## [v0.8] - 2026-02-27

### Added
- 条件语句 (`$if`, `$elif`, `$else`)

## [v0.7] - 2026-02-27

### Added
- 布尔类型
- 比较运算 (`==`, `!=`, `>`, `<`, `>=`, `<=`)
- 逻辑运算 (`&&`, `||`, `!`)

## [v0.6] - 2026-02-26

### Added
- 变量遮蔽 (Shadowing)
- 类型系统

## [v0.5] - 2026-02-26

### Added
- 重构语法，采用 `$` 符号化语法

## [v0.1] - 早期版本

### Added
- 原始版本（`..$` 语法）
