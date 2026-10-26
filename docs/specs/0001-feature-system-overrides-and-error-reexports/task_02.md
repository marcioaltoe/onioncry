---
task: task_02
spec: 0001-feature-system-overrides-and-error-reexports
status: pending
type: backend
complexity: high
---

# Task 02: Prove named Error class exports with the parser

## Overview

Replace the adapter contract's textual Error-class heuristic with proof of the actual named runtime export. Resolve only local analyzed modules, preserving class identity and the existing boundary diagnostics.

## Requirements

1. MUST fix the Error reexport defect measured on 2026-10-26. Reuse Oxc and existing resolved local ImportEdges; keep public ImportEdge/report schemas unchanged and add no dependency.
2. MUST recognize direct exported named Error subclasses, local class plus named export, multiline named reexports, aliases, multiple local hops, and an imported binding followed by a local named export.
3. MUST match the exact class behind the requested export and reject ordinary values, non-Error classes with unrelated Error-class decoys, comments/strings, missing names, type-only bindings, external/unresolved/excluded targets and cycles. Limit reads to the analyzed file set and track visited file/binding pairs. Do not execute source or infer unsupported wildcard/default/indirect-inheritance forms.
4. MUST add real binary tests prefixed feature_system_error_exports, including feature_system_error_exports_follow_runtime_named_bindings. Assert diagnostic files/rules and strict exits; a valid Error reexport must not hide an adapter-to-component import violation.
5. MUST update the repository-canonical .agents/skills/onioncry/SKILL.md to describe these corrected override/export contracts without new flags, schemas or unsupported syntax promises. Keep its released-version metadata until the separate synchronized release bump. Do not import the upstream mirror.

## Subtasks

- [ ] Add red named-export and false-proof CLI cases.
- [ ] Resolve the exact runtime class through bounded local AST summaries.
- [ ] Preserve negative boundaries and update the canonical usage skill.

## Acceptance Criteria

- [ ] Genuine named runtime Error reexports pass with aliases/chains and local import/export.
- [ ] All invalid/unknown/cyclic proofs remain violations without out-of-universe reads.
- [ ] Existing CLI contracts and adapter import boundaries remain enforced and documented.

## Context

- interface: `src/rules/feature_system/mod.rs`
- interface: `src/rules/feature_system/helpers.rs`
- interface: `src/rules/feature_system/adapter_contract.rs`
- creates: `src/rules/feature_system/error_exports.rs`
- interface: `src/imports.rs`
- interface: `tests/feature_system.rs`
- interface: `tests/support/mod.rs`
- interface: `.agents/skills/onioncry/SKILL.md`
- interface: `docs/references/2026-10-26-feature-system-contracts.md`

## Consumers

- Adapter Error export contract — consumers: `--test feature_system`

## Verification

- `grep -q '^fn feature_system_error_exports_follow_runtime_named_bindings' tests/feature_system.rs && rtk cargo test --test feature_system` — expected: exit 0; new runtime binding cases and all existing feature-system consumers pass.

## References

- `_prd.md` → Goals 2 and 3; Core Features 2 and 3; Success Metrics 2 and 3.
- `_techspec.md` → API Contract 1; Build Order 2; Testing Approach.
