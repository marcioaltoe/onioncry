# Iteration 001 Memory

## Objective Snapshot

Completed `task-001: Bootstrap CLI check`.

## Important Decisions

- Scaffolded the first Rust binary crate with Cargo tooling.
- Used `clap` for CLI parsing, `jsonc-parser` for JSONC config loading, `globset` plus `walkdir` for the file universe, and `serde_json` for JSON output.
- Kept architecture rule evaluation out of this slice. `check` currently reports an empty violation set while preserving the result/status/reporting structure required by later tasks.
- Resolved `project.root` relative to the config file directory, so explicit config files can live outside the analyzed project root.

## Files / Surfaces Touched

- `Cargo.toml`
- `Cargo.lock`
- `src/lib.rs`
- `src/main.rs`
- `tests/check_cli.rs`

## Validation Evidence

- `rtk cargo test --all-features` passed with 5 integration tests.
- `rtk cargo fmt --all && rtk make verify` passed.
- `make verify` output included successful conventional commit checks, clippy with no issues, and `cargo test: 5 passed`.

## Errors / Corrections

- The first TDD run failed because Cargo's starter `Hello, world!` binary did not implement `check`; replacing it with the CLI path made the tests pass.
- Removed Cargo's duplicate `.gitignore` target entry after `cargo init` added it.

## Ready for Next Run

- Next tracked action should be `task-002: Detect local import graph`.
- Task-002 can build on `LoadedConfig`, `select_files`, `CheckReport`, and `Violation`.
- Loop verification status was intentionally left pending; the dedicated codex-loop `verify` action must record final PASS after all tasks complete.

## Open Blockers

None.
