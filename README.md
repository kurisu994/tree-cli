# tree-cli

一个跨平台的目录树显示工具，是 Unix `tree` 命令的简单替代方案。

## 特性

- 🚀 **高性能**：使用 Rust 编写，单目录读取 < 30µs，内存占用低
- 🎨 **彩色输出**：支持彩色显示文件类型（蓝色目录，红色可执行文件）
- 🔍 **模式过滤**：支持使用 glob 模式包含或排除文件
- 📊 **统计信息**：显示目录和文件的数量统计
- 📏 **文件大小**：以人类可读格式显示文件大小（支持 EB 级别）
- 🌍 **跨平台**：支持 Linux、macOS 和 Windows
- ⚙️ **灵活配置**：支持多种选项配置
- 🛡️ **安全输出**：过滤文件名中的控制字符，防止恶意文件名借终端转义序列伪造或隐藏输出
- ✅ **全面测试**：包含 52 个测试用例，覆盖所有功能
- 📈 **性能监控**：使用 Criterion 进行性能回归检测

## 安装

### 从源码安装

```bash
git clone https://github.com/kurisu994/tree-cli.git
cd tree-cli
cargo install --path .
```

### 从 GitHub 安装

```bash
cargo install --git https://github.com/kurisu994/tree-cli.git
```

## 使用方法

### 基本用法

```bash
# 显示当前目录的树结构
tree-cli

# 显示指定目录
tree-cli /path/to/directory

# 显示所有文件（包括隐藏文件）
tree-cli -a

# 限制显示深度
tree-cli -L 2

# 启用彩色输出
tree-cli -C

# 禁用彩色输出
tree-cli -N

# 使用模式过滤文件
tree-cli -P "*.rs"  # 只显示 Rust 源文件

# 排除特定模式文件
tree-cli -E "*.txt"  # 排除所有 .txt 文件
tree-cli -E target    # 排除 target 目录

# 组合使用包含和排除模式
tree-cli -P "*.rs" -E "*.txt"  # 显示 .rs 文件但排除 .txt 文件

# 显示文件大小（人类可读格式）
tree-cli -s
```

### 命令行选项

| 选项 | 长选项 | 描述 |
|------|--------|------|
| `-a` | `--all` | 显示所有文件，包括隐藏文件 |
| `-C` | `--color` | 启用彩色输出 |
| `-N` | `--no-color` | 禁用彩色输出 |
| `-s` | `--human-readable` | 以人类可读格式显示文件大小 |
| `-P` | `--pattern <模式>` | 只显示匹配指定模式的文件 |
| `-E` | `--exclude <模式>` | 排除匹配指定模式的文件或目录 |
| `-L` | `--level <层数>` | 限制目录遍历深度 |
| `-h` | `--help` | 显示帮助信息 |
| `-V` | `--version` | 显示版本信息 |

### 示例

```shell
# 基本用法
tree-cli

.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── config
│   └── config.toml
├── rustfmt.toml
└── src
    ├── core.rs
    ├── file_iterator.rs
    ├── filter.rs
    ├── main.rs
    └── symbol.rs

3 directories, 11 files

# 显示 Rust 源文件，限制深度为 2
tree-cli -P "*.rs" -L 2

.
├── Cargo.toml
├── README.md
└── src
    ├── core.rs
    ├── file_iterator.rs
    ├── filter.rs
    ├── main.rs
    └── symbol.rs

1 directories, 6 files

# 显示所有文件（包括隐藏文件）
tree-cli -a -L 1

.
├── .git/
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
├── config/
├── rustfmt.toml
├── src/
└── target/

5 directories, 8 files

# 显示文件大小
tree-cli -s -L 1

.
├── .git/
├── .gitignore [32B]
├── Cargo.lock [2.1KB]
├── Cargo.toml [185B]
├── README.md [7.8KB]
├── config/
├── rustfmt.toml [23B]
├── src/
└── target/

5 directories, 8 files

# 排除特定文件或目录
tree-cli -E target -E "*.lock"

.
├── .git/
├── .gitignore
├── Cargo.toml
├── README.md
├── config/
├── rustfmt.toml
├── src/
└── .github/

5 directories, 6 files
```

## 开发

### 构建

```bash
# 构建调试版本
cargo build

# 构建发布版本
cargo build --release
```

### 测试

```bash
# 运行单元测试
cargo test

# 运行性能基准测试
cargo bench

# 运行代码检查
cargo clippy

# 格式化代码
cargo fmt
```

### 性能

tree-cli 具有出色的性能表现：

- 🚀 **极快的文件系统访问**：单目录读取 < 30µs
- 💾 **低内存占用**：流式处理，不预加载所有文件
- ⚡ **高效的递归遍历**：支持深度限制和模式过滤
- 📈 **线性扩展性**：性能随目录大小线性增长

详细性能数据请参考 [BENCHMARKS.md](BENCHMARKS.md)。

### 测试覆盖

tree-cli 包含全面的测试套件：

- **单元测试**：30个测试，覆盖所有核心功能
- **集成测试**：22个测试，验证命令行功能
  - 基本功能测试（8个）
  - 排除功能测试（4个）
  - 颜色和选项测试（9个）
  - 基础冒烟测试（1个）
- **性能基准测试**：使用 Criterion 监控性能回归

测试覆盖率提升36%，新增测试场景：
- 颜色输出控制
- 错误处理和边界条件
- Unicode和特殊字符支持
- 符号链接处理
- 深层嵌套和大型目录性能

运行测试：
```bash
cargo test              # 运行所有测试（52个测试）
cargo bench             # 运行性能基准测试
```

## 项目结构

```
tree-cli/
├── src/
│   ├── main.rs            # 程序入口和命令行参数解析
│   ├── lib.rs             # 库入口，导出所有模块
│   ├── core.rs            # 核心逻辑和目录树生成
│   ├── file_iterator.rs   # 文件迭代器实现
│   ├── filter.rs          # 文件过滤逻辑
│   └── symbol.rs          # 符号显示和颜色控制
├── tests/
│   ├── test.rs                    # 基础测试
│   ├── integration_test.rs        # 基本功能集成测试
│   ├── test_exclude.rs            # 排除功能测试
│   └── test_color_and_options.rs  # 颜色和选项测试
├── benches/
│   ├── simple_perf.rs     # 简单性能基准测试
│   ├── performance.rs     # 全面性能测试
│   └── regression_simple.rs # 性能回归测试
└── .github/workflows/
    └── ci.yml             # CI/CD 配置
```

## 更新日志

各版本变更详见 [CHANGELOG.md](CHANGELOG.md)。

## License

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

### 开发指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### 代码规范

- 使用 Rust 2024 edition
- 遵循 rustfmt 格式化规则（最大行宽 120）
- 为所有公开函数和结构体添加中文注释
- 确保所有测试通过：`cargo test`
- 运行代码检查：`cargo clippy`