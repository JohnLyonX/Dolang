# Issue 008：fs — 目录操作 native 补齐

**类型**: Feature
**所属 Epic**: E01
**优先级**: P0
**状态**: Closed ✅

## 背景

`std.fs` 目前只有基础文件读写（read_text/write_text/append_text/delete/exists/size/is_dir），
没有任何目录操作，导致无法用 Dolang 做文件树遍历、构建工具等场景。

## 需要新增的 native 函数

| 函数签名 | 描述 |
|---------|------|
| `fs.list(path) -> List` | 列出目录下的文件和子目录名（不递归） |
| `fs.list_all(path) -> List` | 递归列出所有文件路径 |
| `fs.mkdir(path)` | 创建目录（父目录必须存在） |
| `fs.mkdir_all(path)` | 递归创建目录（类似 mkdir -p） |
| `fs.rmdir(path)` | 删除空目录 |
| `fs.remove_all(path)` | 递归删除目录及其内容（危险操作） |
| `fs.copy(src, dst)` | 复制文件 |
| `fs.rename(src, dst)` | 移动/重命名文件或目录 |

## 涉及文件

- `crates/dolang-runtime/src/stdlib_native/fs.rs`：新增上述函数
- `crates/dolang-runtime/src/runtime/intrinsics.rs`：注册对应 intrinsic（如果 fs 走 intrinsic 路径）

## 验收标准

```dolang
$mod std.fs;

$ files = fs.list("/tmp");
$>> files;

fs.mkdir_all("/tmp/dolang/test");
$>> fs.exists("/tmp/dolang/test");
```

运行无报错，输出正确。

## 备注

`remove_all` 属于危险操作，实现时需考虑是否在 Dolang 中暴露，
或要求显式传入 `force: Bool` 参数。
