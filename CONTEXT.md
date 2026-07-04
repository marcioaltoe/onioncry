# OnionCry

OnionCry names and checks architectural boundaries in source projects so architecture drift can be discussed with precise language.

## Language

**Architectural Boundary**:
A named separation in a source project that defines which parts of the code may depend on which other parts. Boundaries are project-specific and may represent layers, contexts, modules, or similar design constraints.
_Avoid_: hardcoded onion rule, fixed architecture

**Architectural Layer**:
A kind of architectural boundary that controls dependency direction between technical responsibilities. A source file belongs to zero or one primary layer for a given ruleset.
_Avoid_: tier, ring

**Architectural Context**:
A kind of architectural boundary that represents a bounded context or capability boundary. A context protects a model, language, and ownership area from depending on another context's internal details.
_Avoid_: feature folder, module when meaning context, package when meaning context

**Architecture Preset**:
A starter vocabulary of architectural boundaries and rules that reflects a common architectural style without making that style mandatory.
_Avoid_: built-in architecture, mandatory onion model

**Project Architecture Mode**:
The configured architecture style that selects exactly one architecture-specific rule family for a project. It is configured with `architecture.mode`; if a configuration file does not select a mode, OnionCry uses the Clean Architecture mode.
_Avoid_: mixed architecture validation, simultaneous clean and vertical slice mode

**Architecture Mode Options**:
Configuration nested under the selected architecture mode, such as `architecture.cleanArchitecture`, that describes structural conventions for that mode. Rule severities remain under `rules` rather than inside mode options.
_Avoid_: rule severity in architecture shape, scattering mode options under unrelated config sections

**Architecture-Specific Rule Family**:
A set of rules that only makes sense for one Project Architecture Mode, such as `cleanarch/*` or `verticalslice/*`. OnionCry activates only the architecture-specific rule family selected by the project mode; architecture-neutral rules remain independent.
_Avoid_: always-on architecture rule, running every architecture pattern at once

**Architecture Rule Mode Mismatch**:
A configuration error raised when a project enables a rule from an Architecture-Specific Rule Family that does not match `architecture.mode`. OnionCry fails configuration validation instead of silently ignoring the incompatible rule.
_Avoid_: silently skipped architecture rule, mixed-mode warning only

**Clean Architecture Preset**:
The default architecture preset and Project Architecture Mode that uses pragmatic layer names: domain, application, infra, and shared. It follows Clean Architecture dependency direction while grouping interface adapters and framework details under infra.
_Avoid_: Uncle Bob names only, splitting adapters/frameworks by default

**Context-First Clean Architecture Layout**:
A Clean Architecture code organization where contextual code lives under the configured context root segment, defaulting to `contexts/<context>/`, and each context contains its own `domain/`, `application/`, and `infra/` layers. Code that does not belong to an Architectural Context uses the same layer shape directly under the source root.
_Avoid_: layer-first clean architecture layout, global use-case dump

**Context Root Segment**:
The configured source path segment that contains Architectural Context folders. OnionCry defaults this to `contexts`, while projects may configure aliases such as `modules`.
_Avoid_: hardcoded modules folder, treating context root as a context

**Layer Path Alias**:
A configured directory-name alias for a canonical Architectural Layer. For example, OnionCry's canonical outer layer is `infra`, but a project may map it to a path segment such as `infrastructure`.
_Avoid_: new layer when meaning path spelling, renaming the architecture vocabulary

**Artifact Folder Map**:
A Project Architecture Mode option that lists the expected artifact folder names for each canonical layer. It lets OnionCry check placement of use cases, ports, repositories, adapters, entities, and value objects without hardcoding every team's directory spelling.
_Avoid_: fixed folder list, unconfigurable artifact taxonomy

**Artifact Filename Suffix**:
A configured filename suffix that helps OnionCry infer an artifact's role, such as `.repository.ts`, `.service.ts`, `.use-case.ts`, `.entity.ts`, `.value-object.ts`, `.adapter.ts`, or `.handler.ts`. Suffix inference complements folder-based classification and should be configurable because teams use different naming conventions.
_Avoid_: class-name inspection as path rule, hardcoded TypeScript suffixes

**Service Artifact**:
A file whose configured suffix marks it as a service, commonly `.service.ts`. In Clean Architecture, a service's architectural role comes from its containing layer or artifact folder; in Vertical Slice, a service is an internal slice detail unless exposed through the slice public surface.
_Avoid_: global service layer, service as default place for all logic

**Global Slice Artifact**:
A file that appears to belong to a Vertical Slice, usually by configured filename suffix or artifact role, but lives outside the configured slice root. In Vertical Slice mode, OnionCry may warn about these files without treating configured global bootstrap, shared, config, library, or platform infrastructure folders as automatic violations.
_Avoid_: every global folder is invalid, clean architecture fallback check

**Artifact Placement Rule**:
A Clean Architecture code organization rule, named `cleanarch/artifact-placement`, that reports artifacts placed outside the configured Context-First Clean Architecture Layout. It defaults to a warning so existing projects can expose structure drift before making it a blocking gate.
_Avoid_: migration blocker by default, folder nitpick rule

**Presence-Based Structure Rule**:
A code organization rule that validates where an artifact belongs when that artifact exists, without requiring every possible folder to exist up front. It prevents misplaced use cases, ports, repositories, adapters, entities, and value objects while allowing small contexts to omit unused layers.
_Avoid_: empty folder requirement, scaffold completeness rule

**Vertical Slice Preset**:
An architecture preset and Project Architecture Mode that organizes code around complete slices of user or business capability instead of global technical layers. It favors slice-local cohesion and cross-slice encapsulation over Clean Architecture layer validation.
_Avoid_: feature folder when meaning context, clean architecture with different folders

**Vertical Slice Layout**:
A code organization where each complete slice of user or business capability lives under the configured slice root segment, defaulting to `features/<domain>/<operation>/`. The slice root may be configured to alternatives such as `slices`, `modules`, or `.`, and `sliceDepth: 1` supports projects that intentionally use `features/<feature>` or root-level feature folders.
_Avoid_: layer-first vertical slice, hardcoded features folder

**Slice Public Surface**:
The explicit files or folders other slices may import from, defaulting to a slice root `index.ts` and `contracts/`. Other slice files are internal details unless the project configures them as public.
_Avoid_: any exported file is public, importable slice internals

**Slice Handler**:
The entrypoint inside a Vertical Slice that handles a command, query, endpoint, or equivalent user/business request. Handlers coordinate slice-local logic and adapters without becoming shared services for unrelated slices.
_Avoid_: global use case when meaning slice handler, controller-only slice

**Slice Root Segment**:
The configured source path segment that contains Vertical Slice folders. OnionCry defaults this to `features`; using `.` means the source root itself contains slice folders and should be an explicit project choice because it is more ambiguous.
_Avoid_: implicit root-level feature detection, treating every source folder as a slice

**Default Rule Preset**:
The starter rule policy generated by the configuration template. It treats layer leaks and cross-context internal imports as errors, external packages as default-deny in sensitive layers, the selected Project Architecture Mode's architecture-specific checks as warnings, shotgun-surgery history analysis as off, and generic import resolution or file-cycle checks as delegated to the JavaScript linter.
_Avoid_: silent starter config

**Domain Layer**:
The layer for entities and core business rules. It should not depend on application workflows, adapters, frameworks, or external details.
_Avoid_: model when meaning database model, entity when meaning ORM record

**Application Layer**:
The layer for use cases and application-specific business rules. It orchestrates domain behavior through ports and should not depend on adapters, frameworks, or external details.
_Avoid_: service when meaning any class, use case folder when meaning layer

**Infrastructure Layer**:
The outer layer for interface adapters, framework code, drivers, SDKs, runtime bootstrapping, queues, database clients, repository implementations, controllers, presenters, and other external details.
_Avoid_: adapter when meaning the whole outer layer, framework when meaning the whole outer layer

**Public Surface**:
The small, intentional set of contracts that other architectural contexts may depend on. A public surface exposes what a context promises to other contexts, not how the context is implemented.
_Avoid_: exports when meaning all exported files, public folder when meaning contract

**Public Surface Segment**:
A configured path segment, such as `contracts`, `events`, `ports`, or `shared`, that marks part of a context as importable by other contexts in the MVP.
_Avoid_: arbitrary public glob in the MVP

**Internal Detail**:
Any part of an architectural context that is not part of its public surface. Other contexts should not depend on internal details because they are owned and changed inside the context.
_Avoid_: private code when meaning language visibility only

**Shared Kernel**:
A small, stable set of concepts intentionally shared across contexts. In the default preset, global shared code is classified as the shared layer and is not modeled as its own context.
_Avoid_: utils, common dump, shared everything

**Cross-Context Import**:
An import where the importing file and imported file belong to different architectural contexts. A cross-context import is acceptable only when it targets the imported context's public surface or an approved shared kernel.
_Avoid_: module import when the important point is context ownership

**Boundary Classification**:
The assignment of a source file to a configured architectural boundary. A classification must be unambiguous for OnionCry to evaluate rules reliably.
_Avoid_: first matching rule, best guess

**Unclassified File**:
An analyzed project file that belongs to the file universe but does not match any configured architectural layer. Unclassified files are reported by default because they may hide gaps in architecture coverage.
_Avoid_: ignored file

**Contextless File**:
An analyzed project file that does not match any configured architectural context. Contextless files are allowed in the MVP because not every file belongs to a bounded context.
_Avoid_: unclassified context as a default violation

**Allowed Import**:
A dependency from one architectural layer to another layer listed in the importing layer's `mayImport` rule. OnionCry evaluates allowed imports from explicit rules rather than from an implied layer order.
_Avoid_: lower layer, higher layer when the configured rule is what matters

**Domain-Safe External Package**:
An external package explicitly allowed in the domain layer because it supports modeling a domain concept without introducing framework, persistence, network, runtime, or infrastructure concerns.
_Avoid_: excluded dependency, ignored package

**External Package Allowlist**:
A rule option that defines which external packages may be imported from a sensitive layer. Packages not on the allowlist are treated as closed by default for that layer.
_Avoid_: package exclude, implicit package permission

**External Package Import**:
An import whose specifier resolves to a package outside the analyzed project rather than to a project file. External package imports are governed by package policy, not by layer or context boundary checks.
_Avoid_: unresolved local import, alias import

**Normalized Package Name**:
The package name extracted from an external import specifier before matching package policy. Unscoped imports use the first path segment, and scoped imports use the first two path segments.
_Avoid_: full import specifier when matching package allowlists by default

**Runtime Built-in Import**:
An import provided by the runtime, such as `node:fs`, `node:path`, or `node:crypto`. Runtime built-ins are treated like external package imports for sensitive-layer package policy.
_Avoid_: automatically safe built-in

**Layer-Specific Severity**:
A rule severity chosen according to the importing layer. OnionCry may treat the same kind of dependency as an error in a sensitive layer, a warning in an orchestration layer, and off in an outer detail layer.
_Avoid_: one-size-fits-all severity

**Rule**:
A named architecture check that can be turned off or reported as a warning or error. Rule names use intent-specific namespaces, such as `cleanarch/...`, `codesmells/...`, and future families such as `solid/...`.
_Avoid_: numbered rule code as the primary name

**Rule Catalog Listing**:
The rules introspection output produced by `onioncry rules`, generated from the built-in rule catalog so canonical names, default severities, architecture families, legacy aliases, and explanations cannot drift from the implementation.
_Avoid_: hand-maintained rule table, rule documentation divorced from the catalog

**Code Organization Rule**:
A rule that checks observable repository structure, such as file naming, test placement, required folders, or required barrel files. It verifies code state, not whether a contributor followed a process or activated an agent skill.
_Avoid_: plugin when meaning configured convention, skill enforcement, process audit

**Rule Option Defaults**:
The implicit rule options OnionCry applies when a rule is configured with only a severity string. Defaults make a concise config meaningful while still allowing a project to override the rule with an explicit `[severity, options]` value.
_Avoid_: default rule preset when meaning one rule's fallback options

**Project-Focused Defaults**:
Opinionated rule option defaults chosen for the maintainers' common project shape while remaining overridable by explicit rule options. OnionCry can be open source while still prioritizing the conventions its maintainers use first.
_Avoid_: universal default, hardcoded project exception

**Forward-Looking Default**:
A rule option default that expresses the desired convention even when existing repositories need temporary overrides during migration. Legacy code should use explicit overrides rather than weakening the default contract.
_Avoid_: legacy-shaped default, silent grandfathering

**Test Placement Rule**:
A code organization rule that distinguishes source-level unit tests from workspace-level integration and end-to-end tests. By default, unit test files belong in co-located `__tests__` folders, while integration and end-to-end tests may live under dedicated test roots.
_Avoid_: requiring every test file under source, mixing unit and integration placement

**Path Naming Rule**:
A code organization rule that checks file and directory names, including casing, suffixes, and collection-versus-feature directory naming. It does not check class names, function names, variable names, constants, or other code symbols.
_Avoid_: symbol naming rule, generic lint naming rule

**Feature System Contract**:
A frontend code organization contract for domain-owned `systems/<domain>` modules. It defines expected folders, public barrel boundaries, dependency flow, API adapter shape, query ownership, and component responsibilities that can be checked when they are observable in files and imports.
_Avoid_: agent skill requirement, documentation-only checklist as automatic rule

**Surface CSS**:
An optional CSS file owned by one feature system and placed at the system root when Tailwind CSS or shared UI primitives are not enough for that surface. If present, it follows the system domain name and is imported through the system public barrel.
_Avoid_: required per-system stylesheet, global feature CSS

**Feature System Public API**:
The explicit named exports in a feature system's root `index.ts` barrel. Other systems and routes depend on this public API rather than importing internal files, and wildcard re-exports are rejected by default because they blur the system contract.
_Avoid_: export everything barrel, cross-system internal import

**Feature System Dependency Flow**:
The allowed import direction inside a feature system. Adapters stay below hooks, contexts, stores, components, and routes; query options bridge adapters into the query layer; routes and other systems depend on the system public API instead of internals.
_Avoid_: component-to-adapter shortcut, route-owned system internals

**Feature System Adapter Contract**:
The observable shape of a feature system API adapter: a domain-named adapter file, a namespace object for API operations, a typed API error export, cancellation-aware operations where applicable, and no imports from upper frontend layers. Semantic response normalization remains an assisted review unless expressed as a machine-checkable contract.
_Avoid_: raw HTTP calls in components, adapter importing UI

**Feature System Query Contract**:
The observable TanStack Query ownership rules inside a feature system: query keys and query options live under `lib`, query hooks reuse option factories, query functions pass cancellation signals to adapters, routes and components do not own duplicate query keys, and mutations invalidate or roll back cache changes explicitly. Cache scope sufficiency remains an assisted review unless the project encodes it as rule options.
_Avoid_: scattered query keys, inline query behavior in components

**Automatic Rule Check**:
A rule evaluation that can report a warning or error from observable project facts such as file paths, imports, exports, and recognizable syntax. Automatic checks should not depend on guessing contributor intent or product ownership.
_Avoid_: human review disguised as a hard rule

**Assisted Contract Review**:
A non-blocking review item for a convention that depends on intent, ownership, domain meaning, UX expectations, or sufficiency. Assisted reviews can explain likely gaps, but they should not fail a check without an explicit machine-checkable contract.
_Avoid_: flaky rule, subjective error

**Violation Baseline**:
A committed `.onioncry-baseline.json` file that records existing violations so a project can adopt stricter rules gradually. Baselined debt remains visible, while only new violations affect the Check Status.
_Avoid_: ignore list, disabled rules, hidden grandfathering

**Baseline Fingerprint**:
The stable identity of a violation baseline entry: `rule + file + target`. The target is the violating import specifier for import diagnostics or a rule-specific subject for file-level diagnostics; line and column are excluded.
_Avoid_: location-only fingerprint, message hash, line-based baseline

**Baselined Violation**:
A current violation that matches a Violation Baseline entry within that entry's count. It is reported separately from active violations and does not affect the failure threshold.
_Avoid_: ignored violation, fixed violation, suppressed warning

**Inline Suppression**:
A source comment in the form `// onioncry-disable-next-line <rule>[, <rule>] -- <reason>` that marks matching violations on the next line as accepted exceptions. Inline suppressions are visible in reports and do not affect the failure threshold.
_Avoid_: hidden ignore, file-level disable, undocumented exception

**Suppression Reason**:
The mandatory explanation after `--` in an Inline Suppression. It records why the exception exists so reviewers can decide whether the source-level exception is still justified.
_Avoid_: empty reason, TODO-only reason, silent waiver

**Stale Baseline Entry**:
A Violation Baseline entry that matches no current violation. It indicates debt that may have been fixed and should be removed by rerunning `--write-baseline`, but it does not fail the run.
_Avoid_: baseline error, missing violation, failed ratchet

**Violation**:
A reported rule finding with a linter-style rule name, severity, message, source location, optional suggestion, and rule-specific context. Violations use the same canonical rule names as configuration and include import line and column when available.
_Avoid_: numeric rule id as the primary identifier

**Check Status**:
The pass/fail result of an OnionCry run after applying the configured or CLI-selected failure threshold. Status reflects whether the run should block automation, while the summary keeps the raw warning and error counts.
_Avoid_: failed whenever warnings exist

**File-Scoped Check**:
A check run whose report is filtered to explicitly listed files while the analysis stays whole-project, so scoped results never disagree with a full run. Project-level findings without a single file location, such as context cycles, are always reported, and paths outside the file universe are skipped with a warning instead of failing.
_Avoid_: partial analysis when meaning a filtered report, per-file rule evaluation without the import graph

**File Explanation**:
An interactive report for one source file that shows boundary classification, matched patterns, resolved imports, and violations for that file. A file explanation is diagnostic and does not act as a CI gate.
_Avoid_: whole architecture report

**Configuration Template**:
A commented starter `.onioncryrc.jsonc` generated by `onioncry init`. The template is conservative and expects the project team to review aliases, boundaries, allowlists, and rules instead of trusting aggressive auto-detection.
_Avoid_: inferred architecture as truth

**Override**:
A file-pattern-specific rule configuration that changes how OnionCry evaluates selected files. Overrides change rule severity or rule options; they do not change file selection, aliases, layers, or contexts.
_Avoid_: ignore list when the file is still analyzed

**Effective Rule Configuration**:
The rule value OnionCry applies to a file after matching overrides. When multiple overrides define the same rule for a file, OnionCry applies them in array order and the last matching override wins.
_Avoid_: merged rule options

**File Universe**:
The set of files selected by project include and exclude patterns before rules are evaluated. Overrides do not add files to the file universe; they only change policy for files already selected.
_Avoid_: override as include

**Configuration File**:
The JSONC file that defines project paths, aliases, boundaries, rules, and overrides for OnionCry. Auto-discovery checks `.onioncryrc.jsonc` first and `.onioncryrc.json` second.
_Avoid_: YAML config as the default

**Configuration Schema**:
The JSON Schema for the configuration file, derived from the configuration types and exposed by `onioncry schema`. The generated template references it through `$schema` so editors validate and autocomplete configuration without the schema drifting from the parser.
_Avoid_: hand-written config schema, schema divorced from the config types

**Configuration Field Naming**:
Configuration and JSON output fields use camelCase. This keeps OnionCry aligned with JSON and JavaScript tooling conventions.
_Avoid_: snake_case fields in JSONC examples

**Alias Mapping**:
An explicit mapping from an import prefix to a project path in the OnionCry configuration. The MVP uses configured aliases rather than inferring TypeScript path mappings.
_Avoid_: implicit tsconfig alias

**Tsconfig Alias Generation**:
The explicit `onioncry init --from-tsconfig` step that translates a tsconfig's wildcard path mappings into the configuration's alias mappings for team review. Entries that cannot become prefix aliases are listed for manual mapping, and check-time alias resolution still reads only the OnionCry configuration.
_Avoid_: runtime tsconfig inference, silent alias discovery

**Local Import Resolution**:
The MVP process for mapping relative and aliased imports to project files by trying common TypeScript and JavaScript file extensions and index files. It does not resolve package exports, package main fields, TypeScript project references, or declaration files.
_Avoid_: full Node resolver, full TypeScript resolver

**Unresolved Local Import**:
A relative or configured-alias import that OnionCry cannot map to a project file using MVP local import resolution. OnionCry exposes this in file explanations but delegates unresolved-import lint diagnostics to Oxlint, Biome, or TypeScript.
_Avoid_: missing npm package

**Type-Only Import**:
A TypeScript import used only for type checking. Type-only imports still count as architectural dependencies because they couple the importing file to a named module.
_Avoid_: harmless import type

**Import Edge**:
A dependency edge discovered from a source file. In the MVP, import edges come from static imports, type-only imports, re-exports, string-literal dynamic imports, and string-literal `require` calls.
_Avoid_: only import declarations

**Context Graph**:
A diagnostic ownership graph derived from Import Edges. In Clean Architecture mode it shows dependencies between architectural contexts; in Vertical Slice mode it shows dependencies between slices. Contextless or slice-less files aggregate into one explicit node.
_Avoid_: file graph, package graph, class diagram

**Context Cycle**:
A cycle of import edges between architectural contexts. Generic file-level import cycles belong to the JavaScript linter; OnionCry reports cycles at the ownership-boundary level.
_Avoid_: file cycle when the important point is context ownership

## Example Dialogue

Dev: "Should OnionCry only understand domain, application, infra, and http?"

Domain expert: "No. Those can come from an architecture preset, but the project defines its own architectural boundaries."

Dev: "So a team could use core, adapters, and delivery instead?"

Domain expert: "Yes. OnionCry should check the boundaries the project names."

Dev: "Are layers and contexts the same thing?"

Domain expert: "No. A layer defines dependency direction; a context defines ownership and encapsulation."

Dev: "Can one context import another context's files?"

Domain expert: "Only through that context's public surface. Everything else is an internal detail."

Dev: "What makes a cross-context import acceptable?"

Domain expert: "It must target a contract exposed by the imported context, not an internal detail owned by that context."

Dev: "How does the MVP recognize a context's public surface?"

Domain expert: "By configured path segments such as contracts, events, ports, and shared."

Dev: "Is shared a context?"

Domain expert: "No. In the default preset, shared is a layer or public surface segment, not a bounded context."

Dev: "What if a file matches two layers?"

Domain expert: "That is an ambiguous boundary classification, not a precedence problem."

Dev: "What if a scanned file matches no layer?"

Domain expert: "That is an unclassified file. The default preset reports it as a warning so coverage gaps are visible."

Dev: "Can a project run Clean Architecture and Vertical Slice validation at the same time?"

Domain expert: "No. The project selects one Project Architecture Mode. If it does not select one, OnionCry uses Clean Architecture by default."

Dev: "What if a Vertical Slice project configures a `cleanarch/*` rule?"

Domain expert: "That is an Architecture Rule Mode Mismatch. OnionCry fails configuration validation instead of ignoring the rule."

Dev: "Where should Vertical Slice code live by default?"

Domain expert: "Use the Vertical Slice Layout: `src/features/<domain>/<operation>` by default. Keep `sliceRoot` configurable to alternatives such as `slices`, `modules`, or `.`, and use `sliceDepth: 1` when a project intentionally uses `features/<feature>`."

Dev: "What folders should a default Vertical Slice contain?"

Domain expert: "Use `index.ts` and `contracts/` as the public surface, with optional internal `handlers/`, `adapters/`, `domain/`, and `__tests__/` folders. Do not require empty folders. A slice should expose a configured entry point such as `setup`, `Map`, or `register` when that rule is enabled."

Dev: "Can OnionCry use filename conventions such as `.repository.ts` and `.service.ts`?"

Domain expert: "Yes. Artifact Filename Suffixes complement folder placement so rules can identify artifact roles during migrations or in flatter layouts."

Dev: "Does `.service.ts` mean the same thing in Clean Architecture and Vertical Slice?"

Domain expert: "No. In Clean Architecture, folder placement decides whether it is a domain, application, or infra service. In Vertical Slice, it is a slice-internal detail unless exposed through the slice public surface."

Dev: "Can a Vertical Slice project keep global `repositories`, `services`, `handlers`, or `use-cases` folders?"

Domain expert: "Only as explicit migration debt or project-specific exceptions. The default Vertical Slice rule treats those shared technical layers as drift because implementation details should live inside the owning slice."

Dev: "If a project selects Vertical Slice, are global `domain`, `application`, or `infra` folders automatically invalid?"

Domain expert: "No. Vertical Slice mode does not run Clean Architecture checks. It may warn about Global Slice Artifacts outside the slice root while still allowing configured global folders for bootstrap, shared code, config, libraries, and platform infrastructure."

Dev: "Where do layout options such as context root, layer path aliases, and artifact folders live?"

Domain expert: "Under the selected mode's Architecture Mode Options, such as `architecture.cleanArchitecture`. Rule severities stay under `rules`."

Dev: "How should a large Clean Architecture backend avoid long global use-case and repository folders?"

Domain expert: "Use the Context-First Clean Architecture Layout: `src/contexts/<context>/{domain,application,infra}` for contextual code and `src/{domain,application,infra}` for contextless base code."

Dev: "Must every context contain every Clean Architecture layer folder from the start?"

Domain expert: "No. Use Presence-Based Structure Rules: validate artifacts when they exist, but do not require empty layer folders."

Dev: "Should artifact placement violations block existing projects by default?"

Domain expert: "No. `cleanarch/artifact-placement` defaults to warning so teams can migrate gradually, then raise it to error when the layout is clean."

Dev: "What if a scanned file matches no context?"

Domain expert: "That is allowed in the MVP. Context rules apply when imports cross between classified contexts."

Dev: "What layer names should the default preset use?"

Domain expert: "Use domain, application, infra, and shared. Infra groups adapters and framework details because most repositories treat them as one outer layer."

Dev: "What should the starter rules enforce?"

Domain expert: "Fail layer leaks and cross-context internal imports, fail closed for domain external packages, warn in application, and start adoption-sensitive checks as warnings."

Dev: "Does OnionCry infer layer direction from the order in the config?"

Domain expert: "No. A layer import is allowed only when the target layer appears in the source layer's may_import rule."

Dev: "Can a Value Object in the domain use a UUID package?"

Domain expert: "Yes, if the package is explicitly allowlisted as domain-safe: it supports the model without pulling in framework, persistence, network, runtime, or infrastructure concerns."

Dev: "Does the external package allowlist apply to local aliases?"

Domain expert: "No. Local imports are checked by layer and context rules; the allowlist governs external package imports."

Dev: "Does allowing @scope/pkg also allow @scope/pkg/subpath?"

Domain expert: "Yes. Package policy matches the normalized package name unless a rule explicitly supports finer subpath matching."

Dev: "Are Node built-ins automatically allowed in the domain layer?"

Domain expert: "No. Runtime built-ins are external dependencies for policy purposes and must be explicitly allowed in sensitive layers."

Dev: "Should external package policy have the same severity in every layer?"

Domain expert: "No. Sensitive layers can fail closed while outer layers stay open for infrastructure work. When a rule supports layer-specific severity, that severity wins for the matching layer."

Dev: "Should rule names be numbered?"

Domain expert: "No. Use linter-style names such as cleanarch/no-layer-leak and cleanarch/no-context-cycle, and control severity through rules and overrides."

Dev: "What rule identifier appears in JSON output?"

Domain expert: "The same linter-style rule name used in configuration."

Dev: "Should a violation point to the import line?"

Domain expert: "Yes, when parser spans are available. Otherwise it still reports the file and import string."

Dev: "Does a warning make JSON status failed?"

Domain expert: "Only when the effective failure threshold is warning. Otherwise warnings are counted but status can still pass."

Dev: "What is explain for?"

Domain expert: "It explains one file's classification, imports, and violations so a developer can understand a result locally."

Dev: "Does init discover our architecture automatically?"

Domain expert: "No. It creates a conservative configuration template that the team must adapt."

Dev: "Why is the default config JSONC instead of YAML?"

Domain expert: "Because OnionCry behaves like a linter: JSONC fits rules, overrides, comments, and schema validation."

Dev: "What config file should a project commit?"

Domain expert: "Prefer `.onioncryrc.jsonc`; OnionCry also auto-discovers `.onioncryrc.json` after JSONC for projects that keep strict JSON config files."

Dev: "Should JSONC fields use may_import or mayImport?"

Domain expert: "Use camelCase fields such as mayImport, fromLayer, and allowSameContext."

Dev: "Does OnionCry read tsconfig path aliases in the MVP?"

Domain expert: "No. The MVP resolves aliases declared in the OnionCry configuration."

Dev: "Does OnionCry report unresolved imports?"

Domain expert: "No. OnionCry still resolves local imports for architecture checks and explain output, but unresolved-import diagnostics belong to Oxlint, Biome, or TypeScript."

Dev: "Does OnionCry report generic file-level circular dependencies?"

Domain expert: "No. Generic import cycles belong to Oxlint or Biome. OnionCry reports architectural context cycles such as cleanarch/no-context-cycle."

Dev: "Does import type bypass architecture rules?"

Domain expert: "No. Type-only imports still create architectural coupling and are checked by layer and context rules."

Dev: "Do barrels and re-exports count?"

Domain expert: "Yes. Re-exports are import edges because they can leak dependencies across architectural boundaries."

Dev: "Are cycles only a problem when they cross layers?"

Domain expert: "No. Any cycle between analyzed local files is coupling worth reporting, though the default preset starts with a warning."

Dev: "Can overrides bring excluded files back into analysis?"

Domain expert: "No. Include and exclude define the file universe first; overrides only change rule policy inside that universe."

Dev: "Can an override make a legacy folder count as a different layer?"

Domain expert: "No. Boundary classification comes from layer and context patterns, not overrides."

Dev: "Do override rule options merge with the base rule options?"

Domain expert: "No. A matching override replaces the whole rule value for that file; if several overrides match, the last one wins."

Dev: "Can OnionCry check whether a frontend change used the required agent skill?"

Domain expert: "No. A code organization rule can verify resulting files, names, folders, imports, and barrels, but it does not audit contributor workflow."

Dev: "If `repo/test-placement` is configured as just `error`, where do test files belong?"

Domain expert: "The rule option defaults apply. For that rule, co-located unit tests use `__tests__` folders unless the project provides explicit options."

Dev: "Should frontend organization rules use `system` or `feature-system` in their names?"

Domain expert: "Use `feature-system`, such as `frontend/feature-system-layout`, because `system` alone is too broad for OnionCry's architecture language."

Dev: "Should path naming defaults use `infra` or `infrastructure` for the outer layer directory?"

Domain expert: "Use `infra` by default. Projects that spell out `infrastructure` can configure that explicitly."
