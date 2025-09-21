# Audit build.rs files

Tool for auditing cargo build.rs files.

The idea is simple; after running `cargo add`, `cargo install` or `cargo update` run `cargo audit-build` before running `cargo build` or `cargo check`.

The tool will fetch dependencies and for each package found with a `build.rs` file open the `build.rs` file in your `EDITOR` so you can inspect it before it would be executed.

## Install

```
cargo install cargo-audit-build
```

## How it works

The program will ask you after viewing each `build.rs` file whether you trust it and your response is then stored in the `~/.cargo/audits/build-rs` folder so that you don't need to re-review build files that have already been trusted. Each `build.rs` file is committed to the audits repository to track the contents of the `build.rs` files that have been reviewed. 

## Requirements

* The `EDITOR` environment variable must be set.
* The `git` executable.

## License

MIT or Apache-2.0
