//! 简化的性能回归测试
//!
//! 只测试核心功能，不依赖终端输出

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::time::Duration;
use tempfile::TempDir;
use tree_cli::file_iterator::FileIterator;
use tree_cli::core::Config;

/// 基准测试：空目录遍历
fn bench_empty_directory(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("无法创建临时目录");
    let config = Config {
        colorful: false,
        show_all: false,
        size: false,
        max_level: 10,
        include_glob: None,
    };

    let mut group = c.benchmark_group("回归测试-空目录");
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("遍历空目录", |b| {
        b.iter(|| {
            let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
            let count = iterator.count();
            black_box(count);
        });
    });

    group.finish();
}

/// 基准测试：单层目录
fn bench_single_level_directory(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("无法创建临时目录");

    // 创建100个文件
    for i in 0..100 {
        let file_path = temp_dir.path().join(format!("file_{:03}.txt", i));
        fs::write(file_path, format!("Content {}", i)).expect("无法创建测试文件");
    }

    let config = Config {
        colorful: false,
        show_all: false,
        size: false,
        max_level: 1,
        include_glob: None,
    };

    let mut group = c.benchmark_group("回归测试-单层目录");

    group.bench_function("遍历100个文件的目录", |b| {
        b.iter(|| {
            let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
            let count = iterator.count();
            black_box(count);
        });
    });

    group.finish();
}

/// 基准测试：深层目录结构
fn bench_deep_directory(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("无法创建临时目录");

    // 创建5层深度的目录结构
    fn create_deep_structure(path: &tempfile::TempDir, depth: usize, current_depth: usize) {
        if current_depth >= depth {
            return;
        }

        let current_dir = path.path().join(format!("level_{}", current_depth));
        fs::create_dir(&current_dir).expect("无法创建目录");

        // 在当前层创建5个文件
        for i in 0..5 {
            let file_path = current_dir.join(format!("file_{:03}.txt", i));
            fs::write(file_path, format!("Content at depth {}", current_depth))
                .expect("无法创建文件");
        }

        create_deep_structure(path, depth, current_depth + 1);
    }

    create_deep_structure(&temp_dir, 5, 0);

    let config = Config {
        colorful: false,
        show_all: false,
        size: false,
        max_level: 5,
        include_glob: None,
    };

    let mut group = c.benchmark_group("回归测试-深层目录");

    group.bench_function("遍历5层深度目录", |b| {
        b.iter(|| {
            let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
            let count = iterator.count();
            black_box(count);
        });
    });

    group.finish();
}

/// 基准测试：文件过滤性能
fn bench_filter_performance(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("无法创建临时目录");

    // 创建多种类型的文件
    let files = vec![
        ("script.sh", "#!/bin/bash"),
        ("program.rs", "fn main() {}"),
        ("data.json", "{}"),
        ("config.yaml", "key: value"),
        ("readme.md", "# Title"),
        ("document.txt", "Some text"),
        ("archive.zip", "ZIP"),
        ("image.png", "PNG"),
    ];

    for (name, content) in files {
        fs::write(temp_dir.path().join(name), content).expect("无法创建文件");
    }

    let mut group = c.benchmark_group("回归测试-文件过滤");

    // 测试无过滤
    group.bench_function("无过滤", |b| {
        let config = Config {
            colorful: false,
            show_all: false,
        size: false,
            max_level: 1,
            include_glob: None,
        };
        b.iter(|| {
            let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
            let count = iterator.count();
            black_box(count);
        });
    });

    // 测试 glob 过滤
    group.bench_function("Glob过滤 (*.rs)", |b| {
        let config = Config {
            colorful: false,
            show_all: false,
        size: false,
            max_level: 1,
            include_glob: Some(globset::Glob::new("*.rs").unwrap().compile_matcher()),
        };
        b.iter(|| {
            let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
            let count = iterator.count();
            black_box(count);
        });
    });

    group.finish();
}

/// 基准测试：深度限制性能
fn bench_depth_limiting(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("无法创建临时目录");

    // 创建3层深度目录
    for i in 0..3 {
        let dir_path = temp_dir.path().join(format!("level_{}", i));
        fs::create_dir(&dir_path).expect("无法创建目录");

        for j in 0..10 {
            let file_path = dir_path.join(format!("file_{:03}.txt", j));
            fs::write(file_path, "Content").expect("无法创建文件");
        }
    }

    let mut group = c.benchmark_group("回归测试-深度限制");

    for max_depth in [1, 2, 3].iter() {
        group.bench_with_input(
            criterion::BenchmarkId::new("深度限制", max_depth),
            max_depth,
            |b, &max_depth| {
                let config = Config {
                    colorful: false,
                    show_all: false,
        size: false,
                    max_level: max_depth,
                    include_glob: None,
                };
                b.iter(|| {
                    let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
                    let count = iterator.count();
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

/// 基准测试：隐藏文件处理
fn bench_hidden_files(c: &mut Criterion) {
    let temp_dir = TempDir::new().expect("无法创建临时目录");

    // 创建普通文件
    for i in 0..50 {
        let file_path = temp_dir.path().join(format!("file_{:03}.txt", i));
        fs::write(file_path, "Content").expect("无法创建文件");
    }

    // 创建隐藏文件
    for i in 0..20 {
        let file_path = temp_dir.path().join(format!(".hidden_{:03}.txt", i));
        fs::write(file_path, "Hidden content").expect("无法创建文件");
    }

    let mut group = c.benchmark_group("回归测试-隐藏文件");

    // 不显示隐藏文件
    group.bench_function("不显示隐藏文件", |b| {
        let config = Config {
            colorful: false,
            show_all: false,
        size: false,
            max_level: 1,
            include_glob: None,
        };
        b.iter(|| {
            let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
            let count = iterator.count();
            black_box(count);
        });
    });

    // 显示隐藏文件
    group.bench_function("显示隐藏文件", |b| {
        let config = Config {
            colorful: false,
            show_all: true,
        size: false,
            max_level: 1,
            include_glob: None,
        };
        b.iter(|| {
            let iterator = FileIterator::new(black_box(temp_dir.path()), black_box(&config));
            let count = iterator.count();
            black_box(count);
        });
    });

    group.finish();
}

criterion_group!(
    regression_benches,
    bench_empty_directory,
    bench_single_level_directory,
    bench_deep_directory,
    bench_filter_performance,
    bench_depth_limiting,
    bench_hidden_files
);

criterion_main!(regression_benches);