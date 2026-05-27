# Codex Loop Memory: onioncry-mvp

## Objective Snapshot

OnionCry MVP tracked loop has all implementation tasks complete and repository verification has passed. Final done validation is the remaining tracking check.

## Important Decisions

- The crate is now scaffolded as the `onioncry` Rust binary.
- `onioncry check` supports default `.onioncryrc.jsonc` discovery, `--config`, `--format pretty|json`, and `--fail-on error|warning`.
- JSONC configuration loading uses `jsonc-parser` with serde.
- File universe selection uses configured include/exclude patterns before rule evaluation.
- `project.root` is resolved relative to the config file directory.
- The local import graph uses `oxc_parser` for JS/TS parsing, `module_record.requested_modules` for static imports/re-exports, and an Oxc AST visitor for dynamic imports and `require(...)`.
- MVP local resolution covers relative imports and configured aliases, trying common TS/JS extensions and `index.*` files. It intentionally does not read tsconfig paths, package exports, package main fields, project references, or `.d.ts` files.
- `onion/unresolved-import` currently reports as a warning for unresolved relative and configured-alias imports only.
- Layer classification uses configured `layers.*.patterns`; ambiguous layer matches report `onion/ambiguous-layer`, unclassified analyzed files report `onion/unclassified-file`, and resolved local layer edges are checked against the source layer's explicit `mayImport` list.
- Type-only imports and re-exports count as architectural dependencies for `onion/no-layer-leak`.
- Unclassified or ambiguously classified files are reported, but they do not participate in `onion/no-layer-leak`.
- Rule policy now validates known `onion/...` rule names and accepts severity strings or `[severity, options]` values.
- Effective rule policy applies per file: base config first, then matching overrides in array order, with the last matching override replacing the whole rule value.
- Overrides only affect rule severity/options for already selected files; include/exclude still define the file universe first.
- `onion/no-forbidden-imports` checks only external package import edges from classified source layers.
- External package policy normalizes package names before matching: `uuid/v4` becomes `uuid`, and `@scope/pkg/subpath` becomes `@scope/pkg`.
- External package allowlists support exact package names and glob patterns. Layer-specific entries use `fromLayer`, `severity`, and `allow`, with later matching entries replacing the effective layer policy.
- Local aliases resolve as local or unresolved local imports and are not checked by the external package allowlist.
- Context classification uses configured `contexts.*.patterns`; files matching no context are allowed, while ambiguous context matches report `onion/ambiguous-context`.
- `onion/no-cross-context-internal-import` checks resolved local edges only when both source and target have exactly one context.
- `contextRules.default.allowSameContext` allows same-context imports, and `allowCrossContext` names public surface path segments such as `contracts`, `events`, `ports`, and `shared`.
- Global shared files are contextless unless explicitly listed under `contexts`, so `shared` is not treated as a context by default.
- `onion/circular-dependency` detects cycles in resolved local import edges within the selected file universe. External imports, unresolved imports, and resolved local targets outside the file universe are ignored.
- Circular dependency diagnostics use a canonical cycle whose first path is the lexicographically smallest file in the cycle and report a readable `cyclePath`.
- The default circular dependency severity is `warn`, and overrides apply to the canonical reported source file.
- `onioncry init` creates `.onioncryrc.jsonc` in the current working directory, refuses to overwrite by default, and supports `--force`.
- The init template is commented JSONC with full-line `// TODO` comments and contains `$schema`, `version`, `project`, `aliases`, `layers`, `contexts`, `contextRules`, `rules`, and `overrides`.
- The init template uses the default domain/application/infra/shared layer vocabulary and default MVP rule preset.
- `onioncry explain <file>` uses the same config discovery/loading, file universe, import graph, rule policy, classification, package policy, and violation paths as `check`.
- `explain` exits 0 even when the explained file has violations. It supports pretty output by default and `--format json` for machine-readable diagnostics.
- Explain reports layer/context classification, matched patterns, public surface status, imports, external package policy decisions, unresolved local imports, and violations for the explained file.

## Learnings

- `jsonc-parser` 0.32.4 supports direct serde deserialization through `parse_to_serde_value` when the `serde` feature is enabled.
- Oxc `ParserReturn` exposes `module_record`, including requested modules and dynamic imports; the AST visitor gives direct hooks for `ImportExpression` and `CallExpression`.
- Resolved paths must be normalized after joining relative imports; without normalization, paths containing `..` can miss configured layer glob patterns.

## Files / Surfaces

- `Cargo.toml`
- `Cargo.lock`
- `src/lib.rs`
- `src/main.rs`
- `tests/check_cli.rs`
- Iteration 002 additionally touched `src/lib.rs`, `tests/check_cli.rs`, `Cargo.toml`, and `Cargo.lock`.
- Iteration 003 touched `src/lib.rs` and `tests/check_cli.rs`.
- Iteration 004 touched `src/lib.rs` and `tests/check_cli.rs`.
- Iteration 005 touched `src/lib.rs` and `tests/check_cli.rs`.
- Iteration 006 touched `src/lib.rs` and `tests/check_cli.rs`.
- Iteration 007 touched `src/lib.rs` and `tests/check_cli.rs`.
- Iteration 008 touched `src/lib.rs`, `src/main.rs`, and `tests/check_cli.rs`.
- Iteration 009 touched `src/lib.rs`, `src/main.rs`, and `tests/check_cli.rs`.

## Validation Evidence

- Iteration 001: `rtk cargo test --all-features` passed with 5 tests.
- Iteration 001: `rtk cargo fmt --all && rtk make verify` passed, including conventional commit checks, clippy, and tests.
- Iteration 002: `rtk cargo test --all-features` passed with 7 tests.
- Iteration 002: final `rtk cargo fmt --all && rtk make verify` passed, including conventional commit checks, clippy, and tests.
- Iteration 003: `rtk cargo test --all-features` passed with 10 tests.
- Iteration 003: final `rtk make verify` passed, including conventional commit checks, clippy with no issues, and `cargo test: 10 passed`.
- Iteration 004: `rtk cargo test --test check_cli --all-features` passed with 12 tests.
- Iteration 004: final `rtk make verify` passed, including conventional commit checks, clippy with no issues, and `cargo test: 12 passed`.
- Iteration 005: `rtk cargo test --test check_cli --all-features` passed with 13 tests.
- Iteration 005: final `rtk make verify` passed, including conventional commit checks, clippy with no issues, and `cargo test: 13 passed`.
- Iteration 006: `rtk cargo test --test check_cli --all-features` passed with 14 tests.
- Iteration 006: final `rtk make verify` passed, including conventional commit checks, clippy with no issues, and `cargo test: 14 passed`.
- Iteration 007: `rtk cargo test --test check_cli --all-features` passed with 15 tests.
- Iteration 007: final `rtk make verify` passed, including conventional commit checks, clippy with no issues, and `cargo test: 15 passed`.
- Iteration 008: `rtk cargo test --test check_cli --all-features` passed with 17 tests.
- Iteration 008: final `rtk make verify` passed, including conventional commit checks, clippy with no issues, and `cargo test: 17 passed`.
- Iteration 009: `rtk cargo test --test check_cli --all-features` passed with 19 tests.
- Iteration 009: final `rtk make verify` passed, including conventional commit checks, clippy with no issues, and `cargo test: 19 passed`.
- Iteration 010: final loop `rtk make verify` passed, including conventional commit checks, clippy with no issues, and `cargo test: 19 passed`.

## Errors / Corrections

- Initial TDD run failed against Cargo's starter binary; implementation replaced it.
- Iteration 002 first verify run failed on an unused Oxc import; removing it made the full gate pass.
- Iteration 003 initially found resolved paths with `..` segments were not matching layer globs; adding path normalization made the observable layer checks pass.
- Iteration 004 first verify run failed on the obsolete `severity_from_rule_value` helper under `-D warnings`; removing the stale helper made the full gate pass.
- Iteration 005 first external-policy test failed because external imports were not evaluated yet; implementation added the rule check. The first verify run then failed on a needless borrow from the edge refactor; removing it made the full gate pass.
- Iteration 006 first bounded-context test failed because context configuration was ignored; adding context classification and public-surface checks made the observable behavior pass.
- Iteration 007 first circular-dependency test failed with zero cycle violations; adding resolved-local graph cycle detection made the observable behavior pass.
- Iteration 008 first init tests failed because `init` was not a registered subcommand; adding the CLI command and config writer made the observable behavior pass.
- Iteration 009 first explain tests failed because `explain` was not a registered subcommand. After implementation, the first verify run failed on needless borrows in the shared violation collector; removing those stale borrows made the full gate pass.

## Ready for Next Run

- Resume at the loop `done` action.
- `validate-tracking.py onioncry-mvp --expect-done` should pass after verification is recorded in state.
- `Violation` supports optional `importSpecifier`, `line`, `column`, `fromLayer`, `toLayer`, `targetFile`, `suggestion`, and `matchedLayers` fields for rule diagnostics.
- `Violation` also supports optional `packageName` for external package policy diagnostics.
- `Violation` also supports optional `fromContext`, `toContext`, and `matchedContexts` for bounded context diagnostics.
- `Violation` also supports optional `cyclePath` for circular dependency diagnostics.

## Open Blockers

None.
