# OnionCry MVP

Status: ready-for-agent

## Summary

OnionCry is a Rust CLI that checks JavaScript and TypeScript projects for architecture drift by analyzing import edges. The MVP uses a linter-style `.onioncryrc.jsonc` configuration, a pragmatic Clean Architecture preset, bounded context rules, external package allowlists, and pretty/JSON reporting suitable for local development and CI.

## Decisions

- The default config file is `.onioncryrc.jsonc`.
- Config and JSON output use camelCase fields.
- Rules use linter-style names such as `onion/no-layer-leak`.
- Rule severities are `off`, `warn`, and `error`.
- Overrides only change rule severity/options for matching files; they do not change file selection, aliases, layers, or contexts.
- The default layer preset is `domain`, `application`, `infra`, and `shared`.
- `infra` is the single outer layer for adapters, framework code, drivers, SDKs, controllers, repository implementations, queues, and runtime bootstrapping.
- Contexts represent bounded contexts or capability boundaries.
- Cross-context imports are only allowed through public surface path segments such as `contracts`, `events`, `ports`, and `shared`.
- External packages in sensitive layers use an allowlist policy: `domain` errors, `application` warns, and `infra` is off by default.
- The MVP uses `oxc_parser` for JavaScript and TypeScript import extraction.

## MVP Commands

- `onioncry check`
- `onioncry check --format json`
- `onioncry check --fail-on warning`
- `onioncry init`
- `onioncry explain <file>`

## Issue Breakdown

Implementation is split into ready-for-agent vertical slices under `issues/`, ordered by dependency.
