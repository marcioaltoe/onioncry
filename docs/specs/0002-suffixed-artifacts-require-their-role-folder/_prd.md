---
spec: 0002-suffixed-artifacts-require-their-role-folder
status: active
created: 2026-11-06
surfaces: [cli]
---

# Suffixed artifacts require their role folder

`cleanarch/artifact-placement` accepts a file that carries an Artifact Filename Suffix, such as `.use-case.ts`, when it sits directly in a context's layer root under the Context-First Clean Architecture Layout, for example `src/contexts/tax/application/probe.use-case.ts`. The same file without the suffix is reported. A team that raises the rule to `error` is therefore protected only against unsuffixed files. The defect was reported by `vortex` in ai-memory message `01a17436-c533-7b12-81de-9ea0012c4e30` (2026-10-20). Its evidence, the local reproduction and the root cause are in `docs/references/2026-11-06-artifact-suffix-placement-evidence.md`.

## Project Constraints

- Identifier strategy: not applicable — no persistent identifiers are introduced. Source: `docs/agents/domain.md`.
- Authentication and HTTP: not applicable — analysis remains a local, read-only CLI operation without authentication or HTTP API changes. Source: `docs/agents/cli.md`.
- Active ADR obligations: applicable — follow ADR-0012 (suffixes classify roles but never exempt placement), keep ADR-0008 (context-first layout, presence-based structure rules, suffixes help classify) and keep ADR-0003 (per-file overrides remain the way to relax the rule). Source: `docs/agents/domain.md`; `docs/adr/0012-artifact-suffixes-classify-roles-but-never-exempt-placement.md`; `docs/adr/0008-use-context-first-clean-architecture-layout.md`; `docs/adr/0003-use-linter-style-configuration.md`.
- Tooling authority: applicable — express maintainer authorization on 2026-11-06 through the question tool to implement, commit, push, open the PR, merge with approved CI and publish the release. Source: `docs/agents/agent-instructions.md`; Spec-contained authorization record: `docs/specs/0002-suffixed-artifacts-require-their-role-folder/_authorization.md`; bounded files: `src/rules/clean_architecture/artifact_policy.rs`, `tests/clean_architecture.rs`, `.agents/skills/onioncry/SKILL.md`, `Cargo.toml`, `Cargo.lock`, `npm/package.json`, `npm/platforms/darwin-arm64/package.json`, `npm/platforms/darwin-x64/package.json`, `npm/platforms/linux-arm64/package.json`, `npm/platforms/linux-x64/package.json`, `npm/platforms/win32-x64/package.json`. The manifest paths cover only the own-package version bump of the release choreography. No dependency or checker-configuration change is proposed.

## Goals

1. A file classified only by its Artifact Filename Suffix is reported by `cleanarch/artifact-placement` when it sits outside an artifact folder of its role's layer, under the Context-First Clean Architecture Layout, just like an unsuffixed file in the same place.
2. Every placement that is valid today stays valid, and every placement reported today keeps its diagnostic.
3. Deliver the correction in a stable release that consumers install from crates.io and npm.

## Core Features

1. Placement check for suffix-only artifacts on the context-first path, with the expected boundary naming the role's folder, such as `src/contexts/tax/application/use-cases`.
2. Real-binary regression tests for the reported case, for the other default roles, and for valid placements that must stay clean.
3. Delivery through terminal QA, the configured review, CI and the existing release choreography.

## Non-Goals

- Change rule names, flags, output schemas, exit codes, diagnostic message layout or the `.onioncryrc.jsonc` shape.
- Change the contextless base layout, the layer-first layout, Vertical Slice mode or `cleanarch/unclassified-file`.
- Classify artifacts by class or symbol names.

## Success Metrics

1. Success Metric: the reported `src/contexts/tax/application/<name>.use-case.ts` case and one suffix-only case per default role in another layer produce `cleanarch/artifact-placement` with the role folder as the expected boundary, and the run fails under `--fail-on warning`.
2. Success Metric: the existing `tests/clean_architecture.rs` suite passes unchanged, and suffixed files inside their role folders, inside nested capability folders within them, and inside overridden paths produce no placement diagnostic.
3. Success Metric: `make verify` and QA pass, and the released version is installable from crates.io and npm.

## Glossary

None.

## Decisions

- 2026-11-06: How far may the correction of `cleanarch/artifact-placement` for suffixed files go? — Up to the release: implement, commit, push, PR, merge with approved CI and publish the patch release on crates.io and npm.
- Autonomous route: the technical scope comes from the reported defect, ADR-0008, ADR-0003, the new ADR-0012 and source inspection at `887bc93`. No product behavior outside the existing rule contract is proposed.
