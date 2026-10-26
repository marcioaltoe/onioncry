---
task: task_04
spec: 0001-feature-system-overrides-and-error-reexports
status: completed
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

## Result

Added `TSModuleDeclaration` to the owning Oxc declaration guard. Only an
identifier root named `Error` invalidates local intrinsic-superclass proof;
exported declarations already pass through this guard. The installed Oxc
0.133.0 parser represents dotted namespaces with the outer identifier at
that root. Export traversal, analyzed-file bounds, class identity, and policy
selection are unchanged.

Pre-change reproduction used a generic temporary project, the default
adapter path `packages/frontend/src/systems/billing/adapters/billing-api.ts`,
`project.include: ["**/*.ts", "**/*.tsx"]`, and only
`frontend/feature-system-adapter-contract: "error"` enabled. The adapter was:

```typescript
export const billingApi = {};
namespace Error { export const marker = 1; }
export class BillingApiError extends Error {}
```

After `rtk cargo build --bin onioncry --locked`, invoking the built binary
with `check --format json --fail-on warning` in that temporary project
returned exit 0, `status: pass`, and zero findings before production edits.
The same project and command after the repair returned exit 1, `status: fail`,
one error, and the exact diagnostic
`adapter file should export typed API error BillingApiError`. Stderr was
empty in both runs. The two new runtime-namespace tests also failed before
the repair because the real binary returned 0 instead of 1.

Acceptance evidence:

- Runtime namespace rejection: the new
  `feature_system_error_exports_reject_runtime_error_namespaces` covers eight
  direct/local-export forms, including exported namespaces, dotted identifier
  roots, `module` spelling, and a declaration after the class. The companion
  `_through_reexports` test covers four namespace forms behind renamed local
  exports, a reexport hop, and an import subsequently exported by name. All
  cases require the exact adapter diagnostic, owner, severity, count, and
  strict exit 1 through the existing real-binary report helper.
- Intrinsic and scope controls: twelve cases retain exit 0 and no findings
  for unrelated namespaces, nested `Error` declarations, nested block/function
  bindings, type aliases/interfaces, and a string-named ambient module.
  Five explicit conservative cases reject root `Error` namespaces that are
  ambient, empty, or contain only types. This is syntax-based proof, not
  TypeScript emit/type inference. The canonical OnionCry skill now explains
  this boundary; its released-version metadata remains `0.1.0`.
- Existing contracts: the filtered Error-export checks retain coverage for
  named bindings, unknown/type-only/cyclic targets, excluded files, module
  aliases, exact shared-module binding identity, adapter import restrictions,
  and warning thresholds. The five existing override tests exercise all five
  rule families. No dependencies, version pins, configuration, public output
  schemas, flags, or exit-code definitions changed.

Focused checks:

| Command / probe | Observed outcome |
| --- | --- |
| `rtk cargo test --locked --test feature_system feature_system_error_exports_reject_runtime_error_namespaces -- --nocapture` before the production repair | Two expected regression failures: actual CLI exit 0, required exit 1. |
| `rtk cargo test --locked --test feature_system feature_system_error_exports_ -- --nocapture` after the final test edits | 11 passed, 26 filtered out; includes four new tests with 29 namespace/control cases. |
| `rtk cargo test --locked --test feature_system feature_system_overrides_ -- --nocapture` | 5 passed, 32 filtered out. |
| `rtk proxy rustfmt --edition 2024 --check --config skip_children=true src/rules/feature_system/error_exports.rs tests/feature_system.rs` | Exit 0. |
| `rtk cargo clippy --locked --lib --bin onioncry --test feature_system -- -D warnings` | Exit 0, no issues found. |

The Task's declared Verification, `make verify`, and `make rust-verify` were
not run; they remain Daemon-owned. The pre-existing Task-file change and
`status: in_progress` were preserved. Only the two implementation/test paths,
the canonical OnionCry skill, and this Result section were edited. No Task
Graph, other Task, or prior QA Report was changed; no commit, push, or PR was
created. Terminal QA remains for the Supervisor's public-command reopening.
