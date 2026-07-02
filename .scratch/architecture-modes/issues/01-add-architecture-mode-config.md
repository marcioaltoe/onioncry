# Add architecture mode configuration

Status: ready-for-agent

## Parent

../PRD.md

## User stories covered

- 1, 2, 3, 5, 33

## What to build

Add `architecture.mode` to the OnionCry configuration contract and apply `cleanArchitecture` as the effective default when the field is omitted. This slice must expose the effective architecture mode to configuration validation, rule selection, diagnostics, and JSON output paths that need configuration context.

## Acceptance criteria

- [ ] `.onioncryrc.jsonc` accepts `architecture.mode` with `cleanArchitecture` and `verticalSlice`.
- [ ] A config without `architecture.mode` behaves as `cleanArchitecture`.
- [ ] Unknown architecture mode values fail configuration validation with an actionable message.
- [ ] Mode-specific option objects are accepted under `architecture.cleanArchitecture` and `architecture.verticalSlice`.
- [ ] Mode-specific options do not change rule severity; severities continue to come from `rules`.
- [ ] The effective mode is available to rule policy, override policy, pretty diagnostics, and JSON diagnostics without breaking existing output contracts.
- [ ] Pretty diagnostics and JSON output remain backward-compatible except for intentional architecture-mode metadata.
- [ ] Tests cover explicit Clean Architecture mode, explicit Vertical Slice mode, missing mode default, and invalid mode.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately
