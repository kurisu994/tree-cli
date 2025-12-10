use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// 测试颜色输出功能
#[test]
fn test_color_output() {
    let temp_dir = TempDir::new().unwrap();

    // 创建测试目录和文件
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();
    fs::write(temp_dir.path().join("file.txt"), "content").unwrap();
    fs::write(temp_dir.path().join("script.sh"), "#!/bin/bash\necho test").unwrap();

    // 先编译程序
    let compile_output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to compile tree-cli");
    assert!(compile_output.status.success());

    // 测试强制启用颜色 (-C)
    let output = Command::new("./target/release/tree-cli")
        .args(&["-C", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 检查是否包含 ANSI 颜色代码
    assert!(stdout.contains("\x1b[")); // ANSI escape sequence

    // 测试禁用颜色 (-N)
    let output = Command::new("./target/release/tree-cli")
        .args(&["-N", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 检查不包含 ANSI 颜色代码
    assert!(!stdout.contains("\x1b["));
}

/// 测试多参数组合
#[test]
fn test_multiple_parameters() {
    let temp_dir = TempDir::new().unwrap();

    // 创建复杂的目录结构
    fs::create_dir_all(temp_dir.path().join("level1/level2")).unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.rs"), "fn main() {}").unwrap();
    fs::write(temp_dir.path().join(".hidden"), "hidden").unwrap();
    fs::write(temp_dir.path().join("level1/file3.txt"), "content3").unwrap();

    // 创建一个较大的文件用于测试大小显示
    let large_content = "x".repeat(1024 * 10); // 10KB
    fs::write(temp_dir.path().join("large.txt"), large_content).unwrap();

    // 先编译程序
    let compile_output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to compile tree-cli");
    assert!(compile_output.status.success());

    // 测试组合多个参数：显示所有文件 + 限制深度 + 显示大小 + 只显示txt文件
    let output = Command::new("./target/release/tree-cli")
        .args(&[
            "-a",                    // 显示所有文件
            "-L", "2",              // 限制深度为2
            "-s",                   // 显示文件大小
            "-P", "*.txt",          // 只显示txt文件
            "-C",                   // 启用颜色
            temp_dir.path().to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 验证所有参数都生效了
    assert!(stdout.contains("file1.txt"));  // 显示的文件
    assert!(stdout.contains("file3.txt"));  // 显示的文件
    assert!(!stdout.contains("file2.rs"));  // 被过滤掉的文件
    assert!(stdout.contains(".hidden"));    // 显示的隐藏文件
    assert!(!stdout.contains("level2"));    // 超出深度限制
    assert!(stdout.contains("[") || stdout.contains("B"));  // 包含大小信息
    assert!(stdout.contains("\x1b["));      // 包含颜色代码
}

/// 测试错误处理
#[test]
fn test_error_handling() {
    // 先编译程序
    let compile_output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to compile tree-cli");
    assert!(compile_output.status.success());

    // 测试不存在的路径
    let output = Command::new("./target/release/tree-cli")
        .arg("/nonexistent/path/that/should/not/exist")
        .output()
        .expect("Failed to execute tree-cli");

    // 应该返回错误
    assert!(!output.status.success());

    // 测试空字符串路径（在某些系统上可能触发错误）
    #[cfg(unix)]
    {
        let _output = Command::new("./target/release/tree-cli")
            .arg("")
            .output()
            .expect("Failed to execute tree-cli");

        // 空路径应该被视为当前目录或报错
        // 具体行为取决于实现，这里只测试不会panic
    }
}

/// 测试特殊字符和Unicode文件名
#[test]
fn test_special_characters_and_unicode() {
    let temp_dir = TempDir::new().unwrap();

    // 创建包含特殊字符和Unicode的文件名
    // 使用Vec来避免&str生命周期问题
    let mut special_files = Vec::new();
    special_files.push("文件.txt".to_string());           // 中文
    special_files.push("файл.rs".to_string());            // 俄文
    special_files.push("🦀 rustacean.py".to_string());   // Emoji
    special_files.push("file with spaces.txt".to_string()); // 空格

    // 添加长文件名
    let long_filename = "a".repeat(100); // 使用100个字符，避免某些文件系统限制
    special_files.push(long_filename);

    // 在支持的系统上测试更多特殊字符
    #[cfg(unix)]
    {
        special_files.push("file_with-dashes.txt".to_string());
        special_files.push("file_with.dots.txt".to_string());
        special_files.push("file_with_underscores.txt".to_string());
    }

    let mut created_files = Vec::new();

    for filename in &special_files {
        let path = temp_dir.path().join(filename);
        match fs::write(&path, "test content") {
            Ok(_) => created_files.push(filename.clone()),
            Err(_) => {
                // 某些文件名可能在特定系统上不支持，忽略错误
                println!("Warning: Could not create file with name: {}", filename);
            }
        }
    }

    // 确保至少创建了一些文件
    assert!(!created_files.is_empty(), "No test files were created");

    // 先编译程序
    let compile_output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to compile tree-cli");
    assert!(compile_output.status.success());

    // 测试程序能否正确处理这些文件名
    let output = Command::new("./target/release/tree-cli")
        .args(&["-a", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // 验证能处理一些特殊文件名
    // 检查常见的Unicode字符
    let found_unicode = stdout.contains("文件.txt") ||
                       stdout.contains("файл.rs") ||
                       stdout.contains("🦀");

    // 检查空格文件名
    let found_spaces = stdout.contains("file with spaces.txt");

    // 检查长文件名（通过查找连续的'a'）
    let found_long = stdout.lines().any(|line| line.contains("aaaaa"));

    // 至少应该找到一种特殊文件名
    assert!(found_unicode || found_spaces || found_long,
            "No special character files found in output. Output:\n{}", stdout);
}

/// 测试深层嵌套目录
#[test]
fn test_deep_nested_directories() {
    let temp_dir = TempDir::new().unwrap();

    // 创建深层嵌套目录（50层）
    let mut path = temp_dir.path().to_path_buf();
    for i in 0..50 {
        path = path.join(format!("level{}", i));
    }
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "deep file").unwrap();

    // 先编译程序
    let compile_output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to compile tree-cli");
    assert!(compile_output.status.success());

    // 不限制深度
    let output = Command::new("./target/release/tree-cli")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 应该包含所有层级的目录
    assert!(stdout.contains("level0"));
    assert!(stdout.contains("level49"));

    // 限制深度为10
    let output = Command::new("./target/release/tree-cli")
        .args(&["-L", "10", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 应该只显示前10层
    assert!(stdout.contains("level0"));
    assert!(stdout.contains("level9"));
    assert!(!stdout.contains("level10"));
}

#[cfg(unix)]
/// 测试符号链接处理
#[test]
fn test_symlink_handling() {
    use std::os::unix::fs::symlink;

    let temp_dir = TempDir::new().unwrap();

    // 创建原始文件和目录
    fs::create_dir(temp_dir.path().join("original_dir")).unwrap();
    fs::write(temp_dir.path().join("original_file.txt"), "original content").unwrap();

    // 创建符号链接
    symlink(temp_dir.path().join("original_dir"), temp_dir.path().join("link_to_dir")).unwrap();
    symlink(temp_dir.path().join("original_file.txt"), temp_dir.path().join("link_to_file")).unwrap();

    // 创建指向不存在的文件的符号链接
    symlink(temp_dir.path().join("nonexistent"), temp_dir.path().join("broken_link")).unwrap();

    // 先编译程序
    let compile_output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to compile tree-cli");
    assert!(compile_output.status.success());

    // 测试符号链接显示
    let output = Command::new("./target/release/tree-cli")
        .args(&["-a", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 验证符号链接被正确显示
    assert!(stdout.contains("link_to_dir") || stdout.contains("link_to_file"));
}

/// 测试大型目录性能
#[test]
fn test_large_directory_performance() {
    let temp_dir = TempDir::new().unwrap();

    // 创建大量文件（1000个文件）
    for i in 0..1000 {
        fs::write(temp_dir.path().join(format!("file_{:04}.txt", i)), "content").unwrap();
    }

    // 先编译程序
    let compile_output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to compile tree-cli");
    assert!(compile_output.status.success());

    // 测量执行时间
    let start = std::time::Instant::now();
    let output = Command::new("./target/release/tree-cli")
        .arg(temp_dir.path())
        .output()
        .expect("Failed to execute tree-cli");
    let duration = start.elapsed();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 验证显示了正确数量的文件
    assert!(stdout.contains("1000 files") || stdout.contains("1000 file"));

    // 性能应该在合理范围内（这里设置为5秒，实际应该更快）
    assert!(duration.as_secs() < 5, "处理1000个文件耗时过长: {:?}", duration);

    // 打印实际耗时以供参考
    println!("处理1000个文件耗时: {:?}", duration);
}

/// 测试短选项 -E 的功能
#[test]
fn test_exclude_short_option() {
    let temp_dir = TempDir::new().unwrap();

    // 创建各种文件
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.rs"), "content2").unwrap();
    fs::write(temp_dir.path().join("test.md"), "markdown").unwrap();

    // 测试使用 -E 选项排除所有 .txt 文件
    let output = Command::new("./target/release/tree-cli")
        .args(&["-E", "*.txt", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 应该包含 .rs 和 .md 文件但不包含 .txt 文件
    assert!(stdout.contains("file2.rs"));
    assert!(stdout.contains("test.md"));
    assert!(!stdout.contains("file1.txt"));
}

/// 测试边界值：空路径和根目录
#[test]
fn test_edge_cases_paths() {
    // 先编译程序
    let compile_output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()
        .expect("Failed to compile tree-cli");
    assert!(compile_output.status.success());

    // 测试根目录（Unix系统）
    #[cfg(unix)]
    {
        let output = Command::new("./target/release/tree-cli")
            .args(&["-L", "1", "/"])  // 限制深度避免扫描整个文件系统
            .output()
            .expect("Failed to execute tree-cli");

        // 应该能成功执行（可能需要权限）
        // 这里只测试不会panic
        let _ = String::from_utf8(output.stdout);
    }

    // 测试当前目录（.）
    let output = Command::new("./target/release/tree-cli")
        .args(&["-L", "1", "."])
        .output()
        .expect("Failed to execute tree-cli");

    assert!(output.status.success());
}