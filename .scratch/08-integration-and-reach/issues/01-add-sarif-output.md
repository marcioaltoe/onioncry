# Add SARIF output format

Status: done

## Parent

../PRD.md

## What to build

Add `--format sarif` to `check` so violations appear as inline annotations in code-review and code-scanning tools that consume SARIF 2.1.0.

## Acceptance criteria

- [x] `onioncry check --format sarif` emits valid SARIF 2.1.0 built with serde types; no hand-assembled JSON.
- [x] Severity maps `error` to SARIF `error` and `warn` to SARIF `warning`.
- [x] The SARIF `tool.driver` section lists rule metadata (id, explanation) from the rule catalog; results reference rules by canonical linter-style name.
- [x] Results carry file path plus line and column when parser spans are available.
- [x] Baselined violations are emitted with SARIF `suppressions` entries so scanning UIs treat them as suppressed instead of new.
- [x] Exit-code behavior matches other formats under the effective failure threshold.
- [x] SARIF output validates against the SARIF 2.1.0 schema in a test (fixture-based validation is acceptable).
- [x] CLI integration tests cover clean, failing, and baselined runs.
- [x] README documents the format with a plain `gh`/upload example (no maintained GitHub Action — out of scope per PRD).
- [x] `make verify` passes.

## Blocked by

violation-baseline/03-render-baseline-in-outputs (suppressions mapping needs baselined violations in the report model)

## Comments

- 2026-07-04: Added `onioncry check --format sarif` with serde-backed SARIF 2.1.0 rendering, rule-catalog driver metadata, severity mapping, parser span locations, and external suppressions for baselined violations. Added fixture-backed SARIF schema validation plus clean/failing/baselined CLI tests, and documented the plain `gh api` upload workflow in README.md. Verification: `rtk cargo test --test cli_sarif` passed with 3 tests; `rtk make verify` passed with clippy clean and 103 tests. The exact commit SHA is reported by the implementation loop after the slice commit is created.

## Comments

Post-delivery fix: SARIF `artifactLocation.uri` values were absolute local
paths, which GitHub code scanning cannot map to repository files. URIs are now
project-root-relative with forward slashes; the integration test asserts the
exact relative URI.
