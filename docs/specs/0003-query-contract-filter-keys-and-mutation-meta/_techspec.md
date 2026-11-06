---
spec: 0003-query-contract-filter-keys-and-mutation-meta
prd: _prd.md
created: 2026-11-06
---

# Query contract filter keys and mutation meta — Technical Spec

## Executive Summary

Replace two text heuristics of `frontend/feature-system-query-contract` with facts read from the Oxc syntax tree (ADR-0004). One private module parses a hook, component or route file once. It reports whether the file declares a query key, ignoring factory keys inside QueryClient filters (ADR-0013), and whether any `useMutation` call lacks an invalidation contract, where `meta.invalidates` counts as one (ADR-0014). The rest of the query contract, the configuration, the diagnostics and the output stay unchanged.

## Project Constraints

- Identifier strategy: not applicable — no persistent identifiers are introduced. Source: `docs/agents/domain.md`.
- Authentication and HTTP: not applicable — analysis remains a local, read-only CLI operation without authentication or HTTP API changes. Source: `docs/agents/cli.md`.
- Active ADR obligations: applicable — follow ADR-0013 (query keys in QueryClient filters are key uses) and ADR-0014 (a mutation's `meta.invalidates` declares its invalidation contract), keep ADR-0004 (Oxc parses JavaScript and TypeScript) and ADR-0003 (linter-style configuration, unchanged); ADR-0012 does not apply because it governs Clean Architecture artifact placement, not the feature-system query contract. Source: `docs/agents/domain.md`; `docs/adr/0013-query-filter-keys-are-key-uses.md`; `docs/adr/0014-mutation-meta-invalidates-declares-invalidation.md`; `docs/adr/0004-use-oxc-parser-for-javascript-and-typescript.md`; `docs/adr/0003-use-linter-style-configuration.md`; `docs/adr/0012-artifact-suffixes-classify-roles-but-never-exempt-placement.md`.
- Tooling authority: applicable — express maintainer authorization on 2026-11-06 through the question tool to implement, commit, push, open the PR, merge with approved CI and publish the release. Source: `docs/agents/agent-instructions.md`; Spec-contained authorization record: `docs/specs/0003-query-contract-filter-keys-and-mutation-meta/_authorization.md`; bounded files: `src/rules/feature_system/query_contract.rs`, `src/rules/feature_system/helpers.rs`, `src/rules/feature_system/query_syntax.rs`, `src/rules/feature_system/mod.rs`, `tests/feature_system.rs`, `README.md`, `CONTEXT.md`, `.agents/skills/onioncry/SKILL.md`, `Cargo.toml`, `Cargo.lock`, `npm/package.json`, `npm/platforms/darwin-arm64/package.json`, `npm/platforms/darwin-x64/package.json`, `npm/platforms/linux-arm64/package.json`, `npm/platforms/linux-x64/package.json`, `npm/platforms/win32-x64/package.json`. The manifest paths cover only the own-package version bump of the release choreography. No dependency or checker-configuration change is proposed.

## System Architecture

`QueryContractAnalysis` in `src/rules/feature_system/query_contract.rs` already reads every analyzed source once into `source_by_file`. Three diagnostics depend on `helpers::source_declares_query_key`, a plain substring test for the text `queryKey`: the factory-reuse diagnostic in `hook_violations`, and the component and route key-ownership diagnostics in `violations`. The mutation diagnostic in `hook_violations` fires when the file contains `useMutation(` and `helpers::source_has_mutation_invalidation` is false.

Add a private module `src/rules/feature_system/query_syntax.rs` that parses one file with `oxc_parser` and walks call expressions with `oxc_ast_visit::Visit`, both already crate dependencies. It returns two facts:

- `declares_query_key`: some object property named `queryKey` (identifier, string-literal or shorthand key) appears outside an exempt filter position. The position is exempt when the property sits in the first argument, an object expression, of a call whose callee is a member expression with property name `invalidateQueries`, `refetchQueries`, `cancelQueries`, `resetQueries`, `removeQueries`, `isFetching`, `getQueriesData` or `setQueriesData`, and its value is not an array expression. A shorthand `{ queryKey }` in that position is exempt, because its value is an identifier.
- `has_uncontracted_mutation`: some call whose callee is the identifier `useMutation`, or a member expression with property name `useMutation`, has no first-argument object with a `meta` property whose object value has an `invalidates` property. Any value of `invalidates` counts, including an empty array.

`QueryContractAnalysis` computes the facts only for files whose area is hooks or components, or that are route files, and caches them per file within one analysis. The hook, component and route checks use `declares_query_key` instead of `source_declares_query_key`. The mutation check becomes `has_uncontracted_mutation && !source_has_mutation_invalidation(source)`, so a file whose mutations all declare `meta.invalidates` passes, and a file that passes today still passes. `source_declares_query_key` loses its last caller and is removed.

## Implementation Design

### Interfaces

```rust
pub(super) struct QuerySyntaxFacts {
    pub(super) declares_query_key: bool,
    pub(super) has_uncontracted_mutation: bool,
}

pub(super) fn summarize_query_syntax(file: &Path, source: &str) -> Result<QuerySyntaxFacts>;
```

A source type or parse failure returns the existing `OnionCryError::ParseSource`, as `error_exports.rs` does. Import scanning already rejects unparsable files, so no new failure mode reaches users.

Invariants:

1. A `queryKey` property whose value is an array expression is a declaration in every position.
2. A file with no `useMutation` call never produces the mutation diagnostic.
3. A file whose mutations would pass today, by containing `onSuccess` or `onSettled` together with `invalidateQueries`, still passes.
4. Comments and string literals never contribute either fact.

### Data Models

None. `QuerySyntaxFacts` is transient analysis state, not a public graph or report format.

### API Contracts

1. API Contract: `check` and `explain` keep their commands, streams, JSON/SARIF/LLM schemas, the rule name `frontend/feature-system-query-contract`, every diagnostic message and tip, and the 0/1/2 exit-code contract. Behavior changes in two places: a non-literal `queryKey` in a QueryClient filter no longer counts as a key declaration, and a `useMutation` call that declares `meta.invalidates` satisfies the invalidation requirement.

### Surface Transcripts

None. No new literal output is promised.

## Coverage Map

- Goal 1; User Stories 1–2; Core Feature 1; Success Metric 1 → `declares_query_key` in `query_syntax.rs` and its use in the hook, component and route checks.
- Goal 2; User Story 3; Core Feature 2; Success Metric 1 → `has_uncontracted_mutation` and the rewritten mutation check.
- Goal 3; User Story 4; Success Metric 2 → invariants 1–3 and the negative real-binary cases.
- Core Feature 3 → the Oxc walk replacing both substring checks (invariant 4).
- Core Feature 4 → the README rule summary, the glossary entry and the canonical OnionCry skill.
- Success Metric 3 → terminal CLI QA, the configured review, current-head CI and `docs/release.md`.
Deviations from the PRD: None.

## Absence Evidence

1. Syntax-based query facts module — command: `rg -n 'oxc|Visit|queryKey' src/rules/feature_system`; result: only `error_exports.rs` parses with Oxc, for Error class exports; every `queryKey` decision is the substring check in `helpers.rs` and `query_contract.rs`.
2. Mutation meta handling — command: `rg -n 'meta|invalidates' src/rules/feature_system`; result: no match; `source_has_mutation_invalidation` checks only `useMutation(`, `onSuccess`/`onSettled` and `invalidateQueries` text.

## ADR Alternatives

- ADR-0003: keep the configuration contract unchanged; the rejected per-file exemption option is not added.
- ADR-0004: keep Oxc as the only parser and replace string heuristics with syntax facts.
- ADR-0013: take its decision and reject the alternative it names, which is exempting every filter key, including inline literals.
- ADR-0012: unrelated; it governs Clean Architecture artifact placement and rejects nothing this design considers.
- ADR-0014: take its decision and reject the alternatives it names, which are a rule option listing exempt files and per-file overrides as the only remedy.

## Integration Points

Analysis remains local and read-only. Registry publication happens only through `docs/release.md` after approved review, required checks, synchronized versions and dry-runs. No consumer repository code is copied into tests.

## Testing Approach

Extend `tests/feature_system.rs` with real-binary cases built on its existing `write_feature_system_query_contract_config`, `write_file`, `run_json_check` and `run_json_check_failure` helpers.

- Filter keys (Build Order 1): a hook that calls `useQuery(factory())` and invalidates with `queryKey: keys.list()`, a component that calls `refetchQueries({ queryKey: keys.list() })`, and a shorthand `{ queryKey }` filter all pass. An inline `useQuery({ queryKey: [...] })` in a hook, an inline `invalidateQueries({ queryKey: ["billing"] })` in a component, a route that declares a key, and a key inside a comment or string all keep their current outcomes.
- Mutation meta (Build Order 2): an export hook with `meta: { invalidates: [] }` and no `invalidateQueries` passes, as does `meta: { invalidates: [["billing"]] }`. A mutation without `meta`, one with `meta: { toast: "saved" }`, and a file with one contracted and one uncontracted mutation and no invalidation fail. An optimistic `onMutate` without rollback still fails. A production component that imports an adapter keeps its dependency-flow diagnostic.
- The existing `check_accepts_default_feature_system_query_contract`, `check_reports_missing_feature_system_query_files_and_inline_hooks` and the rest of the suite stay unchanged and passing.

The QA gate's outside-evidence row compares the `meta: { invalidates: [] }` fixture with the TanStack maintainer guidance quoted in `docs/references/2026-11-06-query-contract-filters-and-mutation-meta.md`, then inspects the binary's JSON for the three variants it names.

## Build Order

1. Add `query_syntax.rs` with `declares_query_key`, use it for the hook, component and route key checks, remove `source_declares_query_key`, and add the filter-key regressions.
2. Add `has_uncontracted_mutation`, use it in the mutation check, and add the mutation-meta regressions (depends on: 1).
3. Update the README rule summary, the glossary entry for the Feature System Query Contract and, where it describes the rule, the canonical OnionCry skill (depends on: 2).
4. Run the authored terminal CLI QA (depends on: 3).
5. After the graph, review and merge through the Delivery Queue, then run the public release plan and the release choreography in `docs/release.md`.

## Risks & Considerations

Exempting only non-literal filter keys keeps hard-coded keys visible. A project that writes `invalidateQueries({ queryKey: ["billing"] })` in a hook that also reads a query keeps its warning, as it does today. Parsing adds one Oxc pass per hook, component and route file; import scanning already parses every file, so the cost is the same order. `meta.invalidates` is a declaration, not a proof: OnionCry does not check that a central `MutationCache` handler exists. The glossary already leaves cache scope to assisted review, and the README must say this.

## Glossary

- changes: **Feature System Query Contract**

## Research

- Local source inspection at `887bc93` established both heuristics and the absence of syntax-based query facts. See `docs/references/2026-11-06-query-contract-filters-and-mutation-meta.md`.
- Secondbrain: `~/dev/secondbrain/wiki/index.md` was read. The session's `qmd query` on OnionCry rules returned only this project's mirrors, byte-identical to the local originals; no Secondbrain page records a prior query-contract decision.
- Exa MCP: TkDodo's "Automatic Query Invalidation after Mutations" and its comment thread establish `meta.invalidates`, with `[]` meaning no invalidation, as published practice. That confirmed the maintainer's choice in ADR-0014.
- Context7 `/tanstack/query`: `UseMutationOptions.meta` is documented as a payload stored on the mutation cache entry and read by `MutationCache` callbacks. The pinned `oxc_ast_visit` 0.155 crate sources are authoritative for exact visitor APIs.

## Decisions

- 2026-11-06: filter keys count as uses only when they are not inline array literals (ADR-0013); `meta.invalidates` on the mutation's own options is the contract for mutations that write nothing or invalidate centrally (ADR-0014). Both facts come from one Oxc pass per relevant file.
