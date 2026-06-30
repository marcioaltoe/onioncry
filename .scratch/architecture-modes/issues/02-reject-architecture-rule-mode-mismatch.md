# Reject architecture rule mode mismatch

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Fail configuration validation when a project enables a rule from an architecture-specific rule family that does not match `architecture.mode`. This prevents mixed Clean Architecture and Vertical Slice validation from running silently.

## Acceptance criteria

- [ ] A `verticalSlice` config that enables any `cleanarch/*` rule fails before analysis.
- [ ] A `cleanArchitecture` config that enables any `verticalslice/*` rule fails before analysis.
- [ ] Architecture-neutral rules remain valid in either mode.
- [ ] The error message names the incompatible rule and the configured architecture mode.
- [ ] Overrides are checked for the same mismatch behavior as top-level `rules`.
- [ ] Tests cover top-level rule mismatch, override rule mismatch, and valid neutral rules in both modes.
- [ ] `make verify` passes.

## Blocked by

- 01-add-architecture-mode-config.md
