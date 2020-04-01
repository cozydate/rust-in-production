# modules_and_binaries

Example Rust modules and binaries:
- [`binary_using_shared_modules`](src/bin/binary_using_shared_modules.rs)
- [`binary_using_local_package`](src/bin/binary_using_local_package.rs) uses `../package1`.
- [`binary_with_internal_modules/`](src/bin/binary_with_internal_modules/)
- [`lib.rs`](src/lib.rs) specifies library crate exports
- [`single_file_module`](src/single_file_module.rs)
- [`multi_file_module`](src/multi_file_module/)

More info:
- [Rust Lang Mailing List - How to refer to a sibling module from binary?](https://users.rust-lang.org/t/how-to-refer-to-a-sibling-module-from-binary/20929/2)
- [The Cargo Book - Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [The Cargo Book - Target auto-discovery](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#target-auto-discovery)
- [Rust by Example - Crates](https://doc.rust-lang.org/rust-by-example/crates.html)
- [Rust Lang Mailing List - How to use local unpublished crate](https://users.rust-lang.org/t/solved-how-to-use-local-unpublished-crate/25738/10)
