use std::collections::VecDeque;
use std::fs::{DirEntry, Metadata};
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::Config;
use globset::GlobMatcher;

/// 表示文件系统中的一个文件项，包含路径、元数据和层级信息
#[derive(Debug)]
pub struct FileItem {
    /// 文件名
    pub file_name: String,
    /// 完整路径
    pub path: PathBuf,
    /// 文件元数据（可能读取失败）
    pub metadata: io::Result<Metadata>,
    /// 在目录树中的层级深度
    pub level: usize,
    /// 是否是同级目录中的最后一个项目
    pub is_last: bool,
}

impl FileItem {
    pub fn new(path: &Path, level: usize, is_last: bool) -> FileItem {
        let metadata = path.symlink_metadata();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .or_else(|| path.to_str())
            .unwrap_or("");

        FileItem {
            file_name: file_name.to_string(),
            path: path.to_owned(),
            metadata,
            level,
            is_last,
        }
    }

    pub fn is_dir(&self) -> bool {
        self.metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false)
    }
}

/// 文件系统迭代器，按照广度优先的顺序遍历目录树
#[derive(Debug)]
pub struct FileIterator {
    /// 待处理的文件项目队列
    queue: VecDeque<FileItem>,
    /// 是否显示隐藏文件
    show_hidden: bool,
    /// 最大遍历深度
    max_level: usize,
    /// 全局匹配器，用于过滤文件
    include_glob: Option<GlobMatcher>,
}

impl FileIterator {
    /// 创建新的文件迭代器
    ///
    /// # 参数
    /// * `path` - 要遍历的根目录路径
    /// * `config` - 配置选项
    pub fn new(path: &Path, config: &Config) -> FileIterator {
        let mut queue = VecDeque::new();
        queue.push_back(FileItem::new(path, 0, true));
        FileIterator {
            queue,
            max_level: config.max_level,
            show_hidden: config.show_all,
            include_glob: config.include_glob.clone(),
        }
    }

    fn is_glob_included(&self, file_name: &str) -> bool {
        if let Some(ref glob) = self.include_glob {
            glob.is_match(file_name)
        } else {
            true
        }
    }

    fn is_included(&self, name: &str, is_dir: bool) -> bool {
        if !self.show_hidden && name.starts_with('.') {
            return false;
        }
        if is_dir {
            true
        } else {
            self.is_glob_included(name)
        }
    }

    fn push_dir(&mut self, item: &FileItem) {
        let dir_entries = match fs::read_dir(&item.path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("错误：无法读取目录 {}：{}", item.path.display(), e);
                return;
            }
        };

        let mut dir_entries: Vec<DirEntry> = match dir_entries.collect() {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("错误：无法读取目录 {}：{}", item.path.display(), e);
                return;
            }
        };
        dir_entries.sort_by_key(|b| std::cmp::Reverse(b.file_name()));

        let mut entries: Vec<FileItem> = dir_entries
            .iter()
            .map(|e| FileItem::new(&e.path(), item.level + 1, false))
            .filter(|item| self.is_included(&item.file_name, item.is_dir()))
            .collect();

        if let Some(item) = entries.first_mut() {
            item.is_last = true;
        }

        for item in entries {
            self.queue.push_back(item);
        }
    }
}

impl Iterator for FileIterator {
    type Item = FileItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.queue.pop_back() {
            if item.is_dir() && item.level < self.max_level {
                self.push_dir(&item);
            }
            Some(item)
        } else {
            None
        }
    }
}
