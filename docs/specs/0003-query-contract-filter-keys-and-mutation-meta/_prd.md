---
spec: 0003-query-contract-filter-keys-and-mutation-meta
status: active
created: 2026-11-06
surfaces: [cli, docs]
---

# Feature System Query Contract: filter keys and mutation meta

The Feature System Query Contract reports correct TanStack Query code as drift. A hook that reads a query through an option factory and also invalidates that query after a mutation is told to "reuse factories". A component that reloads a query through a key factory is told it "owns" a key. A mutation that writes nothing, such as a file export, has no compliant form because every mutation must call `invalidateQueries`. Teams either add meaningless invalidations or turn the rule off and lose its protection. The limitation was reported by `vortex` in ai-memory message `01a17436-c585-7ae2-ad32-c86fb643132d` (2026-10-25). Evidence and outside sources are in `docs/references/2026-11-06-query-contract-filters-and-mutation-meta.md`.

## Project Constraints

- Identifier strategy: not applicable — no persistent identifiers are introduced. Source: `docs/agents/domain.md`.
- Authentication and HTTP: not applicable — analysis remains a local, read-only CLI operation without authentication or HTTP API changes. Source: `docs/agents/cli.md`.
- Active ADR obligations: applicable — follow ADR-0013 (query keys in QueryClient filters are key uses) and ADR-0014 (a mutation's `meta.invalidates` declares its invalidation contract), keep ADR-0004 (Oxc parses JavaScript and TypeScript) and ADR-0003 (linter-style configuration, unchanged); ADR-0012 does not apply because it governs Clean Architecture artifact placement, not the feature-system query contract. Source: `docs/agents/domain.md`; `docs/adr/0013-query-filter-keys-are-key-uses.md`; `docs/adr/0014-mutation-meta-invalidates-declares-invalidation.md`; `docs/adr/0004-use-oxc-parser-for-javascript-and-typescript.md`; `docs/adr/0003-use-linter-style-configuration.md`; `docs/adr/0012-artifact-suffixes-classify-roles-but-never-exempt-placement.md`.
- Tooling authority: applicable — express maintainer authorization on 2026-11-06 through the question tool to implement, commit, push, open the PR, merge with approved CI and publish the release. Source: `docs/agents/agent-instructions.md`; Spec-contained authorization record: `docs/specs/0003-query-contract-filter-keys-and-mutation-meta/_authorization.md`; bounded files: `src/rules/feature_system/query_contract.rs`, `src/rules/feature_system/helpers.rs`, `src/rules/feature_system/query_syntax.rs`, `src/rules/feature_system/mod.rs`, `tests/feature_system.rs`, `README.md`, `CONTEXT.md`, `.agents/skills/onioncry/SKILL.md`, `Cargo.toml`, `Cargo.lock`, `npm/package.json`, `npm/platforms/darwin-arm64/package.json`, `npm/platforms/darwin-x64/package.json`, `npm/platforms/linux-arm64/package.json`, `npm/platforms/linux-x64/package.json`, `npm/platforms/win32-x64/package.json`. The manifest paths cover only the own-package version bump of the release choreography. No dependency or checker-configuration change is proposed.

## Goals

1. A query key passed through a key factory to a QueryClient filter, such as an invalidation or a reload, is treated as a use of an existing key, in hooks, components and routes.
2. A mutation can declare that it writes nothing, or that a central cache handler invalidates for it, in its own code, and the contract accepts that declaration.
3. Every violation the contract reports today for inline query keys, mutations without any invalidation contract and optimistic updates without rollback is still reported.

## User Stories

1. As a frontend developer, I want a hook that reads a query through an option factory and invalidates it after a mutation to pass the Feature System Query Contract, so that I do not add workarounds to correct code.
2. As a frontend developer, I want a component that reloads a query through a key factory to pass the contract, so that "components should not own query keys" points only at real key ownership.
3. As a frontend developer, I want to mark a mutation that writes nothing, such as a file export, with `meta: { invalidates: [] }`, so that the contract stops asking for an invalidation that would cause useless work.
4. As a maintainer who raised the rule to `error`, I want inline query keys and mutations with no invalidation contract to keep failing, so that the gate still protects production code.

## Core Features

1. Filter key use: a `queryKey` in the filter argument of a QueryClient filter method counts as a use of an existing key when its value is not an inline array literal. Inline array literals and every `queryKey` outside those filters keep counting as declarations.
2. Mutation meta contract: a mutation whose options declare `meta.invalidates`, empty or not, satisfies the invalidation requirement. A file still fails when any of its mutations has neither that declaration nor an invalidation in `onSuccess` or `onSettled`.
3. Syntax-based reading: both decisions come from parsed call syntax, so comments, strings and unrelated identifiers named `queryKey` or `meta` no longer decide the result.
4. Documentation: the README rule summary, the glossary entry for the Feature System Query Contract and, where it describes the rule, the canonical OnionCry skill state the two contracts.

## User Experience

The CLI surface is unchanged. Code that follows the contracts produces no `frontend/feature-system-query-contract` diagnostic. Every other diagnostic keeps its message, tip, severity and location.

## Non-Goals / Out of Scope

- New rule options, rule names, flags, output fields or `.onioncryrc.jsonc` shape changes.
- Proving that an invalidation targets the right queries, or that a mutation declared as writing nothing really writes nothing. Cache scope stays an assisted review, as the glossary says.
- Following option factories across files, such as `useMutation(mutationOptions())`; such a call is treated as before.
- Changing the adapter, dependency-flow, layout or public-API rules of the Feature System Contract.

## Success Metrics

1. Success Metric: a fixture that mirrors the reported shape — hooks that read through factories and invalidate through key factories, components that reload through key factories, and an export mutation with `meta: { invalidates: [] }` — produces zero `frontend/feature-system-query-contract` diagnostics.
2. Success Metric: inline query keys in hooks, components and routes, a mutation with no invalidation contract, a mutation with `meta` lacking `invalidates`, and an optimistic update without rollback each keep their existing diagnostic, and the run fails under `--fail-on warning`. A production component importing an adapter keeps its dependency-flow diagnostic.
3. Success Metric: `make verify` and QA pass, and the released version is installable from crates.io and npm.

## Glossary

- changes: **Feature System Query Contract**

## Decisions

- 2026-11-06: What should happen to the Vortex query-contract request? — Make it Spec 0003 and deliver it up to the release: implement, commit, push, PR, merge with approved CI and publish.
- 2026-11-06: How should OnionCry recognize a mutation that writes nothing? — Through `meta.invalidates` in the code, read from the syntax tree, with an empty list meaning no write. No configuration change. See ADR-0014.
- Autonomous route: no person present for the filter-key design; decisions taken from the reported limitation, ADR-0004 and the TanStack Query sources in the committed reference. See ADR-0013.

## Open Questions

None.
