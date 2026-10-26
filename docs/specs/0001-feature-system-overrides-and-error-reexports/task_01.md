---
task: task_01
spec: 0001-feature-system-overrides-and-error-reexports
status: pending
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
