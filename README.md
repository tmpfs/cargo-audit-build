# Audit build.rs files

Tool for auditing cargo build.rs files.

The idea is simple; after running `cargo add`, `cargo install` or `cargo update` run `cargo audit-build` before running `cargo build` or `cargo check`.

The tool will fetch dependencies and for each package found with a `build.rs` file open the `build.rs` file in your `EDITOR` so you can inspect it before it would be executed.

## Install

```
cargo install cargo-audit-build
```

## Requirements

* The `EDITOR` environment variable must be set

## License

MIT or Apache-2.0
