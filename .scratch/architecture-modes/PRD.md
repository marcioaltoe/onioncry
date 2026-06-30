# Architecture Modes

Status: ready-for-agent

## Summary

OnionCry will support one configured project architecture mode at a time. Clean Architecture remains the default when `architecture.mode` is omitted, while Vertical Slice becomes an explicit alternative with its own rule family and structure options.

## Decisions

- `architecture.mode` selects `cleanArchitecture` or `verticalSlice`.
- Missing `architecture.mode` means `cleanArchitecture`.
- Architecture-specific rule families are mutually exclusive. A rule from the wrong family is a configuration error.
- Mode-specific structural options live under `architecture.cleanArchitecture` or `architecture.verticalSlice`.
- Rule severities stay in `rules`.
- Clean Architecture defaults to context-first layout: `src/contexts/<context>/{domain,application,infra}` plus contextless base `src/{domain,application,infra}`.
- Clean Architecture layout options include `contextRoot`, `layerPathAliases`, `artifactFolders`, and `artifactSuffixes`.
- Clean Architecture structure validation is presence-based and does not require empty folders.
- `cleanarch/artifact-placement` defaults to `warn`.
- Vertical Slice defaults to `src/features/<feature>`.
- Vertical Slice layout options include `sliceRoot`, `publicSurface`, `artifactFolders`, `artifactSuffixes`, and `allowedGlobalFolders`.
- Vertical Slice public surface defaults to `index.ts` and `contracts/`.
- Vertical Slice internals default to `handlers/`, `adapters/`, `domain/`, and `__tests__/` when present.
- `.service.ts` is interpreted through its containing Clean Architecture layer, but is slice-internal by default in Vertical Slice mode.

## References

- [Architecture modes](../../docs/architecture-modes.md)
- [ADR 0007: Select one architecture mode per project](../../docs/adr/0007-select-one-architecture-mode-per-project.md)
- [ADR 0008: Use context-first Clean Architecture layout](../../docs/adr/0008-use-context-first-clean-architecture-layout.md)
- [ADR 0009: Use features as the default Vertical Slice root](../../docs/adr/0009-use-features-as-the-default-vertical-slice-root.md)

## Issue breakdown

Implementation is split into ready-for-agent issues under `issues/`, ordered by dependency.
