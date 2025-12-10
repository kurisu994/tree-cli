//! tree-cli - 高性能目录树显示工具
//!
//! 这是一个跨平台的命令行工具，用于以树形结构显示目录内容。
//! 它是 Unix `tree` 命令的轻量级替代方案。

use std::path::Path;
use std::io::Write;

use clap::Parser;
use globset::Glob;

use tree_cli::core::{DirSummary, DirTree, Config};

/// 高性能目录树显示工具
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author)]
struct Args {
    /// Show all files (include hidden files)
    #[arg(short = 'a', long = "all")]
    show_all: bool,
    /// Turn colorization on always
    #[arg(short = 'C', long = "color")]
    color_on: bool,
    /// Turn colorization off always
    #[arg(short = 'N', long = "no-color")]
    color_off: bool,
    /// Print the size of each file in human readable format
    #[arg(short = 's', long = "human-readable")]
    size: bool,
    /// Directory you want to search
    #[arg(value_name = "DIR", default_value = ".")]
    dir: String,
    /// List only those files matching <include_pattern>
    #[arg(short = 'P', long = "pattern")]
    include_pattern: Option<String>,
    /// Exclude those files matching <exclude_pattern>
    #[arg(short = 'E', long = "exclude")]
    exclude_pattern: Option<String>,
    /// Descend only <level> directories deep
    #[arg(short = 'L', long = "level", default_value_t = usize::max_value())]
    max_level: usize,
}

fn main() {
    let Args {
        show_all,
        color_on,
        color_off,
        size,
        dir,
        include_pattern,
        exclude_pattern,
        max_level,
    } = Args::parse();
    let path = Path::new(&dir);
    let mut mt = term::stdout().expect("Could not unwrap term::stdout.");
    let config = Config {
        colorful: color_on || !color_off,
        show_all,
        size,
        max_level,
        include_glob: include_pattern.map(|pat| {
            Glob::new(pat.as_str())
                .expect("include_pattern is not valid")
                .compile_matcher()
        }),
        exclude_glob: exclude_pattern.map(|pat| {
            Glob::new(pat.as_str())
                .expect("exclude_pattern is not valid")
                .compile_matcher()
        }),
    };
    let mut dir_tree = DirTree::new(config, &mut mt);
    let DirSummary {
        num_folders,
        num_files,
    } = dir_tree.print_folders(path).expect("execution failure");

    writeln!(mt, "\n{} directories, {} files", num_folders, num_files).unwrap()
}