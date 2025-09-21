# Audit build.rs files

Tool for auditing cargo build.rs files.

The idea is simple; before running `cargo install` or `cargo update` first run `cargo build-audit` which will fetch dependencies and for each package found with a `build.rs` file open the `build.rs` file in your `EDITOR` so you can inspect it before it would be executed.

## Install

```
cargo install cargo-build-audit
```

## Requirements

* The `EDITOR` environment variable must be set

## License

MIT or Apache-2.0
