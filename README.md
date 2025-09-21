# Audit build.rs files

Tool for auditing cargo build.rs files to help mitigate supply chain attacks.

The idea is simple; after running `cargo add|update` run `cargo audit-build` **before** running `cargo build|check|test|run|bench|doc|install|package`.

The tool will fetch dependencies and for each package found with a `build.rs` file open the `build.rs` file in your `EDITOR` so you can inspect it before it would be executed.

## Install

```
cargo install cargo-audit-build
```

## How it works

The program will ask you after viewing each `build.rs` file whether you trust it and your response is then stored in the `~/.cargo/audits/build-rs` folder so that you don't need to re-review build files that have already been trusted. Each `build.rs` file is committed to the audits repository to track the contents of the `build.rs` files that have been reviewed. 

The `~/.cargo/audits/build-rs` folder is a git repository so it can easily be shared between your machines and/or team members. 

## Requirements

* The `EDITOR` environment variable must be set.
* The `git` executable.

## Bugs

* Iterates all `build.rs` files even for targets that you may not intend to compile for.

## License

MIT or Apache-2.0
