# Add configuration JSON Schema and `onioncry schema`

Status: done

## Parent

../PRD.md

## What to build

Derive a JSON Schema for `.onioncryrc.jsonc` from the Rust configuration types with `schemars`, expose it through an `onioncry schema` subcommand, commit the generated schema, and reference it from the `init` template so editors validate and autocomplete the config.

## Acceptance criteria

- [ ] `schemars` is added via Cargo tooling and derived on the configuration types in `src/config/types.rs` (and nested option types) so the schema cannot drift from the parser.
- [ ] `onioncry schema` prints the JSON Schema to stdout; `onioncry schema --write <path>` writes it to a file and prints the created path.
- [ ] The generated schema is committed at `docs/schema/onioncryrc.schema.json`.
- [ ] A test regenerates the schema and fails with a useful message when the committed file is stale.
- [ ] The `onioncry init` template includes a `$schema` field pointing at the raw GitHub URL of the committed schema on `main`.
- [ ] Schema fields are camelCase and match the parser's accepted shapes, including `[severity, options]` rule tuples and overrides.
- [ ] CLI integration tests cover stdout output, `--write`, and exit codes.
- [ ] README documents the command and the `$schema` editor support.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately

## Comments

- 2026-07-04: Implemented `onioncry schema` with stdout and `--write <path>` modes, generated `docs/schema/onioncryrc.schema.json`, added `schemars` through Cargo, and updated the `init` template to reference the raw GitHub schema URL on `main`. Added CLI integration coverage for schema stdout, schema writes, committed-schema freshness, and init `$schema` output. Verification passed with `rtk cargo test --test cli_schema --test cli_init`, `rtk cargo check --all-targets --all-features`, `rtk cargo clippy --all-targets --all-features -- -D warnings`, `rtk cargo test --all-features`, and `rtk make verify`. The exact commit SHA is reported in the loop handoff after the issue commit is created.
