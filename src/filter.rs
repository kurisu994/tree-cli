//! 文件过滤模块
//!
//! 该模块提供了文件过滤功能，可以根据配置过滤空目录和隐藏文件。

use std::collections::VecDeque;

use crate::file_iterator::{FileItem, FileIterator};

/// 过滤后的文件迭代器，提供额外的过滤功能
pub struct FilteredIterator {
    current: FileIterator,
    cache: VecDeque<FileItem>,
    skip: bool,
    next_item: Option<FileItem>,
}

impl FilteredIterator {
    pub fn new(iterator: FileIterator) -> Self {
        FilteredIterator {
            current: iterator,
            cache: VecDeque::new(),
            skip: false,
            next_item: None,
        }
    }

    pub fn skip_filter(&mut self) {
        self.skip = true;
    }

    fn remove_empty_directories_from_cache(&mut self, item: &FileItem) {
        while let Some(last) = self.cache.pop_back() {
            if last.level < item.level {
                self.cache.push_back(last);
                break;
            }
        }
    }
}

impl Iterator for FilteredIterator {
    type Item = FileItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.skip {
            return self.current.next();
        }
        if let Some(cache_item) = self.cache.pop_front() {
            return Some(cache_item);
        }
        if let Some(next_item) = self.next_item.take() {
            return Some(next_item);
        }
        while let Some(item) = self.current.next() {
            self.remove_empty_directories_from_cache(&item);

            if item.is_dir() {
                self.cache.push_back(item)
            } else {
                return if let Some(cache_front) = self.cache.pop_front() {
                    self.next_item = Some(item);
                    Some(cache_front)
                } else {
                    Some(item)
                };
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Config;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_filtered_iterator_new() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 2,
            include_glob: None,
        };

        let file_iterator = FileIterator::new(temp_dir.path(), &config);
        let filtered_iterator = FilteredIterator::new(file_iterator);

        assert!(!filtered_iterator.skip);
        assert_eq!(filtered_iterator.cache.len(), 0);
        assert!(filtered_iterator.next_item.is_none());
    }

    #[test]
    fn test_skip_filter() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 2,
            include_glob: None,
        };

        let file_iterator = FileIterator::new(temp_dir.path(), &config);
        let mut filtered_iterator = FilteredIterator::new(file_iterator);

        assert!(!filtered_iterator.skip);
        filtered_iterator.skip_filter();
        assert!(filtered_iterator.skip);
    }

    #[test]
    fn test_remove_empty_directories_from_cache() {
        let temp_dir = TempDir::new().unwrap();

        // 创建一些测试文件项（模拟）
        let file_item1 = FileItem::new(&temp_dir.path().join("dir1"), 1, true);
        let file_item2 = FileItem::new(&temp_dir.path().join("dir2"), 2, true);
        let file_item3 = FileItem::new(&temp_dir.path().join("file.txt"), 3, true); // 更高层级

        let config = Config {
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 2,
            include_glob: None,
        };

        let file_iterator = FileIterator::new(temp_dir.path(), &config);
        let mut filtered_iterator = FilteredIterator::new(file_iterator);

        // 添加一些项目到缓存
        filtered_iterator.cache.push_back(file_item1);
        filtered_iterator.cache.push_back(file_item2);

        // 当新项目层级较小时，应该保留缓存中的项目
        filtered_iterator.remove_empty_directories_from_cache(&file_item3);
        assert_eq!(filtered_iterator.cache.len(), 2);

        // 当新项目层级等于或大于缓存项目时，应该移除
        let file_item4 = FileItem::new(&temp_dir.path().join("dir3"), 1, true);
        filtered_iterator.remove_empty_directories_from_cache(&file_item4);
        // 应该移除所有 level >= 1 的项目
        assert_eq!(filtered_iterator.cache.len(), 0);
    }

    #[test]
    fn test_filtered_iterator_with_files() {
        let temp_dir = TempDir::new().unwrap();

        // 创建测试目录结构
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("subdir/file2.txt"), "content2").unwrap();

        let config = Config {
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 2,
            include_glob: None,
        };

        let file_iterator = FileIterator::new(temp_dir.path(), &config);
        let mut filtered_iterator = FilteredIterator::new(file_iterator);

        let mut items = Vec::new();
        while let Some(item) = filtered_iterator.next() {
            items.push(item);
        }

        // 应该至少有根目录
        assert!(!items.is_empty());

        // 验证文件类型
        let dirs: Vec<_> = items.iter().filter(|i| i.is_dir()).collect();
        let files: Vec<_> = items.iter().filter(|i| !i.is_dir()).collect();

        // 应该至少有一个目录
        assert!(!dirs.is_empty());
        // 应该有文件
        assert!(!files.is_empty());
    }

    #[test]
    fn test_filtered_iterator_skip_mode() {
        let temp_dir = TempDir::new().unwrap();

        // 创建测试文件
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.rs"), "content2").unwrap();

        let config = Config {
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 1,
            include_glob: None,
        };

        let file_iterator = FileIterator::new(temp_dir.path(), &config);
        let mut filtered_iterator = FilteredIterator::new(file_iterator);
        filtered_iterator.skip_filter();

        let mut items = Vec::new();
        while let Some(item) = filtered_iterator.next() {
            items.push(item);
        }

        // 在 skip 模式下，应该返回所有原始项目
        assert!(!items.is_empty());
    }

    #[test]
    fn test_filtered_iterator_empty_directory_handling() {
        let temp_dir = TempDir::new().unwrap();

        // 创建空目录
        fs::create_dir(temp_dir.path().join("empty_dir")).unwrap();
        // 创建有文件的目录
        fs::create_dir(temp_dir.path().join("nonempty_dir")).unwrap();
        fs::write(temp_dir.path().join("nonempty_dir/file.txt"), "content").unwrap();

        let config = Config {
            colorful: false,
            human_readable: false,
            show_all: false,
            max_level: 2,
            include_glob: None,
        };

        let file_iterator = FileIterator::new(temp_dir.path(), &config);
        let mut filtered_iterator = FilteredIterator::new(file_iterator);

        let mut items = Vec::new();
        while let Some(item) = filtered_iterator.next() {
            items.push(item);
        }

        // 过滤器应该正确处理空目录和非空目录
        let dir_names: Vec<String> = items.iter()
            .filter(|i| i.is_dir())
            .map(|i| i.file_name.clone())
            .collect();

        // 应该包含目录（根目录和可能的非空目录）
        assert!(!dir_names.is_empty());
    }
}
