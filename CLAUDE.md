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
- **symbol.rs**: 树形结构的符号定义和彩色输出控制

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
cargo test               # 运行单元测试
cargo bench              # 运行性能基准测试（使用 Criterion）

# 代码质量
cargo clippy            # 代码检查
cargo fmt               # 代码格式化（遵循 rustfmt.toml 配置）

# 安装
cargo install --path .  # 从源码本地安装
```

## 代码规范

- 使用 Rust 2024 Edition
- 最大行宽：120 字符
- 使用 4 个空格缩进
- 所有公开的函数和结构体必须包含中文注释
- 复杂逻辑需要行内中文注释

## 性能考虑

- 文件系统访问经过优化，单目录读取 < 30µs
- 使用 `VecDeque` 实现高效的队列操作
- 字符串操作使用 `format_args!` 宏避免额外分配
- 排序使用 `unstable_by` 提升性能

## 测试策略

项目包含全面的测试套件：

### 单元测试
- **core.rs**: 测试配置结构、符号切换逻辑和目录摘要
- **file_iterator.rs**: 测试文件迭代、目录遍历、隐藏文件过滤
- **filter.rs**: 测试过滤逻辑、空目录处理、缓存机制
- **symbol.rs**: 测试符号生成、颜色输出、可执行文件检测

### 集成测试
- **tests/integration_test.rs**: 验证命令行功能
  - 基本目录树显示
  - 隐藏文件选项 (-a)
  - 深度限制 (-L)
  - 模式过滤 (-P)
  - 帮助和版本信息
  - 空目录处理

### 性能基准测试
- **benches/performance.rs**: 全面的性能测试
- **benches/regression_simple.rs**: 性能回归测试
  - 空目录遍历
  - 单层目录（100个文件）
  - 深层目录结构（5层）
  - 文件过滤性能
  - 深度限制效果
  - 隐藏文件处理

### 运行测试
```bash
cargo test               # 运行所有测试
cargo test --lib         # 只运行单元测试
cargo test --test '*'     # 只运行集成测试
cargo bench              # 运行性能基准测试
cargo bench --bench regression_simple  # 运行回归测试
```

## 平台差异

- Unix 系统支持可执行文件检测（蓝色目录，红色可执行文件）
- Windows 平台的可执行文件检测功能尚未实现
- 跨平台路径处理使用标准库

## 依赖管理

主要依赖：
- `clap 4.5`: 命令行参数解析
- `globset 0.4`: 文件模式匹配
- `term 0.7`: 终端控制和彩色输出

开发依赖：
- `tempfile 3.0`: 测试用临时文件
- `criterion 0.5`: 性能基准测试