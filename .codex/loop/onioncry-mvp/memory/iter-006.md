# Iteration 006 Memory

## Objective Snapshot

Completed `task-006: Enforce bounded context public surface`.

## Important Decisions

- Added typed `contexts.*.patterns` configuration and a context classifier separate from layer classification.
- Files with no matching context are contextless and allowed in the MVP.
- Files matching multiple contexts report `onion/ambiguous-context`; ambiguous/contextless files do not participate in cross-context import checks.
- Added `contextRules.default.allowSameContext` and `contextRules.default.allowCrossContext` support.
- `onion/no-cross-context-internal-import` checks only resolved local import edges where both source and target have unambiguous contexts.
- Cross-context imports are allowed when the target path contains one of the configured public surface segments, such as `contracts`, `events`, `ports`, or `shared`.
- Global shared files are not treated as a context unless the user explicitly configures a `shared` context.
- JSON diagnostics for context violations include `fromContext`, `toContext`, import specifier, target file, source location, and suggestion.

## Files / Surfaces Touched

- `src/lib.rs`
- `tests/check_cli.rs`

## Validation Evidence

- Added integration coverage for same-context imports, public surface imports, internal cross-context imports, contextless source/target files, ambiguous context classification, and `shared` as a public surface segment rather than an implicit context.
- The first targeted test failed before implementation because context configuration produced no violations.
- `rtk cargo test check_enforces_bounded_context_public_surface --test check_cli --all-features` passed.
- `rtk cargo test --test check_cli --all-features` passed with 14 tests.
- `rtk make verify` passed.
- Final verify output included conventional commit checks, clippy with no issues, and `cargo test: 14 passed`.

## Errors / Corrections

- No verify failures after implementation. The initial red test showed the missing behavior before the fix.

## Ready for Next Run

- Next tracked action should be `task-007: Detect circular dependencies`.
- Cycle detection can reuse the already-collected resolved local import edges and should ignore external and unresolved imports.

## Open Blockers

None.
