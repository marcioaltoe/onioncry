# Add SARIF output format

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add `--format sarif` to `check` so violations appear as inline annotations in code-review and code-scanning tools that consume SARIF 2.1.0.

## Acceptance criteria

- [ ] `onioncry check --format sarif` emits valid SARIF 2.1.0 built with serde types; no hand-assembled JSON.
- [ ] Severity maps `error` to SARIF `error` and `warn` to SARIF `warning`.
- [ ] The SARIF `tool.driver` section lists rule metadata (id, explanation) from the rule catalog; results reference rules by canonical linter-style name.
- [ ] Results carry file path plus line and column when parser spans are available.
- [ ] Baselined violations are emitted with SARIF `suppressions` entries so scanning UIs treat them as suppressed instead of new.
- [ ] Exit-code behavior matches other formats under the effective failure threshold.
- [ ] SARIF output validates against the SARIF 2.1.0 schema in a test (fixture-based validation is acceptable).
- [ ] CLI integration tests cover clean, failing, and baselined runs.
- [ ] README documents the format with a plain `gh`/upload example (no maintained GitHub Action — out of scope per PRD).
- [ ] `make verify` passes.

## Blocked by

violation-baseline/03-render-baseline-in-outputs (suppressions mapping needs baselined violations in the report model)
