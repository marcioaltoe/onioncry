# Enforce external package policy

Status: ready-for-agent

## What to build

Implement `onion/no-forbidden-imports` for external package imports using default-deny allowlists in sensitive layers. This slice should let teams explicitly allow domain-safe packages like `uuid` while blocking framework, persistence, network, runtime, and infrastructure concerns from core layers.

## Acceptance criteria

- [ ] External package imports are distinguished from local relative and configured-alias imports.
- [ ] Runtime built-ins such as `node:fs`, `node:path`, and `node:crypto` are treated as external imports for policy purposes.
- [ ] Package matching uses normalized package names: `uuid/v4` -> `uuid`, `@scope/pkg/subpath` -> `@scope/pkg`.
- [ ] Allowlist entries support exact package names and glob patterns such as `@aws-sdk/*`.
- [ ] Rule options support layer-specific entries with `fromLayer`, `severity`, and `allow`.
- [ ] Layer-specific severity wins for matching entries, enabling `domain: error`, `application: warn`, and `infra: off`.
- [ ] Local aliases are not checked by the external package allowlist.
- [ ] Tests cover domain allowlisted packages, domain blocked packages, application warnings, infra off, built-ins, scoped packages, subpaths, and glob allowlist entries.
- [ ] `make verify` passes.

## Blocked by

- 04-apply-linter-style-rule-policy.md
