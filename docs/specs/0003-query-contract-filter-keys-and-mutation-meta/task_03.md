---
task: task_03
spec: 0003-query-contract-filter-keys-and-mutation-meta
status: completed
type: docs
complexity: low
---

# Task 03: Document the filter-key and mutation-meta contracts

## Overview

Make the README rule summary, the glossary and the canonical OnionCry skill describe what the query contract now accepts: key-factory keys in QueryClient filters are key uses, and `meta.invalidates` declares a mutation's invalidation contract.

## Requirements

1. MUST update the `frontend/feature-system-query-contract` summary in the README to say that key-factory keys in QueryClient filters are uses of existing keys and that a mutation may declare `meta.invalidates`, with an empty list for a mutation that writes nothing. It MUST also say that OnionCry does not prove the declaration.
2. MUST revise the glossary definition of **Feature System Query Contract** in `CONTEXT.md` through domain-modeling, so that it says mutations invalidate, roll back or declare `meta.invalidates`, and that filter keys from key factories are uses rather than ownership. Keep the term name and its `_Avoid_` line.
3. MUST read `.agents/skills/onioncry/SKILL.md` and update it only if it describes the query contract in a way that is now untrue; keep its released-version metadata until the release bump.

## Subtasks

- [ ] Update the README rule summary.
- [ ] Revise the glossary entry through domain-modeling.
- [ ] Reconcile the canonical OnionCry skill.

## Acceptance Criteria

- [ ] README and glossary state both contracts and the declaration-not-proof limit.
- [ ] The glossary term keeps its name, and the skill matches shipped behavior.

## Context

- interface: `README.md`
- interface: `CONTEXT.md`
- interface: `.agents/skills/onioncry/SKILL.md`
- instruction: `docs/agents/domain.md`

## Consumers

None. Documentation only; no Shared Contract changes.

## Verification

- `grep -qF 'meta.invalidates' README.md && grep -qF 'meta.invalidates' CONTEXT.md && tr -s '[:space:]' ' ' < CONTEXT.md | grep -qF -- "**Feature System Query Contract**:" || { printf 'missing contract text in README.md or CONTEXT.md\n' >&2; exit 1; }` — expected: exit 0; both documents name the `meta.invalidates` contract and the glossary keeps the term.

## References

- `_prd.md` → Core Feature 4; Glossary.
- `_techspec.md` → Build Order 3; Risks & Considerations.

## Result

Skills loaded: implement-task, crafting-effective-readmes, domain-modeling, tech-writer, evidence-gate.

- Acceptance criterion 1: the README rule summary and the glossary now describe key-factory keys in QueryClient filters as uses of existing keys, while inline array literals remain declarations. Both describe `meta.invalidates`, with `[]` for a mutation that writes nothing to the cache, and state that OnionCry does not prove the declaration or the central handler's invalidation. The glossary retains explicit invalidation and rollback and the assisted-review limit for cache scope.
- Acceptance criterion 2: revised only the definition of **Feature System Query Contract**, preserving its name and `_Avoid_` line. Read the entire canonical OnionCry skill: its feature-system section covers effective settings, overrides and adapter checks, and makes no query-contract claim rendered untrue by this change. It therefore remains unchanged, including released-version metadata `0.2.1`.
- Focused evidence: inspected the changed passages with `rtk rg -n -A 10` and compared their semantics with ADR-0013, ADR-0014 and `src/rules/feature_system/query_syntax.rs`. A read-only Python comparison against `HEAD` exited 0, confirming the glossary name, `_Avoid_` line and surrounding content are preserved, and the canonical skill is byte-identical with metadata `0.2.1`. `rtk git -c core.fsmonitor=false diff --check` exited 0.
- Scope: initial status showed only this Task file modified. This slice changes `README.md`, `CONTEXT.md` and this Result; it does not alter implementation code, the Task Graph or another Task file.
- Not run: the Task's declared Verification and `make verify`; both belong to Daemon settlement. Task status remains Daemon-owned. No commit, push or pull request was made. No follow-up work identified.
