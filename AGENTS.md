# Repository Guidelines

## 项目结构与模块组织

这是一个 Rust 2024 CLI 项目，二进制名为 `tree-cli`，库 crate 为 `tree_cli`。核心代码位于 `src/`：`main.rs` 负责命令行参数解析与程序入口，`lib.rs` 导出公共模块，`core.rs` 生成目录树，`file_iterator.rs` 处理遍历，`filter.rs` 处理 glob 过滤，`symbol.rs` 处理符号和颜色输出。集成测试在 `tests/`，性能基准在 `benches/`，示例配置在 `config/config.toml`，CI 配置在 `.github/workflows/ci.yml`。

## 构建、测试与开发命令

- `cargo build`：构建调试版本。
- `cargo build --release`：构建优化后的发布版本，使用 `Cargo.toml` 中的 release profile。
- `cargo run -- -L 2 -C .`：本地运行 CLI，并传入示例参数。
- `cargo test`：运行单元测试和集成测试。
- `cargo bench`：运行 Criterion 性能基准。
- `cargo clippy --all-targets --all-features -- -D warnings`：按 CI 标准运行 lint。
- `cargo fmt --all -- --check`：检查格式；开发时可用 `cargo fmt` 自动格式化。

## 编码风格与命名约定

遵循 `rustfmt.toml`：4 空格缩进、最大行宽 120、自动重排 import/module，edition 为 2024。Rust 文件、模块和函数使用 `snake_case`，类型使用 `PascalCase`，常量使用 `SCREAMING_SNAKE_CASE`。公开 API 和复杂逻辑应保留简短中文说明，避免无意义注释。新增命令行参数时，优先放在 `main.rs` 的 clap 定义中，并在 README 与测试中同步体现。

## 测试规范

测试文件按功能拆分，例如 `tests/test_exclude.rs`、`tests/test_color_and_options.rs`。测试函数使用 `test_*` 命名，并用 `tempfile` 创建临时目录，避免依赖开发者本机路径。CLI 行为测试使用 `assert_cmd::cargo::cargo_bin!("tree-cli")`。涉及输出格式、颜色、过滤、深度限制或错误处理的改动必须补充集成测试；涉及性能路径的改动应运行相关 `benches/`。

## 提交与 Pull Request 规范

历史提交多使用中文说明，并常见 `chore(deps): ...`、`dep: ...` 等 Conventional Commits 风格，可继续采用 `type(scope): 摘要`，例如 `fix(filter): 修复排除目录匹配`。PR 应包含变更摘要、测试命令及结果、关联 issue；若 CLI 输出发生变化，请附前后命令示例或终端输出。提交前至少运行 `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings` 和 `cargo test`。

## 安全与配置提示

不要提交 `target/`、本机 IDE 状态或包含绝对路径的临时文件。修改依赖时同时提交 `Cargo.toml` 与 `Cargo.lock`，并留意 CI 的 `security_audit`。
