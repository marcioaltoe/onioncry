---
task: task_01
spec: 0003-query-contract-filter-keys-and-mutation-meta
status: pending
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
