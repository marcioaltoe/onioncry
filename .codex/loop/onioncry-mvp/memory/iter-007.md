# Iteration 007 Memory

## Objective Snapshot

Completed `task-007: Detect circular dependencies`.

## Important Decisions

- Added `onion/circular-dependency` using resolved local import edges already collected for the selected file universe.
- Cycle detection ignores external package imports, unresolved local imports, and resolved local targets outside the selected file universe.
- The rule reports canonical cycles by starting at the lexicographically smallest path in the cycle and returning to that file at the end of `cyclePath`.
- Circular dependency diagnostics include a readable message and JSON `cyclePath` entries relative to the project root.
- The default severity remains `warn`, and rule overrides apply to the canonical reported source file.

## Files / Surfaces Touched

- `src/lib.rs`
- `tests/check_cli.rs`

## Validation Evidence

- Added integration coverage for a simple cycle, a longer cycle, an acyclic graph, ignored external imports, ignored unresolved imports, override suppression, and `--fail-on warning`.
- The first targeted test failed before implementation because no circular dependency violations were emitted.
- `rtk cargo test check_detects_circular_dependencies_and_honors_rule_policy --test check_cli --all-features` passed.
- `rtk cargo test --test check_cli --all-features` passed with 15 tests.
- `rtk make verify` passed.
- Final verify output included conventional commit checks, clippy with no issues, and `cargo test: 15 passed`.

## Errors / Corrections

- No verify failures after implementation. The initial red test captured the missing cycle detector.

## Ready for Next Run

- Next tracked action should be `task-008: Generate init template`.
- The init template should include all configuration sections currently supported by the MVP: project, aliases, layers, contexts, contextRules, rules, and overrides.

## Open Blockers

None.
