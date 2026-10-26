---
task: task_01
spec: 0001-feature-system-overrides-and-error-reexports
status: completed
type: backend
complexity: high
---

# Task 01: Honor each diagnostic owner's effective rule

## Overview

Correct the first-setting collector defect measured on 2026-10-26 in all five feature-system families. Each diagnostic follows its owner's effective severity/options without losing relationships to files with different overrides.

## Requirements

1. MUST fix the 2026-10-26 first non-Off setting defect in layout, public-api, dependency-flow, adapter-contract and query-contract collectors; use RulePolicy::effective_rule and preserve last-matching whole-value replacement.
2. MUST evaluate each distinct enabled RuleSetting once over the complete files/edges and retain diagnostics according to the existing Violation.file owner. Do not filter the graph or rescan the entire project once per file.
3. MUST preserve the existing representative file for aggregate missing-directory/barrel/adapter/query-file diagnostics and test how an exact Off override applies to that owner.
4. MUST add real binary regression tests named with feature_system_overrides prefix. Cover off/warn/error, later match precedence and file-specific option replacement for all five families, an earlier unrelated file, and cross-file query/adapter facts under differing policies. Assert exact rule/file/severity/counts and --fail-on warning exit behavior.
5. MUST prove a narrowly exempt test/export hook cannot suppress an invalid production component in the same fixture. Keep every production restriction and public CLI/report contract.

## Subtasks

- [ ] Add red CLI cases for file policy and cross-file relationships.
- [ ] Correct and deduplicate collector policy selection.
- [ ] Verify aggregate ownership, strict exits and production protection.

## Acceptance Criteria

- [ ] All five families honor file-scoped severity and whole-option replacement.
- [ ] Full graph relationships and aggregate diagnostic owners remain stable.
- [ ] Invalid production imports still fail beside an exempt file.

## Context

- interface: `src/rules/feature_system/mod.rs`
- interface: `src/rules/feature_system/layout.rs`
- interface: `src/rules/feature_system/public_api.rs`
- interface: `src/rules/feature_system/dependency_flow.rs`
- interface: `src/rules/feature_system/adapter_contract.rs`
- interface: `src/rules/feature_system/query_contract.rs`
- interface: `src/policy.rs`
- interface: `tests/feature_system.rs`
- interface: `tests/support/mod.rs`
- interface: `docs/references/2026-10-26-feature-system-contracts.md`

## Consumers

- Feature-system diagnostic policy — consumers: `--test feature_system`

## Verification

- `rg -q '^fn feature_system_overrides_are_applied_per_diagnostic_file' tests/feature_system.rs && rtk cargo test --test feature_system` — expected: exit 0; the new per-file regression and the existing public CLI suite pass, with preserved production-negative and aggregate behavior.

## References

- `_prd.md` → Goal 1; Core Feature 1; Success Metric 1.
- `_techspec.md` → API Contract 1; Build Order 1; Testing Approach.


## Result

Implemented the Task 01 collector slice in `src/rules/feature_system/mod.rs`
and added five real-binary regression tests in `tests/feature_system.rs`.
Each collector caches `RulePolicy::effective_rule` by diagnostic-owner path,
evaluates each distinct enabled whole `RuleSetting` once with the complete
files/edges, and retains only findings whose owner has that setting. Off
owners emit no findings. Policy implementations and representative-file
selection remain unchanged.

Acceptance-criterion evidence:

- All five families honor file-scoped severity and whole-option replacement:
  `feature_system_overrides_are_applied_per_diagnostic_file` covers base
  error, exact Off/warn/error overrides, later-match precedence, an earlier
  unrelated warning file, duplicate equal settings, exact rule/file/severity
  multisets and summary counts. Warning-only fixtures exit 0 by default and
  1 with `--fail-on warning`.
  `feature_system_overrides_replace_whole_options_per_file` covers each
  family's file-specific options: omitted base `systemsRoots` reset to
  defaults, an option-bearing warning override replaces an earlier Off,
  and findings for siblings under the base options remain absent.
- Full graph relationships and aggregate owners remain stable:
  `feature_system_overrides_preserve_aggregate_owners` asserts the existing
  owner and counts for missing directories/root barrel (3), adapter (1),
  and query files (2), then suppresses only that exact owner without
  relocating its findings. An Off hook still creates query-file obligations
  for the enabled representative.
  `feature_system_overrides_keep_cross_file_query_and_adapter_facts`
  proves that Off adapter/query files remain present to enabled siblings,
  hooks still reuse factories across policies, and an Off adapter still
  creates the query-option owner's cancellation obligation.
- Invalid production imports still fail beside an exempt file:
  `feature_system_overrides_exempt_export_hook_without_relaxing_production`
  exempts only the export hook through its own options, while the production
  component retains exactly one dependency-flow error and exit 1. Existing
  adapter import-boundary, cancellation, dependency-flow, surface CSS,
  query-shape/signal and mutation-cache negative cases were also exercised.

Focused checks:

- Before the collector change,
  `rtk proxy cargo test --test feature_system feature_system_overrides -- --nocapture`
  exited 101, exposing unrelated-file severity leakage, aggregate-owner
  severity mismatch, ignored option replacement and the ignored hook
  exemption. A separate assertion-path mismatch from macOS temporary-path
  canonicalization was corrected in the test helper.
- After the implementation, the same focused regression command exited 0:
  5 passed, 21 filtered out.
- `rtk proxy cargo test --test feature_system check_reports_feature_system -- --nocapture`:
  exit 0; 6 existing negative tests passed, 20 filtered out.
- `rtk proxy cargo clippy --test feature_system -- -D warnings`: exit 0.
- `rtk proxy rustfmt --edition 2024 --check src/rules/feature_system/mod.rs tests/feature_system.rs`:
  exit 0.
- `rtk proxy git diff --check`: exit 0.

Environment: the executor rejected the symlinked workspace root before
launching a command. Work and focused checks used the resolved Run checkout
on the same volume with approved execution escalation.

Daemon-owned checks were not run: the declared Verification command,
`make verify`, `make rust-verify`, and the full suite. Task status and
checkboxes remain unchanged; no commit, push, PR, Task Graph or other Task
file changes were made. Named Error export resolution and the canonical
usage-skill update remain Task 02's slice.

## Carry-forward provenance

- Source Run: `run_20261026T134946Z_90c0398582783f78`
- Source commit: `5c496872ceeb9392f332982bf1745d2907f2b692`
