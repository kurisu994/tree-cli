# 更新日志

本项目所有重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
并且本项目遵循[语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 安全

- 过滤文件名中的控制字符（C0、DEL、C1，含 ESC），防止恶意文件名借 ANSI/终端转义序列在终端上伪造或隐藏 `tree-cli` 的输出。新增 `symbol::sanitize_file_name`，对不含控制字符的正常文件名零额外分配。

### 变更

- 终端输出从 `term` crate 迁移到 `anstyle` + `anstream`。
- 颜色决策统一交由 `anstream::AutoStream` 按 `ColorChoice` 处理（综合是否 TTY 以及 `NO_COLOR`/`CLICOLOR` 等环境变量），移除内部 `Config.colorful` 字段与手写的 TTY 判断逻辑。
- 树形前缀、文件名与颜色统一写入同一输出流；`DirTree` 与 `print_path` 泛型化为 `W: Write`。

### 移除

- 移除 `term` 依赖及其传递依赖 `dirs-next`、`dirs-sys-next`（迁移目标 `anstyle`/`anstream` 已由 `clap` 引入，无新增 crate）。

## [0.1.0] - 2025-12-10

### 新增

- 以树形结构显示目录内容，并统计目录/文件数量。
- 彩色输出：蓝色目录、红色可执行文件（Unix）。
- glob 模式过滤：`-P`/`--pattern` 包含、`-E`/`--exclude` 排除。
- `-L`/`--level` 限制遍历深度。
- `-a`/`--all` 显示隐藏文件。
- `-s`/`--human-readable` 以人类可读格式显示文件大小（支持到 EB 级别）。
- `-C`/`--color` 与 `-N`/`--no-color` 强制开关彩色输出。
- 跨平台支持：Linux、macOS、Windows。
- 基于 Criterion 的性能基准测试与回归检测。

[未发布]: https://github.com/kurisu994/tree-cli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kurisu994/tree-cli/releases/tag/v0.1.0
