---
task: task_02
spec: 0003-query-contract-filter-keys-and-mutation-meta
status: pending
type: backend
complexity: medium
---

# Task 02: Accept meta.invalidates as a mutation's invalidation contract

## Overview

Every `useMutation` must currently call `invalidateQueries` in `onSuccess` or `onSettled`, so a mutation that writes nothing, such as a file export, has no compliant form. Extend the query syntax summary with an uncontracted-mutation fact in which `meta.invalidates`, even an empty list, is the declared contract, and use it in the mutation check.

## Requirements

1. MUST reproduce the false positive through the real CLI before changing production code: a feature-system hook whose only `useMutation` declares `meta: { invalidates: [] }` is currently reported with "mutation hooks should invalidate relevant queries in onSuccess or onSettled". This answers the `vortex` finding of 2026-10-25 (ai-memory message `01a17436-c585-7ae2-ad32-c86fb643132d`).
2. MUST add the `has_uncontracted_mutation` fact to `summarize_query_syntax` in the private `query_syntax` module that Task 01 creates, as the TechSpec's System Architecture describes, and make the mutation check `has_uncontracted_mutation && !source_has_mutation_invalidation(source)`. Keep the message, tip, severity, the optimistic-update check and every other query-contract check unchanged.
3. MUST add the real-binary regression `feature_system_query_contract_accepts_mutation_meta_invalidates`, covering an export hook with `meta: { invalidates: [] }` and no `invalidateQueries`, and a mutation with `meta: { invalidates: [["billing"]] }`.
4. MUST add the real-binary negative control `feature_system_query_contract_reports_mutations_without_invalidation_contract`: a mutation without `meta`, a mutation whose `meta` lacks `invalidates`, a file with one contracted and one uncontracted mutation and no invalidation, and an optimistic `onMutate` without rollback each keep their diagnostic under `--fail-on warning`, and a production component that imports an adapter keeps its dependency-flow diagnostic.

## Subtasks

- [ ] Reproduce the export-mutation false positive through the public binary.
- [ ] Add the uncontracted-mutation fact.
- [ ] Switch the mutation check to the fact.
- [ ] Add the positive and negative real-binary regressions.

## Acceptance Criteria

- [ ] A mutation that declares `meta.invalidates` satisfies the invalidation requirement, whatever list it declares.
- [ ] A file that passes the mutation check today keeps passing, and a file with any mutation lacking both a declaration and an invalidation keeps failing.
- [ ] Existing feature-system tests pass unchanged; no configuration, schema, rule name, message or exit-code change.

## Context

- interface: `src/rules/feature_system/query_contract.rs`
- interface: `src/rules/feature_system/helpers.rs`
- interface: `tests/feature_system.rs`
- instruction: `tests/support/mod.rs`
- instruction: `docs/references/2026-11-06-query-contract-filters-and-mutation-meta.md`

## Consumers

- `frontend/feature-system-query-contract` mutation diagnostics — consumers: `--test feature_system`

## Verification

- `grep -q '^fn feature_system_query_contract_accepts_mutation_meta_invalidates' tests/feature_system.rs && grep -q '^fn feature_system_query_contract_reports_mutations_without_invalidation_contract' tests/feature_system.rs && rtk cargo test --test feature_system` — expected: exit 0; both new real-binary tests and every existing feature-system contract pass.

## References

- `_prd.md` → Goals 2 and 3; User Stories 3 and 4; Core Features 2 and 3; Success Metrics 1 and 2.
- `_techspec.md` → API Contract 1; System Architecture; Interfaces; Testing Approach; Build Order 2.
- ADR-0014; ADR-0004.
