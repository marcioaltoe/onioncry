# Add `onioncry rules`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a `rules` subcommand that lists every rule in the built-in catalog so humans and agents can discover rule names, defaults, and meanings without reading source code. Output is generated from `src/rules/catalog.rs`; rule metadata must not be duplicated by hand.

## Acceptance criteria

- [ ] `onioncry rules` prints every catalog rule with canonical name, default severity, architecture rule family (or `neutral`), and explanation.
- [ ] `onioncry rules --format json` emits a serde-serialized array with camelCase fields (`name`, `defaultSeverity`, `architectureFamily`, `explanation`); no hand-built JSON.
- [ ] Pretty output groups rules by family (`cleanarch/*`, `verticalslice/*`, neutral families) and stays stable and deterministic.
- [ ] Legacy rule aliases are listed on their canonical rule entry.
- [ ] The command needs no configuration file and exits 0.
- [ ] Help text for the new subcommand is concise and truthful.
- [ ] CLI integration tests cover pretty output, JSON output, and exit code.
- [ ] README documents the command.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately
