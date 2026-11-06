---
task: task_01
spec: 0002-suffixed-artifacts-require-their-role-folder
status: pending
type: backend
complexity: medium
---

# Task 01: Report suffix-only artifacts outside their role folder

## Overview

Under the Context-First Clean Architecture Layout, `cleanarch/artifact-placement` accepts a file classified only by its Artifact Filename Suffix when it sits in its expected layer but outside every artifact folder. Add the missing folder check to the context-first branch and prove it through the real binary. This answers the `vortex` report of 2026-10-20 recorded in `docs/references/2026-11-06-artifact-suffix-placement-evidence.md`.

## Requirements

1. MUST reproduce the defect through the real CLI before changing production code: a suffix-only `.use-case.ts` directly in `src/contexts/<context>/application/` currently produces no `cleanarch/artifact-placement` diagnostic. This answers the `vortex` finding F-01 of 2026-10-20 (ai-memory message `01a17436-c533-7b12-81de-9ea0012c4e30`).
2. MUST make the context-first branch of the placement policy report an artifact that was classified by suffix alone (no artifact folder in its path) when its layer equals the expected layer, with the expected boundary `<context root>/<context>/<layer>/<role folder>`, as the TechSpec's System Architecture describes. A folder-classified artifact in its expected layer MUST stay unreported. Do not change the classifier, the location policy, the boundary renderer, the collector, the configuration shape or the diagnostic text.
3. MUST add the real-binary regression `check_reports_suffixed_context_artifacts_outside_role_folder` to `tests/clean_architecture.rs`, covering the reported `application/<name>.use-case.ts` case, a `.repository.ts` directly in `infra/`, an `.entity.ts` directly in `domain/` and a suffix-only file in a non-artifact subfolder of a layer. Assert the exact files, rule, severity and expected boundary, and a failing exit under `--fail-on warning`.
4. MUST add the real-binary negative control `check_accepts_suffixed_context_artifacts_inside_role_folders`: the same files inside their role folders and inside nested capability folders within them report nothing, and a per-file override that sets the rule `off` for one misplaced file silences only that file while another stays reported.
5. MUST keep every existing test in `tests/clean_architecture.rs` unchanged and passing. Edit `.agents/skills/onioncry/SKILL.md` only if it describes artifact placement in a way the fix makes untrue; keep its released-version metadata until the release bump.

## Subtasks

- [ ] Reproduce the false acceptance through the public binary.
- [ ] Add the suffix-only guard to the context-first branch.
- [ ] Add the positive and negative real-binary regressions.
- [ ] Confirm the canonical OnionCry skill still matches the behavior.

## Acceptance Criteria

- [ ] Every suffix-only artifact outside its role folder under the context-first layout is reported with its role folder as the expected boundary, for each default layer.
- [ ] Folder-placed artifacts, nested capability folders and per-file `off` overrides behave exactly as before.
- [ ] No configuration, schema, rule name, message or exit-code change.

## Context

- interface: `src/rules/clean_architecture/artifact_policy.rs`
- interface: `tests/clean_architecture.rs`
- interface: `.agents/skills/onioncry/SKILL.md`
- instruction: `src/rules/clean_architecture/artifact_classifier.rs`
- instruction: `src/rules/clean_architecture/location.rs`
- instruction: `src/rules/clean_architecture/boundary_renderer.rs`
- instruction: `tests/support/mod.rs`
- instruction: `docs/references/2026-11-06-artifact-suffix-placement-evidence.md`

## Consumers

- `cleanarch/artifact-placement` diagnostics — consumers: `--test clean_architecture`

## Verification

- `grep -q '^fn check_reports_suffixed_context_artifacts_outside_role_folder' tests/clean_architecture.rs && grep -q '^fn check_accepts_suffixed_context_artifacts_inside_role_folders' tests/clean_architecture.rs && rtk cargo test --test clean_architecture` — expected: exit 0; both new real-binary tests and every existing clean-architecture contract pass.

## References

- `_prd.md` → Goals 1 and 2; Core Features 1 and 2; Success Metrics 1 and 2.
- `_techspec.md` → API Contract 1; System Architecture; Implementation Design; Testing Approach; Build Order 1.
- ADR-0012; ADR-0008; ADR-0003.
