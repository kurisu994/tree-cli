//! tree-cli 性能基准测试 - 简化版本
//!
//! 专注于测试核心功能的性能

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

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

/// 测试文件系统读取性能
fn bench_filesystem_reading(c: &mut Criterion) {
    let mut group = c.benchmark_group("文件系统读取性能");

    for (size, depth, files) in [
        ("小", 2, 10),
        ("中", 3, 20),
        ("大", 4, 30),
    ].iter() {
        let temp_dir = create_test_directory(*depth, *files);

        group.bench_with_input(
            BenchmarkId::new("读取目录", size),
            &(temp_dir.path()),
            |b, path| {
                b.iter(|| {
                    let count = fs::read_dir(black_box(path))
                        .expect("无法读取目录")
                        .count();
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

/// 测试递归目录遍历性能
fn bench_recursive_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("递归遍历性能");

    group.measurement_time(Duration::from_secs(10));

    for depth in [2, 3, 4].iter() {
        let temp_dir = create_test_directory(*depth, 15);

        group.bench_with_input(
            BenchmarkId::new("递归遍历", format!("{}层", depth)),
            &(temp_dir.path()),
            |b, path| {
                b.iter(|| {
                    let mut count = 0;
                    walk_directory(black_box(path), &mut count);
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

/// 递归遍历目录的辅助函数
fn walk_directory(path: &Path, count: &mut usize) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            *count += 1;
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                walk_directory(&entry.path(), count);
            }
        }
    }
}

/// 测试字符串操作性能（符号生成）
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("字符串操作性能");

    group.bench_function("生成符号前缀", |b| {
        b.iter(|| {
            let mut prefix = String::new();
            for i in 0..10 {
                prefix.push(if i % 2 == 0 { '│' } else { ' ' });
                prefix.push(' ');
                prefix.push(' ');
                prefix.push(' ');
            }
            if prefix.len() > 3 {
                prefix.pop();
                prefix.pop();
                prefix.pop();
                prefix.push('├');
                prefix.push('─');
                prefix.push('─');
                prefix.push(' ');
            }
            black_box(prefix);
        });
    });

    group.bench_function("路径字符串格式化", |b| {
        b.iter(|| {
            let name = "test_file.txt";
            let prefix = "│   │   └── ";
            let result = format!("{}{}", black_box(prefix), black_box(name));
            black_box(result);
        });
    });

    group.finish();
}

/// 测试内存分配性能
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("内存分配性能");

    group.bench_function("创建文件列表", |b| {
        b.iter(|| {
            let mut files = Vec::new();
            for i in 0..1000 {
                files.push(format!("file_{:04}.txt", i));
            }
            black_box(files);
        });
    });

    group.bench_function("创建路径向量", |b| {
        b.iter(|| {
            let base_path = Path::new("/test/directory");
            let mut paths = Vec::new();
            for i in 0..500 {
                paths.push(base_path.join(format!("file_{:03}.txt", i)));
            }
            black_box(paths);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_filesystem_reading,
    bench_recursive_traversal,
    bench_string_operations,
    bench_memory_allocation
);

criterion_main!(benches);