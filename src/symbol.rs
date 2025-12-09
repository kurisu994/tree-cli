//! 符号和颜色显示模块
//!
//! 该模块负责生成目录树的符号（如 ├── └── 等）和处理彩色输出。

use std::fs::Metadata;
use std::io;

use term::color;

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

/// 将字节转换为人类可读的格式
pub fn format_human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

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

pub fn print_path(
    file_name: &str,
    metadata: &Metadata,
    t: &mut Box<term::StdoutTerminal>,
    config: &Config,
) -> io::Result<()> {
    // 先打印文件名
    if metadata.is_dir() {
        write_color(t, config, color::BRIGHT_BLUE, file_name)?;
    } else if is_executable(metadata) {
        write_color(t, config, color::BRIGHT_RED, file_name)?;
    } else {
        write!(t, "{}", file_name)?;
    }

    // 如果启用人类可读格式且是文件，显示文件大小
    if config.human_readable && metadata.is_file() {
        let size = metadata.len();
        let size_str = format_human_readable_size(size);
        // 使用灰色显示文件大小
        write_color(t, config, color::BRIGHT_BLACK, &format!(" [{}]", size_str))?;
    }

    Ok(())
}

fn write_color(
    t: &mut Box<term::StdoutTerminal>,
    config: &Config,
    color: color::Color,
    str: &str,
) -> io::Result<()> {
    if config.colorful {
        t.fg(color)?;
    }
    write!(t, "{}", str)?;
    if config.colorful {
        t.reset()?;
    }
    Ok(())
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
    use tempfile::TempDir;
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_format_human_readable_size() {
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
    fn test_write_color_with_color_enabled() {
        let config = Config {
            colorful: true,
            human_readable: false,
            show_all: false,
            max_level: 1,
            include_glob: None,
        };

        // 注意：这个测试可能需要在有终端支持的环境中运行
        // 在 CI 环境中可能会失败，但逻辑是正确的
        if let Some(terminal) = term::stdout() {
            let result = write_color(&mut Box::new(terminal), &config, color::BRIGHT_BLUE, "test");
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_write_color_with_color_disabled() {
        let config = Config {
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 1,
            include_glob: None,
        };

        if let Some(terminal) = term::stdout() {
            let result = write_color(&mut Box::new(terminal), &config, color::BRIGHT_RED, "test");
            assert!(result.is_ok());
        }
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
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 1,
            include_glob: None,
        };

        let temp_dir = TempDir::new().unwrap();
        let metadata = temp_dir.path().metadata().unwrap();

        if let Some(terminal) = term::stdout() {
            let result = print_path("test_dir", &metadata, &mut Box::new(terminal), &config);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_print_path_regular_file() {
        let config = Config {
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 1,
            include_glob: None,
        };

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();
        let metadata = fs::metadata(&file_path).unwrap();

        if let Some(terminal) = term::stdout() {
            let result = print_path("test.txt", &metadata, &mut Box::new(terminal), &config);
            assert!(result.is_ok());
        }
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
