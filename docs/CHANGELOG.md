# Changelog

本文件记录 Dolang 的用户可见变化。

记录规则：

- 只记录用户可见变化
- 每次发布都按固定栏目归档
- breaking change、弃用、移除必须写清兼容影响

## Unreleased

### Added

- 建立 `docs/spec/` 正式语言规范目录
- 增加 Phase 9 的版本、兼容性、弃用规则文档

### Changed

- 规范化语言行为文档与 `tests/spec` 的映射关系

### Deprecated

- None

### Removed

- None

### Fixed

- 对齐 spec 样例与当前 parser/runtime 真实行为

## Release Template

后续版本请按如下模板追加：

```md
## x.y.z - YYYY-MM-DD

### Added

- ...

### Changed

- ...

### Deprecated

- 当前行为：
- 替代行为：
- 起始版本：
- 计划移除版本：

### Removed

- ...

### Fixed

- ...
```
