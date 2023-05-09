# rust-elf-note-decl

A Rust experiment in embedding data into a binary. Heavily inspired by [noted](https://crates.io/crates/noted).

```sh
cargo build --examples
readelf -x .note.decl target/release/examples/example
cargo run -- target/release/examples/example
```
