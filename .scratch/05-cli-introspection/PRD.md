# CLI Introspection

Status: ready-for-agent

## Summary

OnionCry will expose its rule catalog and configuration contract through the CLI so humans and agents can discover rules, defaults, and config shape without reading source code. This covers a new `onioncry rules` command and a published JSON Schema for `.onioncryrc.jsonc`.

## Decisions

- `onioncry rules` lists every rule from the built-in catalog: canonical name, default severity, architecture rule family (or architecture-neutral), and explanation. It supports `--format pretty` (default) and `--format json`.
- The rules listing is generated from the existing rule catalog (`src/rules/catalog.rs`); rule metadata is never duplicated by hand.
- Rule names, JSON field names, and the rules listing format are public API once shipped.
- The configuration JSON Schema is derived from the Rust config types with `schemars` so the schema cannot drift from the parser.
- `onioncry schema` prints the JSON Schema to stdout; `onioncry schema --write <path>` writes it to a file.
- The generated schema is committed at `docs/schema/onioncryrc.schema.json`, and a test regenerates it and fails when the committed file is stale.
- The `onioncry init` template references the schema with a `$schema` field pointing at the raw GitHub URL of the committed schema on `main`.
- Schema and rules output use camelCase fields, matching the Configuration Field Naming decision.
- New CONTEXT.md glossary entries are added for the rules listing and configuration schema concepts introduced here.

## Command Breakdown

- `onioncry rules [--format pretty|json]`
- `onioncry schema [--write <path>]`

## Dependencies

- `schemars` is the only new dependency; it is added with Cargo tooling and used directly by the config module.
