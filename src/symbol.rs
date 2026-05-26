//! 符号和颜色显示模块
//!
//! 该模块负责生成目录树的符号（如 ├── └── 等）和处理彩色输出。

use std::borrow::Cow;
use std::fs::Metadata;
use std::io::{self, Write};

use anstyle::{AnsiColor, Style};

use crate::core::Config;

/// 横线符号 (─)
pub const HOR: char = '─';
/// 分支符号 (├)
pub const CRO: char = '├';
/// 垂直线符号 (│)
pub const VER: char = '│';
/// 末尾符号 (└)
pub const END: char = '└';
/// 空格符号
pub const SPACE: char = ' ';

/// 过滤文件名中的控制字符，防止 ANSI/终端转义序列注入
///
/// Unix 文件名可包含除 `/` 和 NUL 外的任意字节（含 ESC 等控制字符）。
/// 若把这类文件名原样写入终端，攻击者可借转义序列伪造或隐藏输出。
/// 此处将所有控制字符（C0、DEL、C1）替换为 `?`，对齐 GNU `ls` 的默认行为。
/// 无控制字符时返回原始引用，零额外分配。
pub fn sanitize_file_name(name: &str) -> Cow<'_, str> {
    if name.chars().any(char::is_control) {
        Cow::Owned(name.chars().map(|c| if c.is_control() { '?' } else { c }).collect())
    } else {
        Cow::Borrowed(name)
    }
}

/// 将字节转换为人类可读的格式
pub fn format_human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB", "EB"];

    if bytes == 0 {
        return "0B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    // 根据单位决定小数位数
    let formatted = if unit_index == 0 {
        format!("{}", bytes)
    } else if size < 10.0 {
        format!("{:.1}", size)
    } else {
        format!("{:.0}", size)
    };

    format!("{}{}", formatted, UNITS[unit_index])
}

pub fn set_line_prefix(symbol_switch_list: &[bool], prefix: &mut String) {
    let len = symbol_switch_list.len();
    let index = len.saturating_sub(1);
    prefix.clear();
    for symbol_switch in symbol_switch_list.iter().take(index) {
        if *symbol_switch {
            prefix.push(VER);
        } else {
            prefix.push(SPACE);
        }
        prefix.push(SPACE);
        prefix.push(SPACE);
        prefix.push(SPACE);
    }
    if let Some(symbol_switch) = symbol_switch_list.last() {
        if *symbol_switch {
            prefix.push(CRO);
        } else {
            prefix.push(END);
        }
        prefix.push(HOR);
        prefix.push(HOR);
        prefix.push(SPACE);
    }
}

pub fn print_path<W: Write>(file_name: &str, metadata: &Metadata, out: &mut W, config: &Config) -> io::Result<()> {
    // 过滤控制字符，防止终端转义序列注入
    let file_name = sanitize_file_name(file_name);
    let file_name = file_name.as_ref();
    // 先打印文件名
    if metadata.is_dir() {
        write_color(out, AnsiColor::BrightBlue, file_name)?;
    } else if is_executable(metadata) {
        write_color(out, AnsiColor::BrightRed, file_name)?;
    } else {
        write!(out, "{}", file_name)?;
    }

    // 如果启用大小显示且是文件，显示文件大小
    if config.size && metadata.is_file() {
        let size = metadata.len();
        let size_str = format_human_readable_size(size);
        // 使用灰色显示文件大小
        write_color(out, AnsiColor::BrightBlack, &format!(" [{}]", size_str))?;
    }

    Ok(())
}

/// 写入带前景色的文本
///
/// 始终写出 ANSI 颜色序列；是否真正着色由上层的 `anstream::AutoStream`
/// 根据 `ColorChoice`（是否 TTY、`--color`/`--no-color`、`NO_COLOR` 等）决定，
/// 非彩色场景下转义码会被自动剥离。
fn write_color<W: Write>(out: &mut W, color: AnsiColor, s: &str) -> io::Result<()> {
    let style = Style::new().fg_color(Some(color.into()));
    write!(out, "{style}{s}{style:#}")
}

#[cfg(target_os = "windows")]
fn is_executable(_metadata: &Metadata) -> bool {
    // Windows 平台暂时不支持可执行文件检测
    // 可以通过文件扩展名来判断，但这里简化为返回 false
    false
}

// 针对 Unix 系统（Linux 和 macOS）
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn is_executable(metadata: &Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn is_executable(metadata: &Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    #[test]
    fn test_format_human_readable_size() {
        // 基本测试
        assert_eq!(format_human_readable_size(0), "0B");
        assert_eq!(format_human_readable_size(512), "512B");
        assert_eq!(format_human_readable_size(1024), "1.0KB");
        assert_eq!(format_human_readable_size(1536), "1.5KB");
        assert_eq!(format_human_readable_size(1024 * 1024), "1.0MB");
        assert_eq!(format_human_readable_size(1024 * 1024 * 1024), "1.0GB");
        assert_eq!(format_human_readable_size(10 * 1024), "10KB");
        assert_eq!(format_human_readable_size(10240), "10KB");
    }

    #[test]
    fn test_format_human_readable_size_boundary_values() {
        // 边界值测试
        assert_eq!(format_human_readable_size(1), "1B");
        assert_eq!(format_human_readable_size(1023), "1023B");
        assert_eq!(format_human_readable_size(1024), "1.0KB");

        // 测试单位转换边界 - 函数会进行四舍五入
        assert_eq!(format_human_readable_size(1024 * 1024 - 1), "1024KB"); // 1048575 bytes = 1024KB (四舍五入)
        assert_eq!(format_human_readable_size(1024 * 1024), "1.0MB");
        assert_eq!(format_human_readable_size(1024 * 1024 * 1024 - 1), "1024MB"); // 1073741823 bytes = 1024MB (四舍五入)
        assert_eq!(format_human_readable_size(1024 * 1024 * 1024), "1.0GB");

        // 测试大数值
        assert_eq!(format_human_readable_size(u64::MAX), "16EB");

        // 测试小数格式化
        assert_eq!(format_human_readable_size(2048), "2.0KB");
        assert_eq!(format_human_readable_size(9216), "9.0KB");
        assert_eq!(format_human_readable_size(10240), "10KB");
        assert_eq!(format_human_readable_size(11264), "11KB");

        // 测试TB和PB
        let tb = 1024_u64.pow(4);
        let pb = 1024_u64.pow(5);
        assert_eq!(format_human_readable_size(tb), "1.0TB");
        assert_eq!(format_human_readable_size(pb), "1.0PB");

        // 测试特殊边界值
        assert_eq!(format_human_readable_size(10239), "10.0KB"); // 四舍五入到10.0KB
        assert_eq!(format_human_readable_size(10240), "10KB"); // 整好10KB
    }

    #[test]
    fn test_sanitize_file_name() {
        // 普通文件名（含中文/Unicode）原样返回，且零分配（借用）
        assert!(matches!(sanitize_file_name("normal.txt"), Cow::Borrowed(_)));
        assert!(matches!(sanitize_file_name("文档.md"), Cow::Borrowed(_)));
        assert_eq!(sanitize_file_name("normal.txt"), "normal.txt");

        // ESC 及 ANSI 转义序列被替换为 ?
        assert_eq!(sanitize_file_name("evil\x1b[31m.sh"), "evil?[31m.sh");
        // 回车/换行/制表符等控制字符被替换
        assert_eq!(sanitize_file_name("a\r\nb\tc"), "a??b?c");
        // DEL (0x7F) 被替换
        assert_eq!(sanitize_file_name("x\x7fy"), "x?y");
        // 含控制字符时返回 Owned
        assert!(matches!(sanitize_file_name("a\x1bb"), Cow::Owned(_)));
    }

    #[test]
    fn test_symbol_constants() {
        assert_eq!(HOR, '─');
        assert_eq!(CRO, '├');
        assert_eq!(VER, '│');
        assert_eq!(END, '└');
        assert_eq!(SPACE, ' ');
    }

    #[test]
    fn test_set_line_prefix_empty_list() {
        let symbol_switch_list: Vec<bool> = Vec::new();
        let mut prefix = String::new();
        set_line_prefix(&symbol_switch_list, &mut prefix);
        assert_eq!(prefix, "");
    }

    #[test]
    fn test_set_line_prefix_single_true() {
        let symbol_switch_list = vec![true];
        let mut prefix = String::new();
        set_line_prefix(&symbol_switch_list, &mut prefix);
        assert_eq!(prefix, "├── ");
    }

    #[test]
    fn test_set_line_prefix_single_false() {
        let symbol_switch_list = vec![false];
        let mut prefix = String::new();
        set_line_prefix(&symbol_switch_list, &mut prefix);
        assert_eq!(prefix, "└── ");
    }

    #[test]
    fn test_set_line_prefix_multiple_levels() {
        let symbol_switch_list = vec![true, false, true];
        let mut prefix = String::new();
        set_line_prefix(&symbol_switch_list, &mut prefix);
        // 前两个符号: │   (level 0: true),    (level 1: false), 最后一个: ├── (level 2: true, 但不是最后)
        assert_eq!(prefix, "│       ├── ");
    }

    #[test]
    fn test_set_line_prefix_mixed_patterns() {
        // 测试常见的树形结构模式
        let patterns = vec![
            (vec![true, true], "│   ├── "),
            (vec![true, false], "│   └── "),
            (vec![false, true], "    ├── "),
            (vec![false, false], "    └── "),
        ];

        for (input, expected) in patterns {
            let mut prefix = String::new();
            set_line_prefix(&input, &mut prefix);
            assert_eq!(prefix, expected, "Failed for input: {:?}", input);
        }
    }

    #[test]
    fn test_write_color_writes_text_and_ansi() {
        // 写入裸 Vec（非 AutoStream），ANSI 序列不会被剥离，可直接断言
        let mut buf: Vec<u8> = Vec::new();
        write_color(&mut buf, AnsiColor::BrightBlue, "test").unwrap();
        let s = String::from_utf8(buf).unwrap();

        // 文本本身存在
        assert!(s.contains("test"));
        // 含 ANSI SGR 起始序列与 reset
        assert!(s.contains('\x1b'));
        assert!(s.ends_with("\x1b[0m"));
    }

    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn test_is_executable_unix() {
        let temp_dir = TempDir::new().unwrap();

        // 创建普通文件
        let file_path = temp_dir.path().join("regular.txt");
        fs::write(&file_path, "content").unwrap();
        let metadata = fs::metadata(&file_path).unwrap();
        assert!(!is_executable(&metadata));

        // 创建可执行文件
        let exec_path = temp_dir.path().join("executable.sh");
        fs::write(&exec_path, "#!/bin/bash\necho test").unwrap();
        // 设置执行权限
        let mut perms = fs::metadata(&exec_path).unwrap().permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(&exec_path, perms).unwrap();

        let exec_metadata = fs::metadata(&exec_path).unwrap();
        assert!(is_executable(&exec_metadata));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_is_executable_windows() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.exe");
        fs::write(&file_path, "fake exe").unwrap();
        let metadata = fs::metadata(&file_path).unwrap();

        // Windows 版本总是返回 false
        assert!(!is_executable(&metadata));
    }

    #[test]
    fn test_print_path_directory() {
        let config = Config {
            size: false,
            show_all: false,
            max_level: 1,
            include_glob: None,
            exclude_glob: None,
        };

        let temp_dir = TempDir::new().unwrap();
        let metadata = temp_dir.path().metadata().unwrap();

        let mut buf: Vec<u8> = Vec::new();
        print_path("test_dir", &metadata, &mut buf, &config).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("test_dir"));
    }

    #[test]
    fn test_print_path_regular_file() {
        let config = Config {
            size: false,
            show_all: false,
            max_level: 1,
            include_glob: None,
            exclude_glob: None,
        };

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        let metadata = fs::metadata(&file_path).unwrap();

        let mut buf: Vec<u8> = Vec::new();
        print_path("test.txt", &metadata, &mut buf, &config).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("test.txt"));
    }

    #[test]
    fn test_complex_tree_structure_prefixes() {
        // 测试更复杂的树形结构
        let test_cases = vec![
            // 格式: (symbol_switch_list, expected_prefix)
            (vec![true, true, true, false], "│   │   │   └── "),
            (vec![false, true, false, true], "    │       ├── "),
            (vec![true, false, false, false], "│           └── "),
            (vec![false], "└── "),
        ];

        for (input, expected) in test_cases {
            let mut prefix = String::new();
            set_line_prefix(&input, &mut prefix);
            assert_eq!(prefix, expected, "Failed for input: {:?}", input);
        }
    }
}
