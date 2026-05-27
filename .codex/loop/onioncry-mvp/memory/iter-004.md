# Iteration 004 Memory

## Objective Snapshot

Completed `task-004: Apply linter-style rule policy`.

## Important Decisions

- Added an effective rule policy engine for linter-style `onion/...` rules.
- Rule values now accept severity strings (`off`, `warn`, `error`) and `[severity, options]` arrays.
- Unknown rule names and invalid severities now fail config loading with clear CLI errors.
- Overrides match project-root-relative file globs and apply in array order; the last matching override replaces the whole rule value for that file.
- Overrides affect only rule severity/options. Project include/exclude still defines the file universe, and overrides do not alter aliases, layers, or contexts.
- Current checks now ask the effective rule policy per source file before emitting `onion/unresolved-import`, `onion/unclassified-file`, `onion/ambiguous-layer`, and `onion/no-layer-leak`.

## Files / Surfaces Touched

- `src/lib.rs`
- `tests/check_cli.rs`

## Validation Evidence

- Added integration coverage for base rule severity, `[severity, options]` rule values, disabled rules, override replacement, last matching override wins, override isolation from file selection, and `--fail-on warning`.
- Added integration coverage for unknown rule names and invalid severities.
- The first targeted policy test failed before implementation because overrides were ignored and both layer leaks remained errors.
- `rtk cargo test --test check_cli --all-features` passed with 12 tests.
- `rtk make verify` passed after cleanup.
- Final verify output included conventional commit checks, clippy with no issues, and `cargo test: 12 passed`.

## Errors / Corrections

- The first `rtk make verify` run failed because the old `severity_from_rule_value` helper became unused after the policy engine replaced it. Removed the stale helper and reran the full gate.

## Ready for Next Run

- Next tracked action should be `task-005: Enforce external package policy`.
- The rule policy stores effective rule options, so task-005 can read `onion/no-forbidden-imports` options for layer-specific allowlists and severities.

## Open Blockers

None.
