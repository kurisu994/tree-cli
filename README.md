# tree-cli
a simple cross-platform alternative to the unix `tree` command.

## Usage
`cargo build --release`

## Example 
```shell
./target/release/tree-cli

.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── config
│   └── config.toml
├── rustfmt.toml
└── src
    ├── core.rs
    ├── file_iterator.rs
    ├── filter.rs
    ├── main.rs
    └── symbol.rs

3 directories, 11 files
```

## Install 
`cargo install --path .` 

or 

`cargo install --git  https://github.com/kurisu994/tree-cli.git`