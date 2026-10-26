---
spec: 0001-feature-system-overrides-and-error-reexports
status: active
created: 2026-10-26
surfaces: [cli]
---

# Honor feature-system overrides and named Error reexports

Feature-system collectors currently reuse the first enabled file's RuleSetting for the entire project. A narrow export-hook exemption therefore either fails to apply or can affect unrelated production files. The adapter contract also checks source substrings rather than the actual exported class, rejecting valid named Error reexports and accepting misleading unrelated text.

## Project Constraints

- Identifier strategy: not applicable — no persistent identifiers are introduced. Source: `docs/agents/domain.md`.
- Authentication and HTTP: not applicable — analysis remains a local, read-only CLI operation without authentication or HTTP API changes. Source: `docs/agents/cli.md`.
- Active ADR obligations: applicable — preserve ADR-0003 last-matching per-file rule replacement and ADR-0004 Oxc parsing; do not replace the parser or change configuration shape. Source: `docs/agents/domain.md`; `docs/adr/0003-use-linter-style-configuration.md`; `docs/adr/0004-use-oxc-parser-for-javascript-and-typescript.md`.
- Tooling authority: applicable — express maintainer authorization on 2026-10-26 through the question tool to correct the two confirmed defects, test, review, commit, push, open PRs, merge with approved CI and publish a stable release for the consumer. Source: `docs/agents/agent-instructions.md`; Spec-contained authorization record: `docs/specs/0001-feature-system-overrides-and-error-reexports/_authorization.md`; bounded files: `src/rules/feature_system/mod.rs`, `src/rules/feature_system/helpers.rs`, `src/rules/feature_system/adapter_contract.rs`, `src/rules/feature_system/error_exports.rs`, `tests/feature_system.rs`, `.agents/skills/onioncry/SKILL.md`, `Cargo.toml`, `Cargo.lock`, `npm/package.json`, `npm/platforms/darwin-arm64/package.json`, `npm/platforms/darwin-x64/package.json`, `npm/platforms/linux-arm64/package.json`, `npm/platforms/linux-x64/package.json`, `npm/platforms/win32-x64/package.json`. Regenerate Cargo.lock through Cargo; follow the public release plan and existing release choreography for all version pins. No dependency replacement or checker-configuration relaxation is proposed.

## Goals

1. Honor each diagnostic owner's effective rule severity and options without losing cross-file relationships.
2. Recognize genuine named runtime Error class exports across local reexport chains while rejecting unrelated values and cycles.
3. Deliver a verified correction and a stable consumable release through the existing process.

## Core Features

1. Per-file policy: correct all five feature-system collectors that share the same first-setting defect, preserving last-matching-override replacement and the existing diagnostic owners.
2. Export proof: use the existing parser and resolved local import graph to follow the exact requested named binding; never infer an Error class from comments or unrelated declarations.
3. Delivery: real CLI regression tests, terminal QA, configured review, CI and stable registry artifacts.

## Non-Goals

- Add CLI flags, output fields, rule names, parser dependencies, an architecture baseline or broad exceptions.
- Become a general TypeScript type checker or execute analyzed source.
- Promise wildcard/default export resolution or indirect superclass inference. Preserve direct named Error subclasses and add the named local/reexport forms specified below.
- Change namespace-object detection, cancellation semantics, file selection, import resolution or domain diagnostic ownership.

## Success Metrics

1. Success Metric: five collector families respect file-specific off/warn/error and option replacement, and an exempt file cannot silence another production violation.
2. Success Metric: valid named Error class reexports pass while all declared invalid cases remain reported, including unresolved or excluded targets and cycles.
3. Success Metric: make verify and QA pass; the approved released artifact is consumable from its public registry.

## Glossary

None.

## Decisions

- 2026-10-26: Authorize the owning-tool correction and stable publication? — The maintainer approved both confirmed defects and their tested/reviewed delivery and publication in the question tool.
- Autonomous route: technical scope derives from the maintainer's confirmed defects, ADR-0003, ADR-0004, existing CLI contracts and source inspection at db73399; no product behavior outside those contracts is proposed.
