//! 测试 exclude 功能的集成测试

use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_exclude_with_pattern() {
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();

    // 创建各种文件
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.rs"), "content2").unwrap();
    fs::write(temp_dir.path().join("test.md"), "markdown").unwrap();

    // 创建子目录
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();
    fs::write(temp_dir.path().join("subdir/file3.txt"), "content3").unwrap();
    fs::write(temp_dir.path().join("subdir/file4.rs"), "content4").unwrap();

    // 测试排除所有 .txt 文件
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    cmd.current_dir(temp_dir.path()).args(["--exclude", "*.txt"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 应该包含 .rs 文件但不包含 .txt 文件
    assert!(stdout.contains("file2.rs"));
    assert!(stdout.contains("file4.rs"));
    assert!(!stdout.contains("file1.txt"));
    assert!(!stdout.contains("file3.txt"));
}

#[test]
fn test_exclude_single_pattern() {
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();

    // 创建各种文件
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.rs"), "content2").unwrap();
    fs::write(temp_dir.path().join("test.md"), "markdown").unwrap();
    fs::write(temp_dir.path().join("target.exe"), "executable").unwrap();

    // 测试排除 .txt 文件
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    cmd.current_dir(temp_dir.path()).args(["--exclude", "*.txt"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 应该包含其他文件但不包含 .txt 文件
    assert!(stdout.contains("file2.rs"));
    assert!(stdout.contains("test.md"));
    assert!(stdout.contains("target.exe"));
    assert!(!stdout.contains("file1.txt"));
}

#[test]
fn test_exclude_directories() {
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();

    // 创建目录
    fs::create_dir(temp_dir.path().join("target")).unwrap();
    fs::write(temp_dir.path().join("target/file1.txt"), "content1").unwrap();

    fs::create_dir(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/main.rs"), "main").unwrap();

    // 测试排除 target 目录
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    cmd.current_dir(temp_dir.path()).args(["--exclude", "target"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 应该包含 src 目录但不包含 target
    assert!(stdout.contains("src"));
    assert!(!stdout.contains("target"));
    assert!(stdout.contains("main.rs"));
}

#[test]
fn test_exclude_with_include() {
    // 创建临时目录结构
    let temp_dir = TempDir::new().unwrap();

    // 创建各种文件
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.rs"), "content2").unwrap();
    fs::write(temp_dir.path().join("file3.md"), "markdown").unwrap();
    fs::write(temp_dir.path().join("file4.py"), "python").unwrap();

    // 测试同时使用 include 和 exclude - include 只用单个模式
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    cmd.current_dir(temp_dir.path())
        .args(["--pattern", "*.rs"])
        .args(["--exclude", "*.py"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 应该只包含 .rs 文件，排除其他文件
    assert!(stdout.contains("file2.rs"));
    assert!(!stdout.contains("file1.txt"));
    assert!(!stdout.contains("file3.md"));
    assert!(!stdout.contains("file4.py"));
}
