---
task: task_01
spec: 0002-suffixed-artifacts-require-their-role-folder
status: completed
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

## Result

Skills loaded: implement-task, coding-guidelines, rust-expert, rust-cli, testing-boss, systematic-debugging, no-workarounds, agentic-cli-design, onioncry, tech-writer, evidence-gate.

The context-first policy now accepts an artifact in its expected layer only when classification found an artifact folder. Suffix-only artifacts fall through to the existing finding constructor and boundary renderer. The production diff changes one condition; the classifier, location policy, renderer, collector, configuration, schema and diagnostic text remain unchanged.

Pre-change reproduction through the real binary:

- `rtk proxy cargo test --test clean_architecture suffixed_context_artifacts -- --nocapture` exited 101 before the production edit. The positive regression ran `onioncry check --format json --fail-on warning` over the reported application probe, an infra repository, a domain entity and an application file inside `billing/`. The binary incorrectly exited 0 with `status: pass`, four analyzed files and an empty violations array. The negative control's later override-isolation phase also incorrectly accepted the remaining misplaced repository.

Acceptance evidence from focused checks:

- Suffix-only placement: `check_reports_suffixed_context_artifacts_outside_role_folder` asserts exactly four reported files, `cleanarch/artifact-placement`, severity `warn`, expected layer, and exact existing message and suggestion containing each role boundary. It covers `application/use-cases`, `infra/repositories`, `domain/entities`, and a suffix-only file nested under a non-artifact `application/billing` folder. It asserts exit 1 under `--fail-on warning` and exit 0 with identical findings under the default threshold.
- Valid placement and overrides: `check_accepts_suffixed_context_artifacts_inside_role_folders` first asserts an empty violations array for six files directly inside role folders or nested capability folders. It then adds two misplaced files, one covered by a per-file `off` override, and asserts exactly one remaining repository finding at severity `error`. The fixtures cover both project-root `.` with `contextRoot: src/contexts` and project-root `src` with `contextRoot: contexts`.
- Public contracts: the tests use the existing configuration shape and assert the existing rule, diagnostic wording, boundary rendering, severities and exit thresholds. Existing tests in `tests/clean_architecture.rs` were left byte-identical; only the two new tests were inserted. Inspection of `.agents/skills/onioncry/SKILL.md` found no artifact-placement description made untrue by this guard, so it remains unchanged with released-version metadata `0.2.1`.

Focused check outcomes:

- After the production edit, `rtk proxy cargo test --test clean_architecture suffixed_context_artifacts -- --nocapture`: exit 0, two tests passed.
- After formatting, `rtk proxy cargo test --test clean_architecture artifact`: exit 0, eight tests passed (both new tests and six existing artifact contracts); thirteen unrelated tests filtered out.
- `rtk proxy rustfmt --edition 2024 --check src/rules/clean_architecture/artifact_policy.rs tests/clean_architecture.rs`: exit 0 after applying the formatter's line wrap to a new assertion.
- `rtk proxy git -c core.fsmonitor=false diff --check`: exit 0.

The declared Task Verification, full clean-architecture suite, `make rust-verify` and `make verify` were not run; the Daemon owns them and the terminal verdict. Task status, Task Graph and sibling Tasks were not edited. No commit, push or pull request was created. No follow-up implementation was added.
