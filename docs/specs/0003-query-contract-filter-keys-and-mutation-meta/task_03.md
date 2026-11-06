---
task: task_03
spec: 0003-query-contract-filter-keys-and-mutation-meta
status: pending
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
