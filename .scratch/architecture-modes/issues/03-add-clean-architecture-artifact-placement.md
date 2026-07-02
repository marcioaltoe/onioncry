# Add Clean Architecture artifact placement

Status: ready-for-agent

## Parent

../PRD.md

## User stories covered

- 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 32, 33

## What to build

Add `cleanarch/artifact-placement` as a presence-based code organization rule for Clean Architecture mode. The rule reports use cases, ports, repositories, adapters, entities, value objects, and related configured artifact suffixes when they are outside the configured context-first Clean Architecture layout.

## Acceptance criteria

- [ ] `cleanarch/artifact-placement` is accepted in `rules` with `off`, `warn`, `error`, and `[severity, options]`.
- [ ] The default severity is `warn` in the generated starter configuration.
- [ ] Contextual artifacts are valid under `contexts/<context>/{domain,application,infra}` according to configured artifact folders and suffixes.
- [ ] Contextless base artifacts are valid under root `domain/`, `application/`, and `infra/` according to configured artifact folders and suffixes.
- [ ] Direct capability folders under a contextless base layer, such as `application/reviews`, are reported when they are not configured artifact folders such as `use-cases` or `ports`.
- [ ] A single direct file under a grouped artifact folder is accepted, while two or more direct files in the same grouped artifact folder are reported as a flat list.
- [ ] Direct capability folders with one source file suggest moving the file directly under the artifact folder, such as `domain/entities/classification-export.ts`, instead of suggesting an extra capability subfolder.
- [ ] Direct capability folders under `infra`, such as `infra/reviews`, are reported when they should live under configured artifact folders such as `repositories`, `adapters`, or `bootstrap`.
- [ ] Flat grouped infra artifact folders, such as `infra/repositories`, `infra/adapters`, and `infra/bootstrap`, follow the same one-file exception and multi-file grouping rule as use cases, ports, entities, and value objects.
- [ ] `contextRoot` can be configured to an alternative such as `modules`.
- [ ] `layerPathAliases` can map canonical layers to alternate path segments such as `infrastructure`.
- [ ] `artifactFolders` can map use cases, ports, repositories, adapters, entities, and value objects to the canonical layer where they belong.
- [ ] `artifactSuffixes` classify files such as `.repository.ts`, `.service.ts`, `.use-case.ts`, `.entity.ts`, `.value-object.ts`, `.adapter.ts`, and `.handler.ts`.
- [ ] `.service.ts` classification uses the containing layer or artifact folder rather than a global service layer.
- [ ] The rule does not require empty folders in every context.
- [ ] Pretty and JSON violations include the detected artifact role, current path, expected location pattern, configured architecture mode, and rule name.
- [ ] Tests cover valid context artifacts, valid contextless artifacts, misplaced artifacts, configured context root, configured layer aliases, suffix classification, ambiguous service placement, disabled rule, and overrides.
- [ ] `make verify` passes.

## Blocked by

- 01-add-architecture-mode-config.md
- 02-reject-architecture-rule-mode-mismatch.md
