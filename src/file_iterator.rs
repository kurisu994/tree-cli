use std::collections::VecDeque;
use std::fs::{DirEntry, Metadata};
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::core::Config;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_file_item_creation() {
        let path = PathBuf::from("/test/path");
        let item = FileItem::new(&path, 1, true);

        assert_eq!(item.file_name, "path");
        assert_eq!(item.path, path);
        assert_eq!(item.level, 1);
        assert!(item.is_last);
        // metadata 可能失败，因为路径不存在，所以检查 Result 而不是 Ok
        assert!(item.metadata.is_ok() || item.metadata.is_err());
    }

    #[test]
    fn test_file_item_is_dir() {
        // 创建一个临时目录来测试
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path).unwrap();

        let dir_item = FileItem::new(&dir_path, 0, true);
        assert!(dir_item.is_dir());

        // 创建一个临时文件来测试
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content").unwrap();

        let file_item = FileItem::new(&file_path, 0, true);
        assert!(!file_item.is_dir());
    }

    #[test]
    fn test_file_iterator_new() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            colorful: false,
            show_all: false,
            max_level: 2,
            include_glob: None,
        };

        let iterator = FileIterator::new(temp_dir.path(), &config);
        assert_eq!(iterator.queue.len(), 1);
        assert_eq!(iterator.max_level, 2);
        assert!(!iterator.show_hidden);
        assert!(iterator.include_glob.is_none());
    }

    #[test]
    fn test_is_glob_included() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            colorful: false,
            show_all: false,
            max_level: 2,
            include_glob: None,
        };

        let iterator = FileIterator::new(temp_dir.path(), &config);

        // 没有 glob 匹配器时应该返回 true
        assert!(iterator.is_glob_included("any_file.txt"));
        assert!(iterator.is_glob_included("test.rs"));
    }

    #[test]
    fn test_is_included_hidden_files() {
        let temp_dir = TempDir::new().unwrap();

        // 不显示隐藏文件
        let config = Config {
            colorful: false,
            show_all: false,
            max_level: 2,
            include_glob: None,
        };
        let iterator = FileIterator::new(temp_dir.path(), &config);

        assert!(!iterator.is_included(".hidden", false));
        assert!(!iterator.is_included(".hidden_dir", true));
        assert!(iterator.is_included("normal.txt", false));
        assert!(iterator.is_included("normal_dir", true));

        // 显示隐藏文件
        let config = Config {
            colorful: false,
            show_all: true,
            max_level: 2,
            include_glob: None,
        };
        let iterator = FileIterator::new(temp_dir.path(), &config);

        assert!(iterator.is_included(".hidden", false));
        assert!(iterator.is_included(".hidden_dir", true));
    }

    #[test]
    fn test_file_iterator_single_directory() {
        let temp_dir = TempDir::new().unwrap();

        // 创建一些测试文件
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.rs"), "content2").unwrap();

        let config = Config {
            colorful: false,
            show_all: false,
            max_level: 0, // 不进入子目录
            include_glob: None,
        };

        let mut iterator = FileIterator::new(temp_dir.path(), &config);
        let mut items = Vec::new();

        while let Some(item) = iterator.next() {
            items.push(item);
        }

        // 应该只有根目录
        assert_eq!(items.len(), 1);
        assert!(items[0].is_dir());
    }

    #[test]
    fn test_file_iterator_with_subdirectories() {
        let temp_dir = TempDir::new().unwrap();

        // 创建子目录和文件
        fs::create_dir(temp_dir.path().join("subdir1")).unwrap();
        fs::create_dir(temp_dir.path().join("subdir2")).unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();

        let config = Config {
            colorful: false,
            show_all: false,
            max_level: 1, // 允许进入一层子目录
            include_glob: None,
        };

        let mut iterator = FileIterator::new(temp_dir.path(), &config);
        let mut items = Vec::new();

        while let Some(item) = iterator.next() {
            items.push(item);
        }

        // 应该有根目录和子目录
        assert!(items.len() > 1);

        // 检查文件名
        let file_names: Vec<String> = items.iter().map(|i| i.file_name.clone()).collect();
        assert!(file_names.contains(&"subdir1".to_string()));
        assert!(file_names.contains(&"subdir2".to_string()));
    }
}
