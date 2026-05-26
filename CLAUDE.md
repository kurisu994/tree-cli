# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

tree-cli 是一个用 Rust 编写的高性能跨平台命令行工具，用于以树形结构显示目录内容。这是 Unix `tree` 命令的轻量级替代方案，注重性能和用户体验。

## 核心架构

### 主要模块

- **main.rs**: 程序入口点，使用 clap 解析命令行参数
- **core.rs**: 核心逻辑，包含 `Config` 和 `DirTree` 结构体，负责目录树的生成和显示
- **file_iterator.rs**: 文件系统遍历实现，使用 `FileIterator` 进行广度优先遍历
- **filter.rs**: 文件过滤逻辑，支持 glob 模式匹配和空目录过滤
- **symbol.rs**: 树形结构的符号定义、ANSI 彩色输出，以及文件名控制字符过滤（防终端转义注入）

### 关键设计模式

1. **迭代器模式**: 使用自定义迭代器进行流式文件系统遍历，避免一次性加载所有文件到内存
2. **配置驱动**: 通过 `Config` 结构体集中管理所有命令行选项
3. **模块化设计**: 每个功能模块独立，便于测试和维护

## 常用开发命令

```bash
# 构建
cargo build              # 调试版本
cargo build --release    # 发布版本（启用 LTO 和优化等级 3）

# 测试
cargo test               # 运行所有测试
cargo test --lib         # 只运行单元测试
cargo test --test '*'     # 只运行集成测试
cargo test --bin tree-cli # 测试二进制
cargo test --doc          # 运行文档测试
cargo test --exact <test_name>  # 运行单个测试

# 性能基准测试
cargo bench              # 运行所有性能基准测试
cargo bench --bench simple_perf      # 运行简化基准测试
cargo bench --bench regression_simple # 运行回归基准测试
cargo bench --bench performance       # 运行完整性能测试

# 代码质量
cargo clippy            # 代码检查
cargo fmt               # 代码格式化（遵循 rustfmt.toml 配置）
cargo fmt --check        # 检查格式但不修改文件

# 安装
cargo install --path .  # 从源码本地安装
cargo install --git https://github.com/kurisu994/tree-cli.git  # 从 GitHub 安装

# 运行
./target/debug/tree-cli  # 运行调试版本
./target/release/tree-cli # 运行发布版本
```

## 代码规范

- 使用 Rust 2024 Edition
- 最大行宽：120 字符（在 rustfmt.toml 中配置）
- 使用 4 个空格缩进
- 所有公开的函数和结构体必须包含中文注释
- 复杂逻辑需要行内中文注释
- 模块级注释使用 `//!`，函数/结构体注释使用 `///`
- 错误信息优先使用中文

## 性能考虑

- 文件系统访问经过优化，单目录读取 < 30µs
- 使用 `VecDeque` 实现高效的队列操作
- 字符串操作使用 `format_args!` 宏避免额外分配
- 排序使用 `unstable_by` 提升性能

## 测试策略

项目包含全面的测试套件：

### 单元测试（30个测试）
- **core.rs**: 测试配置结构、符号切换逻辑和目录摘要
- **file_iterator.rs**: 测试文件迭代、目录遍历、隐藏文件过滤
- **filter.rs**: 测试过滤逻辑、空目录处理、缓存机制
- **symbol.rs**: 测试符号生成、颜色输出、可执行文件检测、文件名控制字符过滤

### 集成测试（22个测试）
分布在 4 个测试文件，均通过 `assert_cmd` 调用编译后的二进制进行验证：
- **tests/integration_test.rs**（8个）: 基本命令行功能（目录树显示、`-a` 隐藏文件、`-L` 深度限制、`-P` 模式过滤、帮助与版本、空目录处理）
- **tests/test_color_and_options.rs**（9个）: 颜色输出、多参数组合、错误处理、特殊字符与 Unicode、深层嵌套、符号链接处理、大目录性能、路径边界
- **tests/test_exclude.rs**（4个）: `-E` 排除模式（单一 / 多重 / 目录 / 与 include 组合）
- **tests/test.rs**（1个）: 基础冒烟测试

### 性能基准测试
- **benches/performance.rs**: 全面的性能测试
- **benches/regression_simple.rs**: 性能回归测试，用于 CI/CD
  - 空目录遍历：~14.5 µs
  - 单层目录（100个文件）：~295 µs
  - 深层目录结构（5层）：~161 µs
  - 文件过滤性能：35-39 µs
  - 深度限制效果：22-138 µs
  - 隐藏文件处理：~203 µs

### 运行测试
```bash
cargo test               # 运行所有测试
cargo test --lib         # 只运行单元测试
cargo test --test '*'     # 只运行集成测试
cargo test --doc          # 运行文档测试

# 性能基准测试
cargo bench              # 运行所有性能基准测试
cargo bench --bench regression_simple  # 运行回归测试（用于 CI/CD）
```

## 平台差异

- Unix 系统支持可执行文件检测（蓝色目录，红色可执行文件）
- Windows 平台的可执行文件检测功能尚未实现
- 跨平台路径处理使用标准库

## 依赖管理

### 主要依赖
- `clap 4.5`: 命令行参数解析，支持 derive 特性
- `globset 0.4`: 文件模式匹配，支持 glob 表达式
- `anstyle 1.0`: ANSI 文本样式定义（颜色、加粗等），无 I/O
- `anstream 1.0`: 包裹 stdout，按 `ColorChoice`（是否 TTY、`--color`/`--no-color`、`NO_COLOR`/`CLICOLOR` 等）自动保留或剥离 ANSI 颜色码

### 开发依赖
- `tempfile 3.0`: 测试用临时文件创建
- `criterion 0.5`: 性能基准测试框架，支持 HTML 报告
- `assert_cmd 2.0`: 集成测试工具

### 项目结构注意
- 项目同时定义了二进制目标和库目标
- `src/main.rs`: 二进制程序入口
- `src/lib.rs`: 库入口，导出所有模块供测试和基准测试使用
- 基准测试需要通过库来访问内部模块

## 调试和故障排除

### 常见问题

1. **基准测试编译错误**
   - 基准测试通过库目标 `tree_cli` 访问内部模块，需确保 `src/lib.rs` 已导出对应模块
   - 基准测试直接构造 `Config` 并驱动迭代器，不涉及终端输出

2. **性能测试超时**
   - 减少测试数据量或增加 sample 数量
   - 使用 `cargo bench -- --measurement-time <seconds>` 调整测试时间

3. **测试失败**
   - 单元测试检查错误断言
   - 集成测试确保程序已编译：`cargo build --release`

### 性能分析技巧

1. **使用 Criterion HTML 报告**
   - 报告位置：`target/criterion/report/index.html`
   - 包含详细的性能对比和统计分析

2. **本地性能监控**
   ```bash
   # 使用 time 命令测量执行时间
   time cargo run --release -- /path/to/large/directory

   # 使用 perf (Linux) 或 Instruments (macOS) 进行深入分析
   ```

3. **CI/CD 性能回归检测**
   - 自动在 PR 中运行性能基准测试
   - 性能下降超过 200% 会触发告警

## CI/CD 配置

项目使用 GitHub Actions 进行持续集成，包含以下任务：

### 工作流任务
- **测试**: 在 Ubuntu、Windows、macOS 上使用 stable 和 beta Rust 版本运行测试
- **代码覆盖率**: 使用 `cargo llvm-cov` 生成覆盖率报告并上传到 Codecov
- **性能基准测试**: 仅在主分支推送时运行，检测性能回归
- **代码质量**: 运行 Clippy 代码检查，对警告采用零容忍策略
- **格式检查**: 使用 rustfmt 检查代码格式
- **安全审计**: 检查依赖项的安全漏洞

### 重要说明
- 代码覆盖率只收集单元测试和库代码，集成测试因技术限制被排除
- 性能基准测试使用 `regression_simple` 进行快速回归检测
- 所有任务必须通过才能合并代码