# Add `frontend/feature-system-layout`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a frontend feature system layout rule for domain-owned `systems/<domain>` modules. The default should match the maintainers' common frontend shape while allowing projects to override roots and allowed shared areas.

## Acceptance criteria

- [ ] `frontend/feature-system-layout` is accepted in the `rules` map with normal severity and option shapes.
- [ ] The default systems root is `packages/frontend/src/systems`.
- [ ] Each discovered feature system requires `components/`, `lib/`, and a root `index.ts`.
- [ ] Optional folders such as hooks, adapters, contexts, stores, and guards are recognized when present but are not required empty.
- [ ] Shared shadcn/ui primitives are allowed under `packages/frontend/src/components/ui` by default.
- [ ] Feature-specific frontend components outside a feature system or allowed shared UI root report violations.
- [ ] Legacy `features/<domain>` roots report violations by default when discovered.
- [ ] Surface CSS is optional; when present, it must live at the system root and follow the configured domain CSS naming convention.
- [ ] CLI tests cover valid layout, missing required folders or files, disallowed feature code outside systems, and option customization.
- [ ] `make verify` passes.

## Blocked by

- 02-add-repo-path-naming.md
