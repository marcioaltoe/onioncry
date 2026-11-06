---
spec: 0002-suffixed-artifacts-require-their-role-folder
prd: _prd.md
created: 2026-11-06
---

# Suffix-only artifact placement under the context-first layout — Technical Spec

## Executive Summary

Close the one branch of `CleanArtifactPlacementPolicy::finding` that accepts a suffix-only artifact without checking its folder. When the context-first path finds the file in its expected layer but the artifact was classified by suffix alone, with no artifact folder in its path, return a finding whose expected boundary is `<context root>/<context>/<layer>/<role folder>`. Every other branch already reports this placement, so the fix is one guard plus real-binary regressions. No configuration, schema or output layout changes.

## Project Constraints

- Identifier strategy: not applicable — no persistent identifiers are introduced. Source: `docs/agents/domain.md`.
- Authentication and HTTP: not applicable — analysis remains a local, read-only CLI operation without authentication or HTTP API changes. Source: `docs/agents/cli.md`.
- Active ADR obligations: applicable — follow ADR-0012 (suffixes classify roles but never exempt placement), keep ADR-0008 (context-first layout, presence-based structure rules, suffixes help classify) and keep ADR-0003 (per-file overrides remain the way to relax the rule). Source: `docs/agents/domain.md`; `docs/adr/0012-artifact-suffixes-classify-roles-but-never-exempt-placement.md`; `docs/adr/0008-use-context-first-clean-architecture-layout.md`; `docs/adr/0003-use-linter-style-configuration.md`.
- Tooling authority: applicable — express maintainer authorization on 2026-11-06 through the question tool to implement, commit, push, open the PR, merge with approved CI and publish the release. Source: `docs/agents/agent-instructions.md`; Spec-contained authorization record: `docs/specs/0002-suffixed-artifacts-require-their-role-folder/_authorization.md`; bounded files: `src/rules/clean_architecture/artifact_policy.rs`, `tests/clean_architecture.rs`, `.agents/skills/onioncry/SKILL.md`, `Cargo.toml`, `Cargo.lock`, `npm/package.json`, `npm/platforms/darwin-arm64/package.json`, `npm/platforms/darwin-x64/package.json`, `npm/platforms/linux-arm64/package.json`, `npm/platforms/linux-x64/package.json`, `npm/platforms/win32-x64/package.json`. The manifest paths cover only the own-package version bump of the release choreography. No dependency or checker-configuration change is proposed.

## System Architecture

`CleanArtifactClassifier::classify` returns a `CleanArtifact` whose `folder_index` is `Some` when a configured artifact folder appears in the path and `None` when only a configured suffix matched. `CleanArtifactPlacementPolicy::finding` then chooses one of four location branches: context-first, context missing layer, layer-first (including the contextless base layer), or source-prefix fallback.

In the context-first branch the policy returns `None` as soon as the file's layer equals the expected layer. For a folder-classified artifact that is correct: the folder already sits under the expected layer. For a suffix-only artifact it skips the folder check entirely. That is the defect documented in `docs/references/2026-11-06-artifact-suffix-placement-evidence.md`. The contextless base branch reaches `contextless_base_capability_boundary`, which reports the same file, and an unsuffixed file reaches `layer_direct_folder_finding`, which reports it too.

The correction lives entirely in `src/rules/clean_architecture/artifact_policy.rs`. In the context-first branch, when `layer == expected_layer` and `artifact.folder_index` is `None`, return a `CleanArtifactFinding` with the artifact's role, the expected layer and `boundary_renderer.expected_boundary(Some(&context), &expected_layer, &artifact.role)`. Keep returning `None` when `folder_index` is `Some`. The classifier, the location policy, the boundary renderer and the collector stay unchanged. Severity and override selection keep coming from `RulePolicy::effective_rule` per file in `collect_clean_artifact_placement_violations`.

## Implementation Design

### Interfaces

No signature changes. The guard is a private branch inside `CleanArtifactPlacementPolicy::finding`:

```rust
if layer == expected_layer {
    if artifact.folder_index.is_some() {
        return None;
    }
    return Some(CleanArtifactFinding { /* role, expected_layer, expected_boundary(Some(&context), …) */ });
}
```

Invariants:

1. A path containing a configured artifact folder is classified by that folder, so the new guard never fires for it.
2. Index files never carry an artifact suffix under the default configuration, so they keep their current treatment.
3. Severity `off` from a per-file override still skips the file before the policy runs.

### Data Models

None. `CleanArtifact` and `CleanArtifactFinding` are unchanged private types.

### API Contracts

1. API Contract: `check` and `explain` keep their commands, streams, JSON/SARIF/LLM schemas, rule name `cleanarch/artifact-placement`, diagnostic message and tip layout, and the 0/1/2 exit-code contract. The only observable change is that a suffix-only artifact outside its role folder under the context-first layout now produces the existing `cleanarch/artifact-placement` diagnostic, at the configured severity, with the expected boundary `<context root>/<context>/<layer>/<role folder>`.

### Surface Transcripts

None. No new literal output is promised; the existing diagnostic message and tip render the new finding.

## Coverage Map

- Goal 1; Core Feature 1; Success Metric 1 → the suffix-only guard in the context-first branch of `CleanArtifactPlacementPolicy::finding`.
- Goal 2; Core Feature 2; Success Metric 2 → the unchanged folder-classified branch plus real-binary negative controls in `tests/clean_architecture.rs`.
- Goal 3; Core Feature 3; Success Metric 3 → terminal CLI QA, the configured review, current-head CI and the release choreography in `docs/release.md` after the graph.
Deviations from the PRD: None.

## Absence Evidence

1. Folder check for suffix-only artifacts on the context-first path — command: `rg -n 'folder_index' src/rules/clean_architecture`; result: `folder_index` is read only by `is_flat_grouped_artifact`, `contextless_base_capability_boundary` and `layer_first_context_candidate`, all on the layer-first branch; the context-first branch returns `None` on a layer match without reading it.

## ADR Alternatives

- ADR-0003: keep per-file overrides as the only way to relax the rule; do not add a new option that exempts suffixed files.
- ADR-0008: keep presence-based placement and the context-first layout; the suffix still classifies the role, as ADR-0008 says, and the folder still decides placement.
- ADR-0012: take the decision it records and reject the alternative it names, which is treating a suffix as placement proof in flat layouts.

## Integration Points

Analysis remains local and read-only. Registry publication happens only through `docs/release.md` after approved review, required checks, synchronized versions and dry-runs. No consumer repository code is copied into tests.

## Testing Approach

Extend `tests/clean_architecture.rs` with real-binary cases built on its existing `write_architecture_mode_config`, `write_file` and `run_json_check` helpers.

- Positive: `src/contexts/tax/application/qa-placement-probe.use-case.ts` with `contextRoot: "contexts"` and the rule at `error` reports `cleanarch/artifact-placement` with the expected boundary `src/contexts/tax/application/use-cases` and fails the run. Add one suffix-only case per other default layer: a `.repository.ts` directly in `infra/` and an `.entity.ts` directly in `domain/`. Add a suffix-only file nested in a non-artifact subfolder of the layer, such as `application/billing/<name>.use-case.ts`.
- Negative: the same files inside `application/use-cases/`, `infra/repositories/` and `domain/entities/`, and inside nested capability folders under them, report nothing. A per-file override that sets the rule `off` for the probe path silences only that file while another misplaced file stays reported.
- The existing `check_accepts_context_first_clean_architecture_artifact_placement` and the rest of the suite stay unchanged and passing.

The QA gate's outside-evidence row compares the reported case with the JS Boundaries element-descriptor guidance quoted in `docs/references/2026-11-06-artifact-suffix-placement-evidence.md`, then inspects the binary's JSON for the suffixed and folder-placed variants.

## Build Order

1. Add the suffix-only guard and the real-binary regressions, and confirm the canonical OnionCry skill still describes the shipped behavior.
2. Run the authored terminal CLI QA (depends on: 1).
3. After the graph, review and merge through the Delivery Queue, then run the public release plan and the release choreography in `docs/release.md`.

## Risks & Considerations

Consumers with suffixed files directly in a context layer will see new `cleanarch/artifact-placement` diagnostics. The rule defaults to `warn`, so only projects that raised it to `error` fail, and those projects asked for this protection. The release notes must say so and point to per-file overrides for a gradual migration. Do not touch the classifier: making suffix matches record a folder would change the layer-first branches that already behave correctly.

## Glossary

None.

## Research

- Local source inspection and a sandbox reproduction at `887bc93` established the root cause. See `docs/references/2026-11-06-artifact-suffix-placement-evidence.md`.
- Secondbrain: `~/dev/secondbrain/wiki/index.md` was read, and `qmd query "onioncry artifact placement suffix use-case folder"` returned only this project's mirrors (`projects/onioncry/mirror/docs/adr/0008-use-context-first-clean-architecture-layout.md`, `projects/onioncry/mirror/docs/adr/0003-use-linter-style-configuration.md`, `projects/onioncry/mirror/CONTEXT.md`). They are byte-identical to the local originals, so they confirm ADR-0008 and ADR-0003 as read here and add no prior decision.
- Exa MCP: the JS Boundaries element-descriptor documentation (https://www.jsboundaries.dev/docs/classification/elements/) says that element patterns should match folders and that file-name kinds classify files. That supports ADR-0012's split between role classification and placement.

## Decisions

- 2026-11-06: the folder decides placement and the suffix only classifies the role (ADR-0012). The fix is a guard on the context-first branch, not a classifier change.
