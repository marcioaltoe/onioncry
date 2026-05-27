# task-009: Explain one file

## Description

Implement onioncry explain <file> as an interactive diagnostic report for one source file, showing classification, matched patterns, import resolution, package policy, unresolved imports, and violations.

## Acceptance

- onioncry explain <file> loads the effective config using the same discovery and --config behavior as check.
- The command reports the file layer, context when present, public surface status, and matched patterns.
- The command lists imports with resolved target paths when local resolution succeeds.
- The command identifies external package imports, normalized package names, and whether package policy allows them.
- The command lists unresolved local imports.
- The command lists violations for the file with rule name, severity, reason, and suggestion when available.
- The command exits 0 even when the explained file has violations.
- Tests cover classified files, unclassified files, contextless files, external packages, unresolved imports, and files with multiple violations.
- make verify passes.
