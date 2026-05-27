# Explain one file

Status: ready-for-agent

## What to build

Implement `onioncry explain <file>` as an interactive diagnostic report for one source file. This slice should show how OnionCry classified the file, which patterns matched, how imports resolved, and which violations apply to that file.

## Acceptance criteria

- [ ] `onioncry explain <file>` loads the effective config using the same discovery and `--config` behavior as `check`.
- [ ] The command reports the file's layer, context when present, public surface status, and matched patterns.
- [ ] The command lists imports with resolved target paths when local resolution succeeds.
- [ ] The command identifies external package imports, normalized package names, and whether package policy allows them.
- [ ] The command lists unresolved local imports.
- [ ] The command lists violations for the file with rule name, severity, reason, and suggestion when available.
- [ ] The command exits `0` even when the explained file has violations.
- [ ] Tests cover classified files, unclassified files, contextless files, external packages, unresolved imports, and files with multiple violations.
- [ ] `make verify` passes.

## Blocked by

- 03-enforce-layer-boundaries.md
- 05-enforce-external-package-policy.md
- 06-enforce-bounded-context-public-surface.md
- 07-detect-circular-dependencies.md
