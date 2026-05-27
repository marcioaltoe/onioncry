# Iteration 008 Memory

## Objective Snapshot

Completed `task-008: Generate init template`.

## Important Decisions

- Added `onioncry init` as a top-level CLI subcommand.
- `onioncry init` writes `.onioncryrc.jsonc` in the current working directory.
- Existing config files are not overwritten unless `--force` is passed.
- The generated template is JSONC with full-line `// TODO` comments so it remains valid JSON after those comment lines are stripped.
- The template includes `$schema`, `version`, `project`, `aliases`, `layers`, `contexts`, `contextRules`, `rules`, and `overrides`.
- The template uses the default domain, application, infra, and shared layers with explicit `mayImport` examples.
- The default rules match the MVP preset: layer leak and cross-context internal imports as errors; external package policy with domain error, application warn, and infra off; unresolved imports, circular dependencies, and unclassified files as warnings.

## Files / Surfaces Touched

- `src/lib.rs`
- `src/main.rs`
- `tests/check_cli.rs`

## Validation Evidence

- Added integration coverage for template creation, no-overwrite behavior, `--force` overwrite, required template sections and defaults, TODO comments, and parseability after stripping full-line JSONC comments.
- The first targeted init test run failed because `init` was not yet a registered subcommand.
- `rtk cargo test init_ --test check_cli --all-features` passed with 2 init tests.
- `rtk cargo test --test check_cli --all-features` passed with 17 tests.
- `rtk make verify` passed.
- Final verify output included conventional commit checks, clippy with no issues, and `cargo test: 17 passed`.

## Errors / Corrections

- No verify failures after implementation. The initial red test captured the missing subcommand.

## Ready for Next Run

- Next tracked action should be `task-009: Explain one file`.
- `explain` should reuse the same config discovery and analysis paths as `check` so classifications, imports, package policy, unresolved imports, and violations remain consistent.

## Open Blockers

None.
