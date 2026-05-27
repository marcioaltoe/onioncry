# Iteration 009 Memory

## Objective Snapshot

Completed `task-009: Explain one file`.

## Important Decisions

- Added `onioncry explain <file>` as a top-level CLI subcommand.
- `explain` uses the same config discovery, `--config` handling, file universe selection, import graph, effective rule policy, layer classification, context classification, package policy, and violation generation paths as `check`.
- `explain` exits 0 even when the explained file has violations because it is diagnostic, not a CI gate.
- The command defaults to pretty output and supports `--format json`, matching the existing `check` output format style.
- Explain output reports layer status/name/matched patterns, context status/name/matched patterns, public surface status, imports, local target paths, external normalized package names, package allow decisions, unresolved local imports, and violations for the explained file.

## Files / Surfaces Touched

- `src/lib.rs`
- `src/main.rs`
- `tests/check_cli.rs`

## Validation Evidence

- Added integration coverage for a classified file with matched layer/context patterns, resolved local imports, external package allow/deny decisions, unresolved local imports, and multiple violations.
- Added integration coverage for an unclassified and contextless file, including external package reporting where package policy cannot be evaluated because no source layer is classified.
- The first targeted explain test run failed because `explain` was not yet a registered subcommand.
- `rtk cargo test explain_reports --test check_cli --all-features` passed with 2 explain tests.
- `rtk cargo test --test check_cli --all-features` passed with 19 tests.
- `rtk make verify` passed after cleanup.
- Final verify output included conventional commit checks, clippy with no issues, and `cargo test: 19 passed`.

## Errors / Corrections

- The first `rtk make verify` run after implementation failed on clippy `needless_borrow` warnings in the shared violation collector introduced while refactoring `check` and `explain` to share analysis. Removed the stale borrows and reran the full gate.

## Ready for Next Run

- All implementation tasks are complete.
- Next tracked action should be `verify`; run `rtk make verify` and record PASS evidence through codex-loop tracking if it passes.

## Open Blockers

None.
