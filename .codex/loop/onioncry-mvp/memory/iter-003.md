# Iteration 003 Memory

## Objective Snapshot

Completed `task-003: Enforce layer boundaries`.

## Important Decisions

- Added `layers.*.patterns` configuration support with layer names mapped to explicit pattern sets and `mayImport` lists.
- Added layer classification for analyzed files. A file matching multiple layers reports `onion/ambiguous-layer`; a file matching no layer reports `onion/unclassified-file`.
- Enforced `onion/no-layer-leak` only for resolved local import edges whose source and target files each have exactly one layer classification.
- Kept type-only imports and re-exports as import edges so they count as architectural dependencies.
- Added rule-specific diagnostic context to pretty and JSON output: `fromLayer`, `toLayer`, `targetFile`, `suggestion`, `matchedLayers`, import specifier, file path, line, and column.
- Kept rule defaults scoped to this slice: `onion/no-layer-leak` defaults to error, while `onion/unclassified-file` defaults to warn.

## Files / Surfaces Touched

- `src/lib.rs`
- `tests/check_cli.rs`

## Validation Evidence

- Integration tests cover allowed layer imports, forbidden layer imports, type-only layer dependencies, re-export layer dependencies, ambiguous layer classification, and unclassified files.
- `rtk cargo test --all-features` passed with 10 tests.
- `rtk make verify` passed after implementation.
- Final verify output included conventional commit checks, clippy with no issues, and `cargo test: 10 passed`.

## Errors / Corrections

- A failing layer-boundary test showed that resolved paths containing `..` segments did not match layer glob patterns. Added path normalization after resolution so classification compares stable project-root-relative paths.

## Ready for Next Run

- Next tracked action should be `task-004: Apply linter-style rule policy`.
- Existing rule handling still uses slice-level defaults; task-004 should introduce the full linter-style policy engine for `off`, `warn`, `error`, `[severity, options]`, overrides, and final status calculation.

## Open Blockers

None.
