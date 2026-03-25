# Issue 009：math — 高级函数补齐（反三角 / 对数 / 随机数）

**类型**: Feature
**所属 Epic**: E01
**优先级**: P1
**状态**: Closed ✅

## 背景

`std.math` 目前有 sqrt/pow/abs/floor/ceil/round/min/max/clamp/pi/sin/cos，
但缺少反三角函数、对数/指数函数和随机数，这些无法用纯 .dol 实现。

## 需要新增的 native 函数

### 反三角函数

| 函数 | 描述 |
|------|------|
| `math.tan(x)` | 正切（目前在 `stdlib/math/trig.dol` 用 sin/cos 模拟，精度和性能差） |
| `math.asin(x)` | 反正弦，返回弧度 |
| `math.acos(x)` | 反余弦，返回弧度 |
| `math.atan(x)` | 反正切，返回弧度 |
| `math.atan2(y, x)` | 四象限反正切 |

### 对数 / 指数

| 函数 | 描述 |
|------|------|
| `math.exp(x)` | e^x |
| `math.ln(x)` | 自然对数 |
| `math.log(x, base)` | 任意底对数 |
| `math.log2(x)` | 以 2 为底的对数 |
| `math.log10(x)` | 以 10 为底的对数 |

### 随机数

| 函数 | 描述 |
|------|------|
| `math.random() -> Float` | 返回 [0.0, 1.0) 随机浮点数 |
| `math.random_int(min, max) -> Int` | 返回 [min, max] 随机整数 |

### 其他

| 函数 | 描述 |
|------|------|
| `math.hypot(x, y)` | 直角三角形斜边（`sqrt(x²+y²)`） |
| `math.e()` | 自然常数 e |

## 涉及文件

- `crates/dolang-runtime/src/stdlib_native/math.rs`
- 随机数需要在 `Cargo.toml` 中引入 `rand` crate（如未引入）

## 修复后同步

- `stdlib/math/trig.dol` 中 `tan()` 可直接调用 `math.tan(x)`，删除 sin/cos 模拟实现

## 验收标准

```dolang
$mod std.math;
$>> math.asin(1.0);   // 约 1.5707 (π/2)
$>> math.log(100.0, 10.0);  // 2.0
$>> math.random();    // [0.0, 1.0)
$>> math.random_int(1, 6);  // 骰子
```
