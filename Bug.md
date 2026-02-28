# DaoLang 解释器代码审查报告

## 一、Lexer 问题

### 1. [high] src/lexer/lexer.rs:357-359
- **问题**: `peek()` 方法在 `pos >= input.len()` 时会 panic
- **风险**: 未闭合字符串或注释后继续 tokenization 时可能导致 panic
- **建议修复**:
```rust
fn peek(&self) -> char {
    self.input.get(self.pos).copied().unwrap_or('\0')
}
```

### 2. [medium] src/lexer/lexer.rs:244-246
- **问题**: `read_string` 检测到未闭合字符串后仍然返回空字符串 token，而不是报错
- **风险**: 静默接受无效输入，后续解析可能产生难以追踪的错误
- **建议修复**:
```rust
fn read_string(&mut self, start: usize) -> Result<Token, String> {
    self.advance(); // consume opening "
    let body = self.read_until_rune('"');
    if body.is_empty() {
        if self.pos >= self.input.len() {
            return Err("unclosed string literal".to_string());
        }
    }
    // 检查是否到达文件末尾而未找到闭合引号
    if self.pos >= self.input.len() {
        return Err("unclosed string literal".to_string());
    }
    self.advance(); // consume closing "
    let literal: String = body.iter().collect();
    Ok(Token::new(Type::String, &literal, start))
}
```

### 3. [high] src/lexer/lexer.rs:329-335
- **问题**: `skip_whitespace` 跳过 `\` 字符，这本意是处理 shell 转义，但会破坏字符串中反斜杠的处理
- **风险**: 用户无法在字符串中使用反斜杠
- **建议**: 这个设计需要重新考虑，或者仅在 REPL 模式下启用

### 4. [medium] src/lexer/lexer.rs:358
- **问题**: Token 的 `pos` 字段存储的是字节偏移，但 `calc_line_col` 使用字符索引
- **风险**: 对于多字节字符（如中文），行列号计算可能不准确
- **建议**: Token 位置应使用字符索引，或者确保 lexer 存储字符位置

---

## 二、Parser 问题

### 5. [medium] src/parser/expr.rs:15-16, 32-34
- **问题**: 当解析表达式 token 列表为空时，硬编码返回 `line: 1, column: 1`
- **风险**: 错误位置信息不准确
- **建议**: 使用表达式被嵌入的上下文位置

### 6. [medium] src/parser/stmt.rs:52-56
- **问题**: `expect()` 方法失败时返回 `Error::InvalidStatement`，没有包含期望的 token 类型信息
- **风险**: 用户看不到期望的具体 token，错误体验差
- **建议修复**:
```rust
pub fn expect(&mut self, typ: Type) -> Result<&Token, Error> {
    if self.at_end() || self.peek().typ != typ {
        return Err(Error::Parse(ParseError {
            message: format!("expected {:?} but got {:?}", typ, self.peek().typ),
            line: /* get line */,
            column: /* get column */,
            found: Some(format!("{:?}", self.peek().typ)),
            expected: Some(format!("{:?}", typ)),
        }));
    }
    Ok(self.advance())
}
```

### 7. [medium] src/parser/stmt.rs:248-269
- **问题**: 赋值语句解析时，先解析 name 为表达式再解析值，但 `AssignStmt.name` 是 `Box<Expr>`，允许复杂的左侧表达式
- **风险**: 不符合常规赋值语法（左侧应为简单变量名）
- **建议**: 限制左侧为标识符

---

## 三、AST 问题

### 8. [low] src/ast/ast.rs
- **问题**: AST 节点没有 span 信息，无法精确定位语法错误
- **影响**: 错误信息只能显示 token 位置，无法显示 AST 节点对应的源码范围
- **建议**: 为每个 AST 节点添加 `span: (usize, usize)` 字段

---

## 四、Interpreter 问题

### 9. [high] src/interpreter/eval.rs:71-78
- **问题**: `check_eval_result` 使用特殊字符串 `"__DIVZERO__"` 和 `"__MODZERO__"` 来标记除零错误
- **风险**:
  1. 这是一个 hacky 的实现方式
  2. 如果用户数据中恰好包含这些字符串，会产生误报
- **建议**: 使用 `Result<String, Error>` 替代 `Option<String>`

### 10. [high] src/interpreter/eval.rs:93-101
- **问题**: `VarLookup` 返回 `None` 表示变量未定义，但 `None` 也可能表示其他求值失败
- **风险**: 错误诊断困难
- **建议**: 统一使用 `Result` 类型

### 11. [high] src/interpreter/exec.rs:357-361
- **问题**: `$break` 和 `$continue` 在循环外使用时的检查发生在函数调用返回后，而不是立即检查
- **风险**: 在函数内部的循环中 break，然后函数返回后仍然会被误报为 "break/continue used outside of loop"
- **示例**:
```dol
$fn test() {
    $loop {
        $break;
    }
}
$fn main() {
    test();  // 这会报错：break used outside of loop
}
```

### 12. [high] src/interpreter/exec.rs:140-153
- **问题**: `$#` (return) 在函数外使用时没有明确报错，而是被当作普通语句处理
- **风险**: 语义不明确
- **建议**: 在顶层检测 `$#` 并返回明确错误

### 13. [medium] src/interpreter/env.rs:49-51
- **问题**: `to_bool` 函数将非 "true" 的所有值都视为 false
- **风险**: 与类型系统不一致（"false" 字符串会被转换为 bool false，但其他字符串也返回 false）
- **建议**:
```rust
pub fn to_bool(value: &str) -> bool {
    value == "true"
}
```
但需要在所有比较操作中确保操作数类型一致。

### 14. [low] src/interpreter/eval.rs:157-158
- **问题**: `Eq` 和 `Ne` 使用字符串比较而非语义比较
- **风险**: `1 == 1.0` 返回 false（因为 "1" != "1.0"）
- **建议**: 对于数值比较，先尝试解析为数字再比较

---

## 五、作用域/环境问题

### 15. [medium] src/interpreter/exec.rs:119-136
- **问题**: 变量赋值时如果变量不存在，会创建新变量（隐式声明）
- **风险**: 容易产生拼写错误导致的隐式变量
- **建议**: 赋值前检查变量是否已声明

### 16. [low] src/interpreter/exec.rs:226-234
- **问题**: 函数重定义时只检查 `fns.contains_key()`，但没有检查变量遮蔽
- **风险**: 函数可能被同名变量遮蔽

---

## 六、REPL 问题

### 17. [medium] src/repl.rs:239-256
- **问题**: 多行输入只检测大括号平衡，不检测圆括号/方括号
- **风险**: 用户可能遗漏闭合括号
- **建议**: 检测所有成对符号
```rust
let depth: i64 = buf.chars().fold(0, |acc, c| match c {
    '{' | '(' | '[' => acc + 1,
    '}' | ')' | ']' => acc - 1,
    _ => acc,
});
```

### 18. [medium] src/repl.rs:270-274
- **问题**: 强制要求语句以 `;` 结尾，但对块语句要求以 `}` 结尾
- **风险**: 不一致的行为让用户困惑
- **建议**: 统一处理

### 19. [low] src/repl.rs:176-182
- **问题**: Ctrl+C 返回空字符串，当前行为是开始新行
- **风险**: 用户可能误以为输入被取消
- **建议**: 显示明确的提示（如 "^C"）

---

## 七、错误处理问题

### 20. [medium] src/error/error.rs:68-73
- **问题**: `Error::InvalidStatement`, `Error::InvalidExpression`, `Error::InvalidAssignment` 是通用错误，没有具体信息
- **风险**: 用户不知道具体哪里错了
- **建议**: 这些错误应包含更多上下文信息

### 21. [medium] src/repl.rs:282-288
- **问题**: REPL 中解析错误后直接 continue，丢失了部分输入状态
- **风险**: 用户可能不知道具体哪行出错

---

## 八、跨平台问题

### 22. [medium] src/repl.rs:7-10
- **问题**: 使用 `crossterm` 库，但依赖 `atty` 判断是否在终端
- **风险**: Windows 命令行支持可能不完整
- **建议**: 全面测试 Windows 兼容性

### 23. [medium] src/repl.rs:315-360
- **问题**: 文件执行前的手动语法验证与解析器的验证重复，且实现略有不同
- **风险**: 可能出现一个通过另一个失败的情况
- **建议**: 移除重复验证，或统一使用解析器结果

---

## 九、性能问题

### 24. [low] src/lexer/lexer.rs:22
- **问题**: `toks.push(tok.clone())` - clone 了 token
- **风险**: 轻微的性能开销
- **建议**: 改为 `toks.push(tok)` 或修改 `next_token` 返回引用

### 25. [low] src/interpreter/eval.rs:89-92
- **问题**: 所有字面量都 clone 了字符串值
- **风险**: 轻微的内存开销
- **建议**: 考虑使用 `Cow<str>` 或引用

---

## 十、测试覆盖问题

### 26. [high] 缺少测试目录
- **问题**: 没有 tests/ 目录
- **风险**: 无法进行回归测试
- **建议**: 添加针对每个语法结构的测试用例

---

## 总结

| 严重程度 | 数量 |
|---------|------|
| critical | 0 |
| high | 9 |
| medium | 14 |
| low | 6 |

**最需要修复的问题**:
1. Lexer 中 `peek()` 的 panic 问题
2. 使用特殊字符串标记错误（divzero/modzero）
3. `$break`/`$continue` 在函数内循环的检查问题
4. 变量隐式声明问题
5. 缺少测试覆盖
