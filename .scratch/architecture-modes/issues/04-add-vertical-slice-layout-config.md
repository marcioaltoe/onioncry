# Add Vertical Slice layout configuration

Status: ready-for-agent

## Parent

../PRD.md

## User stories covered

- 23, 24, 25, 26, 29, 31, 33

## What to build

Add Vertical Slice mode defaults and effective configuration for slice root, public surface, artifact folders, artifact suffixes, and allowed global folders. This slice establishes classification data used by later Vertical Slice rules without adding cross-slice violations yet.

## Acceptance criteria

- [ ] `architecture.verticalSlice.sliceRoot` defaults to `features`.
- [ ] `sliceRoot` accepts alternatives such as `slices`, `modules`, and `.`.
- [ ] `publicSurface` defaults to `index.ts` and `contracts`.
- [ ] `artifactFolders` defaults to `handlers`, `adapters`, `domain`, and `__tests__`.
- [ ] `artifactSuffixes` supports configured suffixes such as `.repository.ts`, `.service.ts`, `.handler.ts`, `.adapter.ts`, `.entity.ts`, `.value-object.ts`, and `.use-case.ts`.
- [ ] `allowedGlobalFolders` supports project-approved global folders such as `app`, `config`, `lib`, `shared`, and technical `infra`.
- [ ] Effective configuration can classify a source file as inside a slice, on the slice public surface, or outside the slice root.
- [ ] `.service.ts` is classified as slice-internal unless it is exposed through the configured public surface.
- [ ] Vertical Slice mode does not run Clean Architecture layer checks.
- [ ] Tests cover default features root, alternate roots, root-level slices, public surface classification, internal file classification, and allowed global folders.
- [ ] `make verify` passes.

## Blocked by

- 01-add-architecture-mode-config.md
- 02-reject-architecture-rule-mode-mismatch.md
