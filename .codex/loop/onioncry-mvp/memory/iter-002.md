# Iteration 002 Memory

## Objective Snapshot

Completed `task-002: Detect local import graph`.

## Important Decisions

- Added the Oxc parser stack through Cargo tooling: `oxc_parser`, `oxc_allocator`, `oxc_ast`, `oxc_ast_visit`, and `oxc_span`.
- Used `Parser::new(...).parse()` and `module_record.requested_modules` for static imports and re-exports.
- Used `oxc_ast_visit::Visit` for string-literal dynamic imports and string-literal `require(...)` calls.
- Non-literal dynamic imports and non-literal `require(...)` calls are ignored.
- Local import resolution is intentionally MVP-scoped: relative imports and configured aliases only; no tsconfig paths, package exports, package main fields, project references, or `.d.ts` probing.
- Unresolved local imports produce `onion/unresolved-import` warnings with file, import specifier, line, and column.

## Files / Surfaces Touched

- `Cargo.toml`
- `Cargo.lock`
- `src/lib.rs`
- `tests/check_cli.rs`

## Validation Evidence

- Initial task-002 integration test run failed because unresolved imports were not yet reported.
- `rtk cargo test --all-features` passed with 7 integration tests after implementation.
- First `rtk cargo fmt --all && rtk make verify` failed on an unused `Program` import.
- After removing the unused import, `rtk cargo fmt --all && rtk make verify` passed.
- Final verify output included successful conventional commit checks, clippy with no issues, and `cargo test: 7 passed`.

## Errors / Corrections

- Clippy correctly rejected an unused Oxc AST import under `-D warnings`; removed it and reran the full gate.

## Ready for Next Run

- Next tracked action should be `task-003: Enforce layer boundaries`.
- Task-003 can use `collect_import_edges`, `ImportEdge`, `ImportKind`, and `ImportResolution::Local` to compare local dependency edges against layer policy.
- `Violation` now has optional `importSpecifier`, `line`, and `column` fields.

## Open Blockers

None.
