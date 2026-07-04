# Add `onioncry graph`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a `graph` subcommand that renders the project's dependency graph at the ownership-boundary level — contexts in Clean Architecture mode, slices in Vertical Slice mode — as Mermaid for humans and JSON for scripts and agents.

## Acceptance criteria

- [ ] `onioncry graph` prints a Mermaid `graph TD` diagram of boundary-level dependencies; `--format json` prints a `nodes` / `edges` object with camelCase fields.
- [ ] In Clean Architecture mode, nodes are architectural contexts; in Vertical Slice mode, nodes are slices.
- [ ] Edges aggregate import edges between two boundaries and are labeled with the public surface segment they enter through when applicable.
- [ ] Contextless (or slice-less) files aggregate into a single clearly named node.
- [ ] Output is deterministic: nodes and edges are sorted for stable diffs and snapshot tests.
- [ ] `--config` is honored like other commands; graph output goes to stdout only.
- [ ] The command is diagnostic: it exits 0 regardless of violations and exits 2 only on config or analysis errors.
- [ ] JSON structure (`nodes`, `edges`, edge `from`/`to`/`via`) is documented as public API in the README.
- [ ] CLI integration tests cover both architecture modes, both formats, and empty-graph projects.
- [ ] CONTEXT.md gains a Context Graph glossary entry.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately
