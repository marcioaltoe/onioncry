<!-- setup-context-driven:begin id=guide.rust version=0.0.1 -->

# Rust

These rules govern the repository's Rust crates: the library and binary
targets of its Cargo package or workspace, and their tests. Code in another
language follows its own guide.

- **mandatory**: Change dependencies through Cargo commands such as `cargo add`, `cargo remove`, and `cargo update`, and commit `Cargo.toml` and `Cargo.lock` together.

- **mandatory**: Use current authoritative crate and toolchain documentation before changing Rust APIs, configuration, or dependencies. Follow the edition the crate's `Cargo.toml` declares, not the one a skill example assumes: under edition 2024, `std::env::set_var` and `std::env::remove_var` are `unsafe`, and each call needs an `unsafe` block whose `// SAFETY:` comment states why no other thread reads or writes the environment at that point.

- **mandatory**: Use focused `cargo check`, `cargo test`, and lint commands while iterating, then run the selected repository Verification.

- **mandatory**: Treat CLI flags, streams, JSON fields, and exit codes as public behavior and test them through observable command execution.

- **mandatory**: Keep each binary target's `main` thin: it parses arguments and calls behavior that lives in the library crate.

- **prohibited**: Do not use `anyhow` or another type-erased error type outside a binary's entry point. Only `main` and the command dispatch it calls may turn typed errors into a type-erased report.

- **prohibited**: Do not call `unwrap`, `expect`, or `panic!` on a path that user input, the environment, a file, or an IO result can reach; return a typed error instead. Test code is not such a path.

- **mandatory**: Return typed errors from library and domain code: an error enum per boundary that implements `std::error::Error`, for example through `thiserror`, so a caller can match its variants.

<!-- setup-context-driven:end id=guide.rust -->
