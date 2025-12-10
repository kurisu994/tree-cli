//! 目录树生成模块
//!
//! 该模块负责生成和显示目录树结构，包括文件统计和格式化输出。

use globset::GlobMatcher;
use std::io;
use std::path::Path;

use crate::file_iterator::{FileItem, FileIterator};
use crate::filter::FilteredIterator;
use crate::symbol::{print_path, set_line_prefix};

/// 应用程序配置选项
pub struct Config {
    /// 是否启用彩色输出
    pub colorful: bool,
    /// 是否显示隐藏文件
    pub show_all: bool,
    /// 是否显示文件大小
    pub size: bool,
    /// 最大遍历深度
    pub max_level: usize,
    /// 文件过滤模式
    pub include_glob: Option<GlobMatcher>,
    /// 文件排除模式
    pub exclude_glob: Option<GlobMatcher>,
}

/// 目录树生成器，负责将文件系统结构转换为可视化的树形图
pub struct DirTree<'a> {
    /// 终端输出对象，用于彩色输出
    term: &'a mut Box<term::StdoutTerminal>,
    /// 配置选项
    config: Config,
}

impl<'a> DirTree<'a> {
    pub fn new(config: Config, term: &'a mut Box<term::StdoutTerminal>) -> DirTree<'a> {
        DirTree { config, term }
    }
    pub fn print_folders(&mut self, path: &Path) -> io::Result<DirSummary> {
        let mut summary = DirSummary::init();

        let mut symbol_switch_list: Vec<bool> = Vec::new();
        let mut prefix = String::new();

        for entry in self.get_iterator(path) {
            self.cal_symbol_switch(&mut symbol_switch_list, entry.level, entry.is_last);

            if entry.is_dir() {
                summary.num_folders += 1;
            } else {
                summary.num_files += 1;
            }

            set_line_prefix(&symbol_switch_list, &mut prefix);
            self.print_line(&entry, &prefix)?;
        }
        summary.num_folders = summary.num_folders.saturating_sub(1);
        Ok(summary)
    }

    fn cal_symbol_switch(&self, symbol_switch_list: &mut Vec<bool>, level: usize, is_last: bool) {
        while symbol_switch_list.len() > level {
            symbol_switch_list.pop();
        }
        if level > symbol_switch_list.len() {
            symbol_switch_list.push(true);
        }
        let levels_len = symbol_switch_list.len();
        if levels_len > 0 {
            symbol_switch_list[levels_len.saturating_sub(1)] = !is_last;
        }
    }

    fn get_iterator(&self, path: &Path) -> FilteredIterator {
        let list = FileIterator::new(path, &self.config);
        let mut list = FilteredIterator::new(list);
        if self.config.include_glob.is_none() {
            list.skip_filter();
        }
        list
    }

    fn print_line(&mut self, entry: &FileItem, prefix: &str) -> io::Result<()> {
        print!("{}", prefix);
        if let Ok(ref metadata) = entry.metadata {
            print_path(&entry.file_name, metadata, self.term, &self.config)?;
        } else {
            print!("{} [Error File]", entry.file_name);
        }
        println!();
        Ok(())
    }
}

pub struct DirSummary {
    pub num_folders: usize,
    pub num_files: usize,
}

impl DirSummary {
    pub fn init() -> DirSummary {
        DirSummary {
            num_folders: 0,
            num_files: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config {
            colorful: true,
            show_all: false,
            size: false,
            max_level: 3,
            include_glob: None,
            exclude_glob: None,
        };
        assert!(config.colorful);
        assert!(!config.show_all);
        assert!(!config.size);
        assert_eq!(config.max_level, 3);
        assert!(config.include_glob.is_none());
        assert!(config.exclude_glob.is_none());
    }

    #[test]
    fn test_dir_summary_init() {
        let summary = DirSummary::init();
        assert_eq!(summary.num_folders, 0);
        assert_eq!(summary.num_files, 0);
    }

    #[test]
    fn test_cal_symbol_switch_logic() {
        // 直接测试 cal_symbol_switch 的逻辑，不依赖终端

        // 模拟 DirTree::cal_symbol_switch 的核心逻辑
        let mut symbol_switch_list: Vec<bool> = Vec::new();

        // 测试逻辑：当 level > symbol_switch_list.len() 时，push true
        let level = 1;
        let is_last = true;

        while symbol_switch_list.len() > level {
            symbol_switch_list.pop();
        }
        if level > symbol_switch_list.len() {
            symbol_switch_list.push(true);
        }
        if let Some(last) = symbol_switch_list.last_mut() {
            *last = !is_last;
        }

        assert_eq!(symbol_switch_list.len(), 1);
        assert!(!symbol_switch_list[0]); // is_last = true, 所以应该是 false
    }
}
