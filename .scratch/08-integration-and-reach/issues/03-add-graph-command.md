# Add `onioncry graph`

Status: done

## Parent

../PRD.md

## What to build

Add a `graph` subcommand that renders the project's dependency graph at the ownership-boundary level — contexts in Clean Architecture mode, slices in Vertical Slice mode — as Mermaid for humans and JSON for scripts and agents.

## Acceptance criteria

- [x] `onioncry graph` prints a Mermaid `graph TD` diagram of boundary-level dependencies; `--format json` prints a `nodes` / `edges` object with camelCase fields.
- [x] In Clean Architecture mode, nodes are architectural contexts; in Vertical Slice mode, nodes are slices.
- [x] Edges aggregate import edges between two boundaries and are labeled with the public surface segment they enter through when applicable.
- [x] Contextless (or slice-less) files aggregate into a single clearly named node.
- [x] Output is deterministic: nodes and edges are sorted for stable diffs and snapshot tests.
- [x] `--config` is honored like other commands; graph output goes to stdout only.
- [x] The command is diagnostic: it exits 0 regardless of violations and exits 2 only on config or analysis errors.
- [x] JSON structure (`nodes`, `edges`, edge `from`/`to`/`via`) is documented as public API in the README.
- [x] CLI integration tests cover both architecture modes, both formats, and empty-graph projects.
- [x] CONTEXT.md gains a Context Graph glossary entry.
- [x] `make verify` passes.

## Blocked by

None - can start immediately

## Comments

- 2026-07-04: Added `onioncry graph` with Mermaid and JSON output, deterministic context/slice node and edge aggregation, public-surface `via` labels, contextless/sliceless nodes, README JSON contract docs, and a Context Graph glossary entry. Verification: `rtk cargo test --test cli_graph` passed with 5 tests; `rtk make verify` passed with clippy clean and 115 tests. The exact commit SHA is reported by the implementation loop after the slice commit is created.

## Comments

Accepted API extensions beyond the original `from`/`to`/`via` contract, found
in spec review and kept deliberately: edge `importCount` (aggregated import
edge count, useful for weighting) and the `ambiguous` node kind (files matching
more than one boundary pattern surface as their own node instead of being
hidden). Both are documented in the README JSON contract.
