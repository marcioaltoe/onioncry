# Bootstrap CLI check

Status: ready-for-agent

## What to build

Create the first usable OnionCry CLI path: `onioncry check` loads `.onioncryrc.jsonc`, selects files from the configured file universe, and emits a pass/fail result with pretty and JSON-capable reporting scaffolding. This slice should prove the end-to-end CLI shape without implementing architecture rules yet.

## Acceptance criteria

- [ ] `onioncry check` discovers `.onioncryrc.jsonc` by default and accepts `--config <path>`.
- [ ] The config loader supports JSONC, `version`, `project.root`, `project.include`, `project.exclude`, `rules`, and `overrides` fields needed by later slices.
- [ ] File selection applies include/exclude before any rule evaluation.
- [ ] `--format pretty` and `--format json` are accepted, with a minimal valid result containing `status`, `summary`, and `violations`.
- [ ] `--fail-on error` is the default, and `--fail-on warning` is accepted.
- [ ] Tests cover config discovery, explicit config path, missing config, include/exclude selection, and status/exit-code behavior with an empty violation set.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately
