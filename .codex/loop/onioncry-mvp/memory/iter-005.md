# Iteration 005 Memory

## Objective Snapshot

Completed `task-005: Enforce external package policy`.

## Important Decisions

- Added `onion/no-forbidden-imports` for import edges whose resolution is `External`.
- The rule runs only for files with an unambiguous source layer classification.
- Local aliases remain outside the external package allowlist: configured aliases resolve as local imports or unresolved local imports and are skipped by package policy.
- Package policy normalizes external import specifiers before allowlist matching: `uuid/v4` becomes `uuid`, and `@scope/pkg/subpath` becomes `@scope/pkg`.
- Runtime built-ins using the `node:` prefix are treated as external package names, so `node:fs`, `node:path`, and `node:crypto` are governed by the allowlist.
- Rule options support a `layers` array whose entries use `fromLayer`, `severity`, and `allow`. Layer-specific severity and allowlist replace the default rule setting for the matching layer.
- Allowlists support exact package names and glob patterns such as `@aws-sdk/*`.
- JSON diagnostics for external package violations include `packageName`, `fromLayer`, import specifier, file path, line, column, severity, and suggestion.

## Files / Surfaces Touched

- `src/lib.rs`
- `tests/check_cli.rs`

## Validation Evidence

- Added integration coverage for domain allowlisted packages, domain blocked packages, application warnings, infra off, runtime built-ins, scoped packages, subpaths, glob allowlist entries, and local aliases not being checked by external package policy.
- The first targeted test failed before implementation because external imports produced no violations.
- `rtk cargo test check_enforces_external_package_policy_by_layer --test check_cli --all-features` passed.
- `rtk cargo test --test check_cli --all-features` passed with 13 tests.
- `rtk make verify` passed after cleanup.
- Final verify output included conventional commit checks, clippy with no issues, and `cargo test: 13 passed`.

## Errors / Corrections

- The first `rtk make verify` run failed on a clippy `needless_borrow` warning introduced while changing layer checks to reuse collected import edges. Removed the needless borrow and reran the full gate.

## Ready for Next Run

- Next tracked action should be `task-006: Enforce bounded context public surface`.
- Context checks can reuse resolved local import edges and should add separate context classification rather than overloading layer classification.

## Open Blockers

None.
