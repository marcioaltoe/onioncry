# Add Clean Architecture artifact placement

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add `cleanarch/artifact-placement` as a presence-based code organization rule for Clean Architecture mode. The rule reports use cases, ports, repositories, adapters, entities, value objects, and related configured artifact suffixes when they are outside the configured context-first Clean Architecture layout.

## Acceptance criteria

- [ ] `cleanarch/artifact-placement` is accepted in `rules` with `off`, `warn`, `error`, and `[severity, options]`.
- [ ] The default severity is `warn` in the generated starter configuration.
- [ ] Contextual artifacts are valid under `contexts/<context>/{domain,application,infra}` according to configured artifact folders.
- [ ] Contextless base artifacts are valid under root `domain/`, `application/`, and `infra/` according to configured artifact folders.
- [ ] `contextRoot` can be configured to an alternative such as `modules`.
- [ ] `layerPathAliases` can map canonical layers to alternate path segments such as `infrastructure`.
- [ ] `artifactSuffixes` classify files such as `.repository.ts`, `.service.ts`, `.use-case.ts`, `.entity.ts`, `.value-object.ts`, `.adapter.ts`, and `.handler.ts`.
- [ ] `.service.ts` classification uses the containing layer or artifact folder rather than a global service layer.
- [ ] The rule does not require empty folders in every context.
- [ ] Violations include the detected artifact role, current path, expected location pattern, and rule name.
- [ ] Tests cover valid context artifacts, valid contextless artifacts, misplaced artifacts, configured context root, configured layer aliases, suffix classification, ambiguous service placement, disabled rule, and overrides.
- [ ] `make verify` passes.

## Blocked by

- 01-add-architecture-mode-config.md
- 02-reject-architecture-rule-mode-mismatch.md
