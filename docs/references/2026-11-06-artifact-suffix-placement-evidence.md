# Artifact suffix placement evidence (2026-11-06)

## Reported defect

The Vortex backend runs OnionCry 0.1.0 in `cleanArchitecture` mode with `contextRoot: "src/contexts"`, and both `cleanarch/artifact-placement` and `cleanarch/unclassified-file` set to `error`. Its QA for Spec 0152 on 2026-10-20 created `src/contexts/tax/application/qa-placement-probe.use-case.ts` directly in the `application` layer root, outside `use-cases`. `onioncry check --llm-mode` exited 0 with no placement problem. The same file named `qa-placement-probe.ts` failed with `cleanarch/artifact-placement`. The report arrived as ai-memory message `01a17436-c533-7b12-81de-9ea0012c4e30` from `vortex`, dated 2026-10-20 (Vortex QA finding F-01, Vortex PR #481).

## Local reproduction on `main` at `887bc93`

On 2026-11-06 a temporary project generated with `onioncry init`, with `contextRoot` set to `src/contexts` and both rules set to `error`, reproduced the defect. `src/contexts/tax/application/qa-placement-probe.use-case.ts` produced no `cleanarch/artifact-placement` diagnostic. Renaming it to `qa-placement-probe.ts` produced one, with the tip `move this application artifact to src/contexts/tax/application/<artifact-folder>`.

## Root cause

`src/rules/clean_architecture/artifact_classifier.rs` classifies a file by the nearest configured artifact folder in its path. Failing that, it falls back to the configured suffixes and records no folder (`folder_index: None`). In `src/rules/clean_architecture/artifact_policy.rs`, `CleanArtifactPlacementPolicy::finding` returns no finding on the context-first path whenever the file's layer equals the expected layer, and it never checks that a suffix-only artifact sits in an artifact folder. The contextless base layout reaches `contextless_base_capability_boundary`, which reports the same file, and an unsuffixed file reaches `layer_direct_folder_finding`, which reports it too. Only the context-first branch skips the folder check.

## Outside source: element identity comes from folders

Source read on 2026-11-06: JS Boundaries (`eslint-plugin-boundaries`) element descriptor documentation, https://www.jsboundaries.dev/docs/classification/elements/ . The page says: "Element patterns should match folders, not individual files. Do not include file extensions in element patterns. To classify individual files by kind, use file descriptors." In that widely used JavaScript architecture linter, a file's architectural element, its placement, is decided by the folder that contains it. A file-name kind classifies the file but does not place it. This supports ADR-0012: the suffix tells OnionCry the role, and the folder must still be the role's folder.

QA outside-evidence criterion: for `src/contexts/tax/application/<name>.use-case.ts`, the role inferred from the suffix (`use-cases`) and the folder that actually contains the file (`application`) disagree. The binary must report `cleanarch/artifact-placement` with the expected boundary `src/contexts/tax/application/use-cases`. The same file under `application/use-cases/` must report nothing. Inspect the real CLI JSON for both.

## Consultation limits

Secondbrain: `~/dev/secondbrain/wiki/index.md` was read, and `qmd query "onioncry artifact placement suffix use-case folder"` returned only this project's mirrors (`projects/onioncry/mirror/docs/adr/0008-use-context-first-clean-architecture-layout.md`, `projects/onioncry/mirror/docs/adr/0003-use-linter-style-configuration.md`, `projects/onioncry/mirror/CONTEXT.md`). They are byte-identical to the local originals, so they confirm ADR-0008 and ADR-0003 as read here and add no prior decision. Exa MCP returned the JS Boundaries documentation cited above. No Secondbrain file was written.
