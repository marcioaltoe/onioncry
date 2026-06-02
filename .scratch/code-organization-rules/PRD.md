# Code Organization Rules

Status: ready-for-agent

## Summary

OnionCry will add built-in, configurable code organization rules for observable repository conventions. The first rule set covers test placement, path naming, and frontend feature system contracts for projects that use Tailwind CSS, shadcn/ui, and `systems/<domain>` frontend modules.

## Decisions

- Code organization rules check observable project state, not whether contributors followed an agent skill or workflow.
- Rule option defaults may be project-focused and forward-looking while remaining overridable through explicit rule options and overrides.
- `repo/test-placement` defaults to unit tests in co-located `__tests__` folders, with integration tests under `tests/integration/<context>/` and E2E tests under `tests/e2e/<context>/`.
- `repo/path-naming` checks file and directory names, not code symbols.
- Frontend feature system rules use the `frontend/feature-system-*` namespace.
- Feature system defaults target `packages/frontend/src/systems/<domain>` with shared shadcn/ui primitives allowed under `packages/frontend/src/components/ui`.
- Surface CSS is optional; when present, it lives at the system root and is imported through the system public barrel.
- Assisted reviews cover conventions that depend on intent, ownership, domain meaning, UX expectations, or sufficiency.

## Rule Breakdown

- `repo/test-placement`
- `repo/path-naming`
- `frontend/feature-system-layout`
- `frontend/feature-system-public-api`
- `frontend/feature-system-dependency-flow`
- `frontend/feature-system-adapter-contract`
- `frontend/feature-system-query-contract`
