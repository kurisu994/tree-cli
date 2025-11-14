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
