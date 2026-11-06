---
task: task_02
spec: 0003-query-contract-filter-keys-and-mutation-meta
status: completed
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

## Result

Skills loaded: implement-task, coding-guidelines, rust-expert, rust-cli, testing-boss, systematic-debugging, no-workarounds, evidence-gate, tech-writer, brainstorming, agentic-cli-design, onioncry, review.

Implemented only the mutation-meta slice. `QuerySyntaxFacts` now records whether any identifier or member `useMutation` call lacks a first-argument options object declaring `meta.invalidates`. The declaration accepts any value, including an empty list. Each call is examined independently; a contracted mutation cannot hide an uncontracted sibling. The hook diagnostic uses `syntax.has_uncontracted_mutation && !source_has_mutation_invalidation(source)` and retains its existing message, suggestion and severity. The invalidation helper, optimistic-update check and other query-contract checks are unchanged.

### Reproduction before production edits

`rtk proxy cargo test --test feature_system feature_system_query_contract_accepts_mutation_meta_invalidates -- --exact --nocapture` exited 101 before the production change. Its real CLI child analyzed one export hook with `meta: { invalidates: [] }`, no invalidation, and returned exit 1 under `--fail-on warning`. JSON contained exactly one warning: `mutation hooks should invalidate relevant queries in onSuccess or onSettled`. The same focused command exited 0 after the production change, covering both `[]` and `[["billing"]]`.

### Acceptance evidence

1. Declared invalidation: the new positive real-binary regression accepts both required lists with zero violations under `--fail-on warning`. The syntax unit test also covers identifier/member callees, quoted properties, shorthand `invalidates`, non-array values, comments, strings, unrelated calls, nonliteral options and declaration placement.
2. Existing valid mutations and missing contracts: the unchanged default query-contract regression passes, including explicit `onSuccess` and `onSettled` invalidation. The new negative real-binary regression asserts CLI exit 1 and five findings: missing `meta`, `meta` without `invalidates`, mixed contracted/uncontracted mutations, optimistic `onMutate` without rollback despite declared invalidation, and a production component importing an adapter. Mutation message, suggestion and warning severity are asserted. The existing mutation-cache regression also passes unchanged.
3. Compatibility: all existing tests remain byte-identical; the selected existing query-contract and mutation-cache tests pass. No configuration, schema, diagnostic constructor, rule identifier or exit-code implementation changed. The complete feature-system suite and repository gates remain for Daemon Verification; focused results do not establish their outcome. The documentation/skill wording belongs to Task 03 and is untouched.

### Focused checks

- `rtk proxy cargo test --test feature_system query_contract -- --nocapture` — exit 0; six tests passed, 35 filtered out, including both new real-binary regressions and existing default/custom and filter-key tests.
- `rtk proxy cargo test --test feature_system check_reports_feature_system_mutation_cache_handling_violations -- --exact --nocapture` — exit 0; one existing real-binary test passed, 40 filtered out.
- `rtk proxy cargo test --lib rules::feature_system::query_syntax::tests -- --nocapture` — exit 0; three unit tests passed.
- `rtk proxy cargo clippy --lib --test feature_system -- -D warnings` — exit 0.
- `rtk proxy rustfmt --edition 2024 --check src/rules/feature_system/query_syntax.rs src/rules/feature_system/query_contract.rs tests/feature_system.rs` — exit 0.
- Separate read-only Standards and Spec reviews of the implementation diff returned `No findings.`

The negative regression's initial assertion abbreviated the dependency-flow message; it was corrected to assert the existing complete diagnostic. No production change was needed for that assertion.

Task status remains Daemon-owned. The declared Verification, `make verify`, `make rust-verify` and full suite were not run. No commits, pushes, PRs, Task Graph edits or sibling Task edits were made.

## Recorded paths

The Daemon recorded these paths, which this Task changed without declaring them in `## Context`.

- `src/rules/feature_system/query_syntax.rs`
