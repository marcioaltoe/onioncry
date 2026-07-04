# Architecture Modes

Status: ready-for-agent

## Problem Statement

OnionCry currently treats Clean Architecture as the implied architecture style for architecture-specific rules. Projects that prefer Vertical Slice organization need a way to select that style without also running Clean Architecture validation.

Backend projects such as Fluxus also expose a structure problem that OnionCry does not yet model well: use cases, ports, repositories, adapters, entities, and value objects can drift into long global folders. Teams need rules that encourage context-first Clean Architecture, while other teams need feature-local Vertical Slice rules. Running both rule families at once would create noisy, contradictory diagnostics.

## Solution

Add a Project Architecture Mode to the configuration. `cleanArchitecture` remains the default when no mode is configured, and `verticalSlice` becomes an explicit alternative. Architecture-specific rule families are mutually exclusive: Clean Architecture projects can use `cleanarch/*`, Vertical Slice projects can use `verticalslice/*`, and a rule from the wrong family fails configuration validation.

Clean Architecture mode gets context-first organization rules. Contextual code belongs under a configured context root, with each Architectural Context containing its own domain, application, and infra layers. Contextless base code can still use root domain, application, and infra folders. The placement rules are presence-based, so OnionCry validates artifacts when they exist but does not require empty folders.

Vertical Slice mode gets slice-first organization rules. Slices live under a configured slice root, defaulting to `features`. Each slice exposes a configured public surface and keeps handlers, adapters, domain helpers, services, and tests internal by default. Vertical Slice mode does not run Clean Architecture checks, but it can warn when files that look like slice artifacts drift into global folders.

## User Stories

1. As a project maintainer, I want to configure one Project Architecture Mode, so that OnionCry validates the architecture my project actually uses.
2. As a project maintainer, I want Clean Architecture to remain the default, so that existing configs keep working.
3. As a project maintainer, I want `verticalSlice` to be explicit, so that adopting it is a deliberate project choice.
4. As a tool user, I want a clear error when I configure a rule from the wrong architecture family, so that copy-paste config drift does not silently hide checks.
5. As a tool user, I want architecture-neutral rules to work in any mode, so that repository conventions still apply regardless of architecture style.
6. As a backend architect, I want Clean Architecture rules to prefer context-first layout, so that large projects do not accumulate long global folders by artifact type.
7. As a backend architect, I want each Architectural Context to own its domain, application, and infra layers, so that context ownership is visible in the source tree.
8. As a backend architect, I want contextless base code to remain valid, so that shared backend primitives do not need fake contexts.
9. As a backend architect, I want context root naming to be configurable, so that projects using `modules` can still follow the rule.
10. As a backend architect, I want layer path aliases to be configurable, so that projects using `infrastructure` can map it to OnionCry's canonical infra layer.
11. As a backend architect, I want artifact folders to be configurable by canonical layer, so that the rule can match each team's vocabulary.
12. As a backend architect, I want artifact filename suffixes to help classification, so that migrated or flatter code can still be diagnosed.
13. As a Fluxus backend maintainer, I want `application/use-cases` files to be reported when they belong to a context, so that use cases can move toward context-owned folders.
14. As a Fluxus backend maintainer, I want `application/ports` files to be reported when they belong to a context, so that port contracts do not become one global list.
15. As a Fluxus backend maintainer, I want `infra/repositories` files to be reported when they belong to a context, so that repository implementations stay near the context they serve.
16. As a Fluxus backend maintainer, I want `infra/adapters` files to be reported when they belong to a context, so that external adapters do not become one global list.
17. As a domain model maintainer, I want entities and value objects to be checked against domain folders, so that core model artifacts stay in the domain layer.
18. As a migration owner, I want Clean Architecture artifact placement to default to warning, so that existing projects can expose drift before making it a blocking gate.
19. As a migration owner, I want overrides to lower or disable placement checks for legacy areas, so that migration can be staged.
20. As a developer, I want OnionCry to avoid requiring empty folders, so that small contexts can stay small.
21. As a developer, I want `.service.ts` to be interpreted by its containing Clean Architecture layer, so that service files do not create a fake global service layer.
22. As a developer, I want `.repository.ts`, `.adapter.ts`, `.entity.ts`, `.value-object.ts`, `.use-case.ts`, and `.handler.ts` suffixes to be configurable classifiers, so that project naming conventions can drive diagnostics.
23. As a Vertical Slice adopter, I want slices to live under `features` by default, so that source root stays available for app, config, shared, and library code.
24. As a Vertical Slice adopter, I want the slice root to be configurable, so that projects can use `features`, `slices`, `modules`, or root-level slices intentionally.
25. As a Vertical Slice adopter, I want `index.ts` and `contracts` to be the default public surface, so that other slices have an explicit import boundary.
26. As a Vertical Slice adopter, I want handlers, adapters, domain helpers, services, and tests to be internal by default, so that slice implementation details stay private.
27. As a Vertical Slice adopter, I want imports between slices to target only the public surface, so that slices remain independent.
28. As a Vertical Slice adopter, I want type-only imports and re-exports to count as slice dependencies, so that type coupling cannot bypass architecture checks.
29. As a Vertical Slice adopter, I want configured global platform folders to be allowed, so that bootstrap, config, shared libraries, and platform infrastructure can exist outside slices.
30. As a Vertical Slice adopter, I want warnings for slice-looking files outside the slice root, so that projects do not drift back into global feature lists.
31. As a CLI user, I want `onioncry init` to show the architecture mode contract, so that new projects start from an explicit configuration.
32. As a CLI user, I want diagnostics to name the rule, architecture mode, detected artifact role, and expected boundary, so that violations are actionable.
33. As a CI maintainer, I want rule severities to remain under `rules`, so that mode shape and policy strictness stay separate.
34. As an agent implementing this work, I want the PRD and issues to name dependencies and acceptance criteria, so that each slice can be implemented without re-opening the design discussion.

## Implementation Decisions

- Add `architecture.mode` to the configuration contract.
- Support `cleanArchitecture` and `verticalSlice` as the allowed mode values.
- Treat missing `architecture.mode` as `cleanArchitecture`.
- Keep mode-specific structural options under the selected mode object.
- Keep rule severities and rule option values under `rules`.
- Treat any enabled architecture-specific rule from a non-selected mode as a configuration error.
- Apply the same architecture rule mode mismatch validation to top-level rules and override rules.
- Keep architecture-neutral rule families independent from Project Architecture Mode.
- Add Clean Architecture mode options for context root, layer path aliases, artifact folder maps, and artifact filename suffixes.
- Use `contexts` as the default Clean Architecture context root segment.
- Use canonical layer names `domain`, `application`, and `infra`.
- Allow path aliases such as `infrastructure` for the canonical infra layer.
- Validate Clean Architecture artifact placement as a presence-based rule.
- Add `cleanarch/artifact-placement` with default severity `warn`.
- Classify Clean Architecture artifacts by configured folder maps and configured filename suffixes.
- Interpret `.service.ts` in Clean Architecture through the containing layer or artifact folder.
- Allow contextless base artifacts in root domain, application, and infra layers.
- Avoid requiring empty layer folders for every Architectural Context.
- Add Vertical Slice mode options for slice root, slice depth, public surface, artifact folders, artifact suffixes, allowed global folders, entry point names, and shared layer folders.
- Use `features/<domain>/<operation>` as the default Vertical Slice layout.
- Allow `slices`, `modules`, `features/<feature>`, and root-level slices through explicit `sliceRoot` and `sliceDepth` configuration.
- Use `index.ts` and `contracts` as the default Vertical Slice public surface.
- Treat handlers, adapters, domain helpers, services, and tests as slice internals by default.
- Interpret `.service.ts` in Vertical Slice as a slice-internal artifact unless exposed through the public surface.
- Add `verticalslice/no-cross-slice-internal-import` to prevent imports from one slice into another slice's internals.
- Add `verticalslice/no-global-slice-artifacts` to report slice-looking files outside the configured slice root.
- Add `verticalslice/slice-entry-point` to report slices without configured public entry points.
- Add `verticalslice/no-shared-layer-artifacts` to report global technical layers such as repositories, services, handlers, and use cases outside slices.
- Allow configured global folders in Vertical Slice mode for bootstrap, config, shared code, libraries, and platform infrastructure.
- Update init output so new projects see the architecture mode contract and the Clean Architecture defaults.
- Keep the CLI output model linter-style: violations include rule name, severity, file location, context needed to understand the finding, and a suggestion when one is known.

## Testing Decisions

- The primary test seam is the CLI behavior using fixture projects and fixture configuration files. This checks configuration parsing, effective defaults, local import graph behavior, pretty diagnostics, JSON diagnostics, and exit status together.
- `onioncry init` output needs snapshot or direct content tests for the new architecture mode block and default rules.
- Configuration tests must cover explicit Clean Architecture mode, explicit Vertical Slice mode, missing mode default, invalid mode, and architecture rule mode mismatch.
- Clean Architecture placement tests must cover valid contextual artifacts, valid contextless base artifacts, misplaced artifacts, configured context root, configured layer aliases, configured artifact suffixes, `.service.ts` ambiguity, disabled rules, and overrides.
- Vertical Slice layout tests must cover default `features/<domain>/<operation>` layout, alternate roots, root-level slices, public surface classification, internal file classification, allowed global folders, entry points, shared-layer drift, and mode isolation from Clean Architecture checks.
- Cross-slice import tests must cover imports through public surface, imports into internals, same-slice imports, type-only imports, re-exports, custom public surface options, and disabled rule behavior.
- Global slice artifact tests must cover misplaced slice-looking files, configured allowed global folders, root-level slice mode, valid slice artifacts, and disabled rule behavior.
- Tests must assert observable behavior instead of private implementation details.
- Existing parser, import graph, rule policy, init template, and CLI integration tests are the prior art for this feature.
- `make verify` is the required completion gate for every implementation issue.

## Out of Scope

- Migrating Fluxus, Vortex, or any other application repository to a new folder layout.
- Auto-moving files or rewriting imports.
- Inferring bounded contexts or slices from Git history, runtime behavior, or semantic ownership.
- Running Clean Architecture and Vertical Slice architecture-specific rules at the same time.
- Adding a plugin runtime for custom architecture patterns.
- Adding new programming-language parsers beyond the existing JavaScript and TypeScript import analysis.
- Enforcing subjective design quality, domain sufficiency, or whether a slice is the right business boundary.
- Changing commit, PR title, or release conventions.

## Further Notes

- ADR 0007 records the one-mode-per-project decision.
- ADR 0008 records the context-first Clean Architecture layout.
- ADR 0009 records `features` as the default Vertical Slice root.
- The existing issue breakdown under this feature remains the implementation queue. It covers configuration parsing, rule-family mismatch validation, Clean Architecture artifact placement, Vertical Slice configuration defaults, cross-slice internal import checks, global slice artifact checks, and init/template documentation.
