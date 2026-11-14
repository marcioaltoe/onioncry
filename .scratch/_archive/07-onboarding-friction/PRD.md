# Onboarding Friction

Status: ready-for-agent

## Summary

OnionCry will reduce the two highest-friction moments in adoption: writing the `aliases` block by hand in projects that already declare TypeScript path mappings, and running whole-project checks when only a few files changed. This covers `onioncry init --from-tsconfig` and `onioncry check --files`.

## Decisions

### Aliases from tsconfig

- `onioncry init --from-tsconfig [path]` generates the `aliases` block of the configuration template from the tsconfig `compilerOptions.paths` and `baseUrl`. The path argument defaults to `tsconfig.json` in the project root.
- This is explicit generation at init time for team review, not runtime inference. The committed OnionCry configuration remains the single source of truth for alias resolution, preserving the existing Alias Mapping decision.
- Wildcard mappings translate directly: `"@/*": ["./src/*"]` with the tsconfig's `baseUrl` becomes `"@/": "src/"` (paths normalized relative to the OnionCry project root).
- Entries that cannot be expressed as a prefix mapping — non-wildcard keys, multiple targets, targets outside the project root — are skipped and listed in a template comment so the team resolves them manually.
- tsconfig files are parsed as JSONC with the same tolerant parsing already used for `.onioncryrc.jsonc`. The MVP does not follow `extends`; when `extends` is present, the generated comment says so.
- The decision is captured as ADR `docs/adr/0011-generate-aliases-from-tsconfig-at-init.md`.

### File-scoped check

- `onioncry check --files <path>...` accepts one or more project-relative file paths and filters the report to violations located in those files.
- Analysis stays whole-project: the file universe, import graph, and boundary classification are computed exactly as in an unscoped run. Scoping filters the report, not the analysis, so results never differ from a full run for the same file.
- Project-level violations that have no single file location, such as `cleanarch/no-context-cycle` findings, are always reported regardless of `--files`.
- Paths outside the file universe are reported on stderr as skipped, without failing the run, so pre-commit hooks can pass staged paths verbatim.
- Exit code follows the filtered report and the effective failure threshold.
- `--files` composes with `--format`, `--llm-mode`, `--tips`, and the baseline flags.
- New CONTEXT.md glossary entries are added for File-Scoped Check and the tsconfig generation concept.

## Issue Breakdown

1. `onioncry init --from-tsconfig`
2. `onioncry check --files`
