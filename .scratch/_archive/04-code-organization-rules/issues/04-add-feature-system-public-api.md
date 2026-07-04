# Add `frontend/feature-system-public-api`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a frontend feature system public API rule that keeps each system's root `index.ts` as an explicit contract. Other systems and routes should depend on that public barrel rather than importing system internals.

## Acceptance criteria

- [ ] `frontend/feature-system-public-api` is accepted in the `rules` map with normal severity and option shapes.
- [ ] Each feature system root `index.ts` rejects wildcard re-exports by default.
- [ ] Explicit named exports from the system barrel are accepted.
- [ ] Imports from outside a feature system to that system's internal files report violations.
- [ ] Imports within the same feature system can use internal files.
- [ ] Routes can import a feature system through its public barrel.
- [ ] Explicit options can customize route roots, system roots, and allowed public entry points.
- [ ] CLI tests cover wildcard exports, named exports, cross-system internal imports, route imports, and same-system imports.
- [ ] `make verify` passes.

## Blocked by

- 03-add-feature-system-layout.md
