//! 目录树生成模块
//!
//! 该模块负责生成和显示目录树结构，包括文件统计和格式化输出。

use std::io;
use std::path::Path;
use globset::GlobMatcher;

use crate::file_iterator::{FileItem, FileIterator};
use crate::filter::FilteredIterator;
use crate::symbol::{print_path, set_line_prefix};

/// 应用程序配置选项
pub struct Config {
    /// 是否启用彩色输出
    pub colorful: bool,
    /// 是否显示隐藏文件
    pub show_all: bool,
    /// 最大遍历深度
    pub max_level: usize,
    /// 文件过滤模式
    pub include_glob: Option<GlobMatcher>,
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
