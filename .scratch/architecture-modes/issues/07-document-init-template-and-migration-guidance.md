# Document init template and migration guidance

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Update the generated starter configuration and user-facing documentation so new projects see the architecture mode contract and existing projects can migrate without enabling both architecture-specific rule families.

## Acceptance criteria

- [ ] `onioncry init` emits `architecture.mode` with `cleanArchitecture` as the default.
- [ ] The generated Clean Architecture options include default `contextRoot`, `layerPathAliases`, `artifactFolders`, and `artifactSuffixes`.
- [ ] The generated `rules` map includes `cleanarch/artifact-placement` at `warn`.
- [ ] Documentation explains that `cleanarch/*` and `verticalslice/*` are mutually exclusive by mode.
- [ ] Documentation includes a Clean Architecture context-first example and a Vertical Slice `features/<feature>` example.
- [ ] Documentation explains how to configure `modules`, `infrastructure`, and root-level slices when a project needs those path names.
- [ ] Documentation explains the `.service.ts` distinction between Clean Architecture and Vertical Slice.
- [ ] Documentation includes a migration note for projects with existing global use case or repository lists.
- [ ] Tests or snapshots cover the updated `init` template output.
- [ ] `make verify` passes.

## Blocked by

- 01-add-architecture-mode-config.md
- 03-add-clean-architecture-artifact-placement.md
- 04-add-vertical-slice-layout-config.md
