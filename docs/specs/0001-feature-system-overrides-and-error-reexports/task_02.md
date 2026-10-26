---
task: task_02
spec: 0001-feature-system-overrides-and-error-reexports
status: completed
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

- `rg -q '^fn feature_system_error_exports_follow_runtime_named_bindings' tests/feature_system.rs && rtk cargo test --test feature_system` — expected: exit 0; new runtime binding cases and all existing feature-system consumers pass.

## References

- `_prd.md` → Goals 2 and 3; Core Features 2 and 3; Success Metrics 2 and 3.
- `_techspec.md` → API Contract 1; Build Order 2; Testing Approach.

## Result

Implemented the Task 02 slice for Daemon Verification; status and authored
Verification remain Daemon-owned.

- Acceptance criterion 1: replaced the source-substring Error heuristic with
  private Oxc summaries of top-level classes, named runtime exports and named
  imports. The resolver follows the exact exported/local/imported name through
  existing resolved local ImportEdges, including aliases, multiline exports,
  multiple hops and import-then-export. Summaries are cached per resolver; an
  iterative traversal tracks visited file/name pairs, allowing a file to be
  revisited under a different binding while rejecting cycles.
  `feature_system_error_exports_follow_runtime_named_bindings`,
  `feature_system_error_exports_follow_configured_module_aliases` and
  `feature_system_error_exports_keep_shared_module_binding_identity` exercise
  these behaviors through the real binary with strict exit and exact
  diagnostic file/rule/count assertions.
- Acceptance criterion 2: ordinary values, non-Error classes with Error decoys,
  comments, strings, name-prefix decoys, absent names, ambient declarations,
  type-only imports/exports, external/unresolved targets, cycles and unsupported
  wildcard/default/indirect-inheritance forms do not prove the export. Local
  bindings shadowing Error also prevent a direct-subclass proof. The resolver
  checks membership in the analyzed file set before reading each module.
  `feature_system_error_exports_reject_false_and_unknown_proofs`,
  `feature_system_error_exports_reject_shadowed_error_superclasses` and
  `feature_system_error_exports_do_not_read_excluded_targets` cover the negative
  cases; the excluded target is tested with both valid Error source and invalid
  TypeScript, without producing an operational parsing error.
- Acceptance criterion 3: adapter-to-component imports still produce their
  existing finding after a valid Error reexport. The boundary regression
  asserts the adapter file, rule, targetFile, warning counts and default/strict
  exits (0/1). No public ImportEdge/report schemas, dependencies, CLI flags,
  diagnostic wording, namespace-object checks or cancellation checks changed.
  Updated only the repository-canonical OnionCry skill with the corrected
  per-file override and named Error export contracts; metadata.version remains
  0.1.0 for the separate synchronized release bump.

Focused implementation evidence:

- Red: `rtk cargo test --test feature_system feature_system_error_exports -- --nocapture`
  initially exited 101 (1 passed, 3 failed), exposing valid export rejection,
  the unrelated Error-class false proof and the extra false adapter finding.
  The shadowed-superclass case was also observed red before its guard.
- Green: `rtk cargo test --test feature_system feature_system_error_exports -- --nocapture`
  exited 0, with 7 passed and 26 filtered out.
- `rtk cargo test --test feature_system feature_system_adapter`: exit 0,
  5 passed, 28 filtered out; existing adapter naming/export, cancellation,
  import-boundary and custom-option behavior retained.
- `rtk cargo test --test feature_system feature_system_overrides`: exit 0,
  5 passed, 28 filtered out; prior per-file and whole-graph behavior retained.
- `rtk cargo clippy --lib --test feature_system -- -D warnings`: exit 0,
  no warnings.
- Formatted the changed Rust files with rustfmt. `rtk proxy git diff --check`:
  exit 0.
- Canonical skill validation: skill-creator's
  `quick_validate.py .agents/skills/onioncry`, run through
  `uv run --with pyyaml`: exit 0, `Skill is valid!`. The initial plain-python
  attempt lacked PyYAML; the isolated uv tool environment supplied it without
  project dependency edits.

Not run: the Task's authored Verification command, selected repository
Verification (`make verify`), incremental Verification (`make rust-verify`),
and the full test suite, per the Daemon-assigned execution contract. No Task
Graph, other Task, status, release metadata, commit, push or PR mutation was
performed. No follow-up implementation scope identified.
