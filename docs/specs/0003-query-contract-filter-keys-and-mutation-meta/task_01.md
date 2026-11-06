---
task: task_01
spec: 0003-query-contract-filter-keys-and-mutation-meta
status: completed
type: backend
complexity: medium
---

# Task 01: Treat factory keys in QueryClient filters as key uses

## Overview

The Feature System Query Contract counts any `queryKey` text as a key declaration, so hooks that invalidate through key factories and components that reload through them are reported as drift. Add a private Oxc-based query syntax summary and use its key-declaration fact for the hook, component and route checks. This answers the `vortex` report of 2026-10-25 recorded in `docs/references/2026-11-06-query-contract-filters-and-mutation-meta.md`.

## Requirements

1. MUST reproduce the false positive through the real CLI before changing production code: a feature-system hook that reads through an option factory and invalidates with a key-factory `queryKey` is currently reported with "query hooks should reuse factories". This answers the `vortex` finding of 2026-10-25 (ai-memory message `01a17436-c585-7ae2-ad32-c86fb643132d`).
2. MUST add the private module `query_syntax` with `summarize_query_syntax` and its `declares_query_key` fact, as the TechSpec's System Architecture and Interfaces describe, using the existing `oxc_parser` and `oxc_ast_visit` dependencies and returning `OnionCryError::ParseSource` on failure. Do not add dependencies or change configuration.
3. MUST use `declares_query_key` for the "query hooks should reuse factories", "components should not own query keys" and "routes should not own query keys" checks, and remove `source_declares_query_key` once it has no caller. Keep every message, tip, severity and diagnostic owner unchanged.
4. MUST add the real-binary regression `feature_system_query_contract_accepts_factory_keys_in_query_filters`, covering a hook that reads through a factory and invalidates with a key-factory filter, a component that calls `refetchQueries` with a key-factory filter, and a shorthand `{ queryKey }` filter.
5. MUST add the real-binary negative control `feature_system_query_contract_reports_inline_query_key_declarations`: an inline `useQuery({ queryKey: [...] })` in a hook, an inline array literal in a component's `invalidateQueries` filter and a route that declares a key keep their diagnostics, and a `queryKey` that appears only in a comment or string no longer decides the result.

## Subtasks

- [ ] Reproduce the false positive through the public binary.
- [ ] Add the Oxc query syntax summary and its key-declaration fact.
- [ ] Switch the hook, component and route key checks to the fact.
- [ ] Add the positive and negative real-binary regressions.

## Acceptance Criteria

- [ ] Non-literal `queryKey` values in QueryClient filter calls never count as key declarations in hooks, components or routes.
- [ ] Inline array-literal keys keep counting as declarations in every position.
- [ ] Existing feature-system tests pass unchanged; no configuration, schema, rule name, message or exit-code change.

## Context

- creates: `src/rules/feature_system/query_syntax.rs`
- interface: `src/rules/feature_system/mod.rs`
- interface: `src/rules/feature_system/query_contract.rs`
- interface: `src/rules/feature_system/helpers.rs`
- interface: `tests/feature_system.rs`
- instruction: `src/rules/feature_system/error_exports.rs`
- instruction: `tests/support/mod.rs`
- instruction: `docs/references/2026-11-06-query-contract-filters-and-mutation-meta.md`

## Consumers

- `frontend/feature-system-query-contract` key-ownership diagnostics — consumers: `--test feature_system`

## Verification

- `grep -q '^fn feature_system_query_contract_accepts_factory_keys_in_query_filters' tests/feature_system.rs && grep -q '^fn feature_system_query_contract_reports_inline_query_key_declarations' tests/feature_system.rs && rtk cargo test --test feature_system` — expected: exit 0; both new real-binary tests and every existing feature-system contract pass.

## References

- `_prd.md` → Goals 1 and 3; User Stories 1, 2 and 4; Core Features 1 and 3; Success Metrics 1 and 2.
- `_techspec.md` → API Contract 1; System Architecture; Interfaces; Testing Approach; Build Order 1.
- ADR-0013; ADR-0004.

## Result

Skills loaded: implement-task, coding-guidelines, systematic-debugging, no-workarounds, rust-expert, rust-cli, testing-boss, agentic-cli-design, onioncry, context7-cli, tech-writer, evidence-gate.

Implemented Build Order 1 only. The private Oxc summary returns `declares_query_key` and typed `OnionCryError::ParseSource` errors. Query contract analysis caches the summary for hooks, components and routes, and all three key-ownership checks consume it. Removed the unused text helper. Diagnostic messages, suggestions, severity and owning files remain unchanged; no dependency, configuration, schema or exit-code changes were made. Mutation-meta analysis remains for task_02.

Reproduction before production edits: `rtk proxy cargo test --test feature_system feature_system_query_contract_accepts_factory_keys_in_query_filters -- --exact --nocapture` failed with test exit 101. The initial real-binary fixture returned CLI exit 1 and 17 query-contract errors across 27 files: one factory-reusing query hook, eight components and eight routes. Its hook combined `useQuery(options())` with an `onSuccess` invalidation using `keys.list()`, isolating the reported factory-reuse false positive.

Acceptance evidence:

- Non-literal filter keys: `feature_system_query_contract_accepts_factory_keys_in_query_filters` passes with zero violations. It exercises all eight specified methods in hooks that read through option factories, components and routes, including factory calls, shorthand `{ queryKey }`, quoted property names and computed member calls. The hook combining a read and mutation invalidation also passes.
- Inline keys: `feature_system_query_contract_reports_inline_query_key_declarations` passes while requiring exactly 26 diagnostics with the existing messages and owners: inline filter arrays for all eight methods in hooks, components and routes, plus an inline `useQuery` key and a route-owned key. Comment/string-only files contribute no diagnostic. Unit cases also retain declarations outside filters, in later arguments and in nested values, including a nested declaration inside an otherwise exempt key expression.
- Compatibility: the five existing tests selected by `query_` pass unchanged, including default/custom configuration, option shape/signal diagnostics, missing query files and cross-file override facts. The existing mutation-cache negative control also passes unchanged. The full feature-system suite and repository gate remain for Daemon Verification; these focused results do not claim those gates passed.

Focused checks on the implementation:

- `rtk proxy cargo test --lib query_syntax::tests -- --nocapture`: exit 0, 2 tests passed, covering syntax positions and typed parse/source-type errors.
- `rtk proxy cargo test --test feature_system query_ -- --nocapture`: exit 0, 7 tests passed (the two new real-binary regressions and five existing tests).
- `rtk proxy cargo test --test feature_system check_reports_feature_system_mutation_cache_handling_violations -- --exact`: exit 0, 1 existing test passed.
- `rtk proxy cargo clippy --lib --test feature_system -- -D warnings`: exit 0, no warnings.
- `rtk proxy rustfmt --check --edition 2024 src/rules/feature_system/query_syntax.rs src/rules/feature_system/query_contract.rs tests/feature_system.rs`: exit 0.
- `rtk proxy git -c core.fsmonitor=false diff --check`: exit 0.

The initial worktree already had a Daemon-owned change to this Task file. Its status and authored sections were preserved; only this Result was appended. No declared Verification command, repository Verification, incremental Verification, commit, push or PR operation was run. No other Task or Task Graph file was changed.
