---
task: task_04
spec: 0001-feature-system-overrides-and-error-reexports
status: pending
type: backend
complexity: medium
---

# Task 04: Reject runtime namespaces that shadow Error

## Overview

Resolve standing pre-PR review finding F1 at candidate c6c81be6c3588c7c56da72764fb138f22c647143. The named-export resolver guards several runtime bindings called Error, but its declaration inspection omits TypeScript namespace declarations. A runtime namespace can therefore be mistaken for the intrinsic Error constructor.

## Requirements

1. MUST reproduce F1 through the real CLI before changing production code, using a generic temporary project whose runtime namespace Error has an exported value and whose configured API error class extends that local binding. Confirm the existing false acceptance, then require the adapter Error-export diagnostic and strict failure after the repair.
2. MUST correct the owning Oxc binding inspection for runtime namespace forms that introduce a top-level Error binding, including an exported namespace and an identifier-root dotted namespace when supported by the installed parser. Preserve local named-export resolution, analyzed-file bounds, exact class identity and existing conservative unknown handling. Do not replace AST proof with text matching or add dependencies.
3. MUST add feature_system_error_exports_reject_runtime_error_namespaces and related real-binary cases to the existing feature_system suite. Cover the direct namespace, a reexport chain, and negative controls that keep a genuine intrinsic Error subclass accepted beside an unrelated namespace or a nested Error binding that does not shadow the class's scope. Characterize ambient/type-only forms explicitly rather than silently broadening the runtime contract.
4. MUST preserve the five-family per-file override behavior, adapter import restrictions, flags, output schemas, exit codes and existing assertions. Update the canonical OnionCry skill only if its runtime-binding explanation needs clarification; keep released-version metadata until the synchronized release bump.
5. MUST stay within src/rules/feature_system/error_exports.rs, tests/feature_system.rs and .agents/skills/onioncry/SKILL.md plus this Task file. The operative authorization already bounds these paths. Do not change tooling configuration, version pins, the Task Graph, another Task file or the prior QA Report. The Supervisor reopens terminal QA through the public command.

## Subtasks

- [ ] Reproduce the namespace false proof through the public binary.
- [ ] Correct runtime namespace binding ownership and add positive/negative regressions.
- [ ] Record focused evidence and reconcile the canonical skill's contract.

## Acceptance Criteria

- [ ] Runtime namespaces that shadow Error cannot prove an API Error export, directly or through named reexports.
- [ ] Genuine intrinsic Error subclasses and unrelated/nested namespace controls retain their expected diagnostics and exits.
- [ ] Existing feature-system contracts remain enforced with no new dependency or configuration changes.

## Context

- interface: `src/rules/feature_system/error_exports.rs`
- interface: `src/rules/feature_system/adapter_contract.rs`
- interface: `src/rules/feature_system/mod.rs`
- interface: `tests/feature_system.rs`
- interface: `tests/support/mod.rs`
- interface: `.agents/skills/onioncry/SKILL.md`

## Consumers

- Named runtime Error export proof — consumers: `--test feature_system`

## Verification

- `rg -q '^fn feature_system_error_exports_reject_runtime_error_namespaces' tests/feature_system.rs && rtk cargo test --test feature_system` — expected: exit 0; the new real-binary namespace regression and all existing feature-system contracts pass.

## References

- `_prd.md` → Goals 2 and 3; Core Features 2 and 3; Success Metrics 2 and 3.
- `_techspec.md` → API Contracts; Implementation Design; Testing Approach.
