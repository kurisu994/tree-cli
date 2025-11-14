//! 符号和颜色显示模块
//!
//! 该模块负责生成目录树的符号（如 ├── └── 等）和处理彩色输出。

use std::fs::Metadata;
use std::io;

use term::color;

use crate::Config;

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
    if metadata.is_dir() {
        write_color(t, config, color::BRIGHT_BLUE, file_name)
    } else if is_executable(metadata) {
        write_color(t, config, color::BRIGHT_RED, file_name)
    } else {
        write!(t, "{}", file_name)
    }
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
