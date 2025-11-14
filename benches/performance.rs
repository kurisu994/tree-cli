//! tree-cli 性能基准测试
//!
//! 该模块包含针对 tree-cli 各个组件的性能测试，包括：
//! - 目录遍历性能
//! - 文件过滤性能
//! - 内存使用分析
//! - 大目录处理能力

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::{TempDir, NamedTempFile};
use tree_cli::core::{DirTree, DirSummary, Config};
use tree_cli::file_iterator::{FileItem, FileIterator};
use tree_cli::filter::FilteredIterator;

/// 创建测试目录结构
fn create_test_directory(depth: usize, files_per_dir: usize) -> TempDir {
    let temp_dir = TempDir::new().expect("无法创建临时目录");

    fn create_dir_recursive(path: &Path, current_depth: usize, max_depth: usize, files_per_dir: usize) {
        if current_depth > max_depth {
            return;
        }

        // 在当前目录创建文件
        for i in 0..files_per_dir {
            let file_path = path.join(format!("file_{:03}.txt", i));
            fs::write(file_path, format!("File content {}", i)).expect("无法创建测试文件");
        }

        // 创建子目录
        for i in 0..3 {
            let sub_dir = path.join(format!("dir_{}", i));
            fs::create_dir(&sub_dir).expect("无法创建测试目录");
            create_dir_recursive(&sub_dir, current_depth + 1, max_depth, files_per_dir);
        }
    }

    create_dir_recursive(temp_dir.path(), 0, depth, files_per_dir);
    temp_dir
}

/// 创建带隐藏文件的测试目录
fn create_test_directory_with_hidden() -> TempDir {
    let temp_dir = TempDir::new().expect("无法创建临时目录");

    // 创建普通文件
    for i in 0..10 {
        let file_path = temp_dir.path().join(format!("file_{}.txt", i));
        fs::write(file_path, format!("Content {}", i)).expect("无法创建测试文件");
    }

    // 创建隐藏文件
    for i in 0..5 {
        let file_path = temp_dir.path().join(format!(".hidden_file_{}.txt", i));
        fs::write(file_path, format!("Hidden content {}", i)).expect("无法创建隐藏文件");
    }

    // 创建隐藏目录
    let hidden_dir = temp_dir.path().join(".hidden_dir");
    fs::create_dir(&hidden_dir).expect("无法创建隐藏目录");
    for i in 0..3 {
        let file_path = hidden_dir.join(format!("hidden_file_{}.txt", i));
        fs::write(file_path, format!("Hidden dir content {}", i)).expect("无法创建隐藏目录文件");
    }

    temp_dir
}

/// 创建不同类型文件的测试目录
fn create_test_directory_with_various_file_types() -> TempDir {
    let temp_dir = TempDir::new().expect("无法创建临时目录");

    // 创建不同类型的文件
    let files = vec![
        ("script.sh", "#!/bin/bash\necho 'Hello World'"),
        ("program.rs", "fn main() { println!(\"Hello\"); }"),
        ("data.json", "{\"key\": \"value\"}"),
        ("config.toml", "[settings]\nenabled = true"),
        ("document.md", "# 标题\n这是文档内容"),
        ("image.png", b"PNG\x89\x0D\x0A\x1A\x0A"),
        ("executable", b"ELF"),
    ];

    for (filename, content) in files {
        let file_path = temp_dir.path().join(filename);
        fs::write(file_path, content).expect("无法创建测试文件");
    }

    temp_dir
}

/// 创建模拟终端输出
fn create_mock_terminal() -> Box<dyn std::io::Write> {
    Box::new(Vec::new())
}

/// 基准测试：小目录遍历性能
fn bench_small_directory_traversal(c: &mut Criterion) {
    let temp_dir = create_test_directory(2, 10);
    let mut group = c.benchmark_group("小目录遍历");

    group.measurement_time(Duration::from_secs(10));

    group.bench_function("遍历2层深度目录", |b| {
        b.iter(|| {
            let config = Config {
                colorful: false,
                show_all: false,
                max_level: usize::max_value(),
                include_glob: None,
            };
            let mut terminal = create_mock_terminal();
            let mut dir_tree = DirTree::new(config, &mut terminal);
            let _summary: DirSummary = dir_tree.print_folders(black_box(temp_dir.path()))
                .expect("遍历目录失败");
        });
    });

    group.finish();
}

/// 基准测试：中等目录遍历性能
fn bench_medium_directory_traversal(c: &mut Criterion) {
    let temp_dir = create_test_directory(3, 20);
    let mut group = c.benchmark_group("中等目录遍历");

    group.measurement_time(Duration::from_secs(15));

    group.bench_function("遍历3层深度目录", |b| {
        b.iter(|| {
            let config = Config {
                colorful: false,
                show_all: false,
                max_level: usize::max_value(),
                include_glob: None,
            };
            let mut terminal = create_mock_terminal();
            let mut dir_tree = DirTree::new(config, &mut terminal);
            let _summary: DirSummary = dir_tree.print_folders(black_box(temp_dir.path()))
                .expect("遍历目录失败");
        });
    });

    group.finish();
}

/// 基准测试：大目录遍历性能
fn bench_large_directory_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("大目录遍历");

    for depth in [2, 3, 4].iter() {
        let temp_dir = create_test_directory(*depth, 50);

        group.bench_with_input(
            BenchmarkId::new("遍历目录", format!("{}层深度", depth)),
            depth,
            |b, _| {
                b.iter(|| {
                    let config = Config {
                        colorful: false,
                        show_all: false,
                        max_level: usize::max_value(),
                        include_glob: None,
                    };
                    let mut terminal = create_mock_terminal();
                    let mut dir_tree = DirTree::new(config, &mut terminal);
                    let _summary: DirSummary = dir_tree.print_folders(black_box(temp_dir.path()))
                        .expect("遍历目录失败");
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：文件迭代器性能
fn bench_file_iterator(c: &mut Criterion) {
    let temp_dir = create_test_directory(3, 30);
    let config = Config {
        colorful: false,
        show_all: false,
        max_level: usize::max_value(),
        include_glob: None,
    };

    let mut group = c.benchmark_group("文件迭代器");

    group.bench_function("创建文件迭代器", |b| {
        b.iter(|| {
            let _iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
        });
    });

    group.bench_function("遍历所有文件", |b| {
        b.iter(|| {
            let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
            let count = iterator.count();
            black_box(count);
        });
    });

    group.finish();
}

/// 基准测试：文件过滤性能
fn bench_file_filtering(c: &mut Criterion) {
    let temp_dir = create_test_directory_with_various_file_types();
    let mut group = c.benchmark_group("文件过滤");

    // 测试无过滤的情况
    group.bench_function("无过滤遍历", |b| {
        b.iter(|| {
            let config = Config {
                colorful: false,
                show_all: false,
                max_level: usize::max_value(),
                include_glob: None,
            };
            let mut terminal = create_mock_terminal();
            let mut dir_tree = DirTree::new(config, &mut terminal);
            let _summary: DirSummary = dir_tree.print_folders(black_box(temp_dir.path()))
                .expect("遍历目录失败");
        });
    });

    // 测试模式匹配过滤
    group.bench_function("模式匹配过滤 (*.rs)", |b| {
        b.iter(|| {
            let config = Config {
                colorful: false,
                show_all: false,
                max_level: usize::max_value(),
                include_glob: Some(globset::Glob::new("*.rs").unwrap().compile_matcher()),
            };
            let mut terminal = create_mock_terminal();
            let mut dir_tree = DirTree::new(config, &mut terminal);
            let _summary: DirSummary = dir_tree.print_folders(black_box(temp_dir.path()))
                .expect("遍历目录失败");
        });
    });

    // 测试隐藏文件过滤
    group.bench_function("隐藏文件过滤", |b| {
        b.iter(|| {
            let config = Config {
                colorful: false,
                show_all: true,
                max_level: usize::max_value(),
                include_glob: None,
            };
            let mut terminal = create_mock_terminal();
            let mut dir_tree = DirTree::new(config, &mut terminal);
            let _summary: DirSummary = dir_tree.print_folders(black_box(temp_dir.path()))
                .expect("遍历目录失败");
        });
    });

    group.finish();
}

/// 基准测试：深度限制性能
fn bench_depth_limiting(c: &mut Criterion) {
    let temp_dir = create_test_directory(5, 10);
    let mut group = c.benchmark_group("深度限制");

    for max_depth in [1, 2, 3, 4, 5].iter() {
        group.bench_with_input(
            BenchmarkId::new("深度限制", max_depth),
            max_depth,
            |b, &max_depth| {
                b.iter(|| {
                    let config = Config {
                        colorful: false,
                        show_all: false,
                        max_level: max_depth,
                        include_glob: None,
                    };
                    let mut terminal = create_mock_terminal();
                    let mut dir_tree = DirTree::new(config, &mut terminal);
                    let _summary: DirSummary = dir_tree.print_folders(black_box(temp_dir.path()))
                        .expect("遍历目录失败");
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：内存使用和大型目录处理
fn bench_memory_usage(c: &mut Criterion) {
    let temp_dir = create_test_directory(4, 100);
    let mut group = c.benchmark_group("内存使用测试");

    group.measurement_time(Duration::from_secs(20));

    group.bench_function("处理大型目录", |b| {
        b.iter(|| {
            let config = Config {
                colorful: false,
                show_all: false,
                max_level: usize::max_value(),
                include_glob: None,
            };
            let mut terminal = create_mock_terminal();
            let mut dir_tree = DirTree::new(config, &mut terminal);
            let _summary: DirSummary = dir_tree.print_folders(black_box(temp_dir.path()))
                .expect("遍历目录失败");
        });
    });

    group.finish();
}

/// 基准测试：符号生成性能
fn bench_symbol_generation(c: &mut Criterion) {
    use tree_cli::symbol::set_line_prefix;

    let mut group = c.benchmark_group("符号生成");

    group.bench_function("浅层目录符号生成", |b| {
        b.iter(|| {
            let symbol_list = vec![true, true, false];
            let mut prefix = String::new();
            set_line_prefix(black_box(&symbol_list), black_box(&mut prefix));
            black_box(prefix);
        });
    });

    group.bench_function("深层目录符号生成", |b| {
        b.iter(|| {
            let symbol_list = vec![true; 50];
            let mut prefix = String::new();
            set_line_prefix(black_box(&symbol_list), black_box(&mut prefix));
            black_box(prefix);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_directory_traversal,
    bench_medium_directory_traversal,
    bench_large_directory_traversal,
    bench_file_iterator,
    bench_file_filtering,
    bench_depth_limiting,
    bench_memory_usage,
    bench_symbol_generation
);

criterion_main!(benches);