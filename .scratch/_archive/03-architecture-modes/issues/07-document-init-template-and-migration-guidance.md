# Document init template and migration guidance

Status: ready-for-agent

## Parent

../PRD.md

## User stories covered

- 31, 34

## What to build

Update the generated starter configuration and user-facing documentation so new projects see the architecture mode contract and existing projects can migrate without enabling both architecture-specific rule families. Keep the implementation issue queue aligned with the PRD so agents can pick up each slice without reopening the design discussion.

## Acceptance criteria

- [ ] `onioncry init` emits `architecture.mode` with `cleanArchitecture` as the default.
- [ ] The generated Clean Architecture options include default `contextRoot`, `layerPathAliases`, `artifactFolders`, and `artifactSuffixes`.
- [ ] The generated `rules` map includes `cleanarch/artifact-placement` at `warn`.
- [ ] Documentation explains that `cleanarch/*` and `verticalslice/*` are mutually exclusive by mode.
- [ ] Documentation includes a Clean Architecture context-first example and a Vertical Slice `features/<domain>/<operation>` example.
- [ ] Documentation explains `sliceDepth: 1` for projects that intentionally use `features/<feature>` or root-level feature folders.
- [ ] Documentation explains how to configure `modules`, `infrastructure`, and root-level slices when a project needs those path names.
- [ ] Documentation explains the `.service.ts` distinction between Clean Architecture and Vertical Slice.
- [ ] Documentation includes a migration note for projects with existing global use case, service, handler, or repository lists.
- [ ] The architecture modes PRD links or describes the final implementation issue queue in dependency order.
- [ ] Tests or snapshots cover the updated `init` template output.
- [ ] `make verify` passes.

## Blocked by

- 03-add-clean-architecture-artifact-placement.md
- 05-add-cross-slice-internal-import-rule.md
- 06-add-global-slice-artifacts-rule.md
