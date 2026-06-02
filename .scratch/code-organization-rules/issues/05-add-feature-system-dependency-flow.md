# Add `frontend/feature-system-dependency-flow`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a frontend feature system dependency flow rule that checks allowed import direction inside and around a feature system. The rule should prevent upper UI layers from shortcutting into adapters and prevent routes or other systems from depending on internals.

## Acceptance criteria

- [ ] `frontend/feature-system-dependency-flow` is accepted in the `rules` map with normal severity and option shapes.
- [ ] Adapters cannot import from hooks, components, routes, stores, or contexts.
- [ ] Components cannot import adapters directly.
- [ ] Routes import feature systems through public barrels rather than internal files.
- [ ] Query options are allowed to bridge adapters into the query layer.
- [ ] Contexts can import hooks and lib by default.
- [ ] Stores can import lib and adapters by default, but not components.
- [ ] Same-layer and allowed downward imports are accepted according to the configured flow.
- [ ] CLI tests cover allowed and disallowed imports across adapters, lib, hooks, contexts, stores, components, and routes.
- [ ] `make verify` passes.

## Blocked by

- 03-add-feature-system-layout.md
- 04-add-feature-system-public-api.md
