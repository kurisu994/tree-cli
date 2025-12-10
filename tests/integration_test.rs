use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// 测试基本的目录树显示功能
#[test]
fn test_basic_tree_display() {
    let temp_dir = TempDir::new().unwrap();

    // 创建测试目录结构
    fs::create_dir(temp_dir.path().join("subdir")).unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("subdir/file2.txt"), "content2").unwrap();

    // 运行程序
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd.arg(temp_dir.path()).output().expect("Failed to execute tree-cli");

    if !output.status.success() {
        eprintln!("Command failed with status: {:?}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // 验证输出包含预期的文件和目录
    assert!(stdout.contains("subdir"));
    assert!(stdout.contains("file1.txt"));
    assert!(stdout.contains("file2.txt"));
    assert!(stdout.contains("directories") || stdout.contains("directory"));
    assert!(stdout.contains("files") || stdout.contains("file"));
}

/// 测试显示隐藏文件选项
#[test]
fn test_show_all_files() {
    let temp_dir = TempDir::new().unwrap();

    // 创建隐藏文件
    fs::write(temp_dir.path().join(".hidden"), "hidden content").unwrap();
    fs::write(temp_dir.path().join("normal.txt"), "normal content").unwrap();

    // 不使用 -a 选项
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd.arg(temp_dir.path()).output().expect("Failed to execute tree-cli");

    if !output.status.success() {
        eprintln!("Command failed with status: {:?}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains(".hidden"));
    assert!(stdout.contains("normal.txt"));

    // 使用 -a 选项
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd
        .args(["-a", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(".hidden"));
    assert!(stdout.contains("normal.txt"));
}

/// 测试深度限制选项
#[test]
fn test_level_limit() {
    let temp_dir = TempDir::new().unwrap();

    // 创建多层目录结构
    fs::create_dir_all(temp_dir.path().join("level1/level2/level3")).unwrap();
    fs::write(temp_dir.path().join("level1/level2/level3/deep.txt"), "deep content").unwrap();

    // 不限制深度
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd.arg(temp_dir.path()).output().expect("Failed to execute tree-cli");

    if !output.status.success() {
        eprintln!("Command failed with status: {:?}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("level1"));
    assert!(stdout.contains("level2"));
    assert!(stdout.contains("level3"));
    assert!(stdout.contains("deep.txt"));

    // 限制深度为 2
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd
        .args(["-L", "2", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("level1"));
    assert!(stdout.contains("level2"));
    assert!(!stdout.contains("level3"));
    assert!(!stdout.contains("deep.txt"));
}

/// 测试模式过滤功能
#[test]
fn test_pattern_filter() {
    let temp_dir = TempDir::new().unwrap();

    // 创建不同类型的文件
    fs::write(temp_dir.path().join("file1.txt"), "text content").unwrap();
    fs::write(temp_dir.path().join("file2.rs"), "rust code").unwrap();
    fs::write(temp_dir.path().join("script.py"), "python code").unwrap();

    // 过滤 Rust 文件
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd
        .args(["-P", "*.rs", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    if !output.status.success() {
        eprintln!("Command failed with status: {:?}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("file2.rs"));
    assert!(!stdout.contains("file1.txt"));
    assert!(!stdout.contains("script.py"));
}

/// 测试帮助信息
#[test]
fn test_help_option() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd.arg("--help").output().expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("tree-cli"));
    assert!(stdout.contains("显示所有文件") || stdout.contains("-a"));
    assert!(stdout.contains("启用彩色输出") || stdout.contains("-C"));
}

/// 测试版本信息
#[test]
fn test_version_option() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd.arg("--version").output().expect("Failed to execute tree-cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("tree-cli"));
}

/// 测试空目录
#[test]
fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    // 空目录应该能正常显示
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd.arg(temp_dir.path()).output().expect("Failed to execute tree-cli");

    if !output.status.success() {
        eprintln!("Command failed with status: {:?}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("0 files") || stdout.contains("0 file"));
}

/// 测试人类可读文件大小显示
#[test]
fn test_human_readable_size() {
    let temp_dir = TempDir::new().unwrap();

    // 创建不同大小的测试文件
    fs::write(temp_dir.path().join("small.txt"), "Hello").unwrap(); // 5 bytes
    let large_content = "x".repeat(2048); // 2KB
    fs::write(temp_dir.path().join("large.txt"), large_content).unwrap();

    // 不使用 -h 选项
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd.arg(temp_dir.path()).output().expect("Failed to execute tree-cli");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("[5B]"));
    assert!(!stdout.contains("[2.0KB]"));

    // 使用 -s 选项
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("tree-cli"));
    let output = cmd
        .args(["-s", temp_dir.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute tree-cli");

    if !output.status.success() {
        eprintln!("Command failed with status: {:?}", output.status);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("[5B]") || stdout.contains("["));
    assert!(stdout.contains("small.txt"));
    assert!(stdout.contains("large.txt"));
}
