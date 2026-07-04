# Refactoring Plan: Rule Modules and Configuration

Date: 2026-07-03
Scope: `src/rules/feature_system.rs`, `src/config.rs`, `src/diagnostics.rs`, `src/rules/clean_boundaries.rs`
Method: refactoring-analysis skill, current code inspection, `CONTEXT.md`, ADR 0004, ADR 0006

## Executive Summary

The four target files are doing too much work per module. The main issue is not algorithmic complexity in isolation; it is divergent change pressure. New architecture rules, diagnostic messages, config options, and frontend feature-system checks all tend to grow the same few files.

The preferred refactoring is a no-behavior-change module split that keeps existing public collector functions stable while moving rule-family logic into smaller modules. This fits OnionCry's config-driven linter model and ADR 0006: rules remain built-in, configurable checks, while subjective intent checks stay outside the static rule engine.

## Findings By Priority

### P1 - `src/rules/feature_system.rs` mixes five rule families

Current size: 1573 lines.

Observed responsibilities:

- Layout policy: required directories, legacy roots, component placement, surface CSS.
- Public API policy: entry points, wildcard re-export checks, internal import checks.
- Dependency flow policy: same-system area imports, cross-system public API imports.
- Adapter contract policy: expected adapter files, namespace API export, API error export, abort signal, adapter imports.
- Query contract policy: query keys, query options, hook/component rules, adapter-backed reads.

Smells:

- Large Module
- Divergent Change
- Duplicated Code
- Repeated low-level path classification logic across policies

Recommended structure:

```text
src/rules/feature_system/
  mod.rs
  layout.rs
  public_api.rs
  dependency_flow.rs
  adapter_contract.rs
  query_contract.rs
  location.rs
  helpers.rs
```

Migration plan:

1. Convert `feature_system.rs` into `feature_system/mod.rs` that keeps the existing public collector functions.
2. Move `FeatureSystemLayoutPolicy` to `layout.rs`.
3. Extract shared path/system classification into `location.rs`.
4. Move `FeatureSystemPublicApiPolicy` and `FeatureSystemDependencyFlowPolicy`, then make both use `location.rs`.
5. Move `FeatureSystemAdapterContractPolicy`.
6. Move `FeatureSystemQueryContractPolicy`.
7. Move rendering/export helpers into `helpers.rs`.

Acceptance criteria:

- No rule id, diagnostic message, severity, or file path recommendation changes.
- Existing tests pass unchanged.
- `mod.rs` exposes the same collector functions used by `src/rules/mod.rs`.

### P1 - `src/config.rs` mixes config types, defaults, templates, and option parsing

Current size: 1210 lines.

Observed responsibilities:

- Default constants for Clean Architecture, Vertical Slice, and Feature System rules.
- `INIT_CONFIG_TEMPLATE`.
- Public config data types.
- Rule option parsing helpers.
- Rule severity defaults and canonical rule-name parsing.
- Architecture-mode validation.

Smells:

- Large Module
- Divergent Change
- Mixed abstraction levels
- Primitive/string-heavy rule catalog logic mixed with config model types

Recommended structure:

```text
src/config/
  mod.rs
  types.rs
  defaults.rs
  rule_options.rs
  rule_catalog.rs
  template.rs
```

Notes:

- The user-proposed split should be used as the base: `types.rs`, `defaults.rs`, `rule_options.rs`, `template.rs`.
- Add `rule_catalog.rs` to keep canonical rule names, default severities, and architecture-rule-mode validation out of generic defaults.
- Keep `src/config.rs` as a temporary facade only if needed during migration. The final shape should prefer `src/config/mod.rs`.

Migration plan:

1. Move public data model structs/enums into `types.rs`.
2. Move `INIT_CONFIG_TEMPLATE` into `template.rs`.
3. Move option extraction helpers into `rule_options.rs`.
4. Move default constants and default value builders into `defaults.rs`.
5. Move canonical rule parsing and default severity selection into `rule_catalog.rs`.
6. Re-export stable public types/functions from `mod.rs`.

Acceptance criteria:

- Existing config file compatibility is unchanged.
- JSONC template output is byte-for-byte equivalent unless intentionally updated in a separate docs/config task.
- Default architecture mode remains Clean Architecture unless project config overrides it.

### P1 - `src/diagnostics.rs` centralizes all violation constructors

Current size: 878 lines.

Observed responsibilities:

- Clean Architecture diagnostics.
- Vertical Slice diagnostics.
- repo/test/path diagnostics.
- frontend feature-system diagnostics.
- repeated `Violation { ... }` construction boilerplate.

Smells:

- Large Module
- Duplicated Code
- Divergent Change

Recommended structure:

```text
src/diagnostics/
  mod.rs
  builder.rs
  clean_architecture.rs
  vertical_slice.rs
  repo.rs
  frontend.rs
  code_smells.rs
```

Migration plan:

1. Add a small internal builder/helper for common `Violation` fields.
2. Move constructors by rule family.
3. Keep the public constructor names stable so rule modules do not need behavior changes.
4. Reduce repetition only after the split is passing, to avoid mixing movement with behavior changes.

Acceptance criteria:

- Existing snapshots/assertions for diagnostic messages remain unchanged.
- Each new rule family can add diagnostics without editing an unrelated family file.

### P2 - `src/rules/clean_boundaries.rs` combines layer, package, context, and cycle checks

Current size: 738 lines.

Observed responsibilities:

- Layer dependency violations.
- External package layer ownership.
- Context dependency rules.
- framework-in-core checks.
- outer data format checks.
- public surface re-export checks.
- context cycle detection.
- unowned schema import checks.
- Tarjan strongly connected component algorithm.

Smells:

- Large Module
- Divergent Change
- Long Functions
- Algorithm code mixed with rule orchestration

Recommended structure:

```text
src/rules/clean_boundaries/
  mod.rs
  layer.rs
  package.rs
  context.rs
  core_imports.rs
  public_surface.rs
  cycle.rs
```

Migration plan:

1. Keep collector function names stable in `mod.rs`.
2. Move layer checks to `layer.rs`.
3. Move external package checks to `package.rs`.
4. Move context dependency and unowned schema checks to `context.rs`.
5. Move framework-in-core and outer data format checks to `core_imports.rs`.
6. Move public re-export checks to `public_surface.rs`.
7. Move Tarjan/context-cycle logic to `cycle.rs`.

Acceptance criteria:

- Cycle output ordering remains deterministic.
- Existing Clean Architecture warnings stay unchanged for tested fixtures.
- No new coupling from Clean Architecture rules into Vertical Slice or Feature System rules.

## Shared Refactoring Opportunities

### Shared path classification for Feature System rules

The Feature System policies repeat similar logic for route files, system locations, public entry points, and adapter files. Extracting this into `feature_system/location.rs` should happen before moving the public API and dependency-flow policies, because those policies share the most classification behavior.

Candidate concepts:

- `FeatureSystemLocation`
- `FeatureSystemDependencyArea`
- `FeatureSystemPaths`
- `SystemPathClassifier`

### Small diagnostic builder

Diagnostics repeat the same object shape with only rule id, message, severity, file, expected, found, and help changing. A minimal internal builder can reduce repetition after the family split.

Keep it boring:

```text
ViolationBuilder::new(rule_id, severity, file)
  .message(...)
  .expected(...)
  .found(...)
  .help(...)
  .build()
```

This helper should not hide rule-specific wording. Diagnostic text is public user-facing behavior.

### Config rule catalog

Rule names, default severity, and architecture-mode applicability are domain concepts. Keeping them in `config/rule_catalog.rs` will make later additions to Clean Architecture and Vertical Slice less likely to touch unrelated config types or templates.

## Suggested Execution Order

1. Split `src/config.rs`.
   This has the best safety-to-value ratio and creates clearer places for architecture-mode config defaults before more rule movement.

2. Split `src/rules/clean_boundaries.rs`.
   The collector boundaries are already visible, and the Tarjan algorithm is an obvious isolated module.

3. Split `src/diagnostics.rs`.
   Do this before the Feature System split so moved policies can keep calling stable diagnostic constructors.

4. Split `src/rules/feature_system.rs`.
   This is the largest change. Move one policy at a time, running tests after each move if the branch gets noisy.

## Verification Strategy

Run after each major split:

```bash
rtk cargo fmt --all -- --check
rtk cargo test --all-features
```

Run before delivery:

```bash
rtk make verify
```

Use fixture-level checks for these behavior contracts:

- Clean Architecture default mode still applies when architecture mode is absent.
- Vertical Slice checks only run when configured.
- Feature System diagnostics keep current rule ids and messages.
- Config template still initializes a valid `.onioncryrc.jsonc`.

## Non-Goals

- Do not change rule semantics while moving modules.
- Do not rename public rule ids.
- Do not change diagnostic wording unless covered by a separate issue.
- Do not add dependencies.
- Do not introduce a framework-heavy abstraction around rules.

## Issue Breakdown

Suggested implementation issues:

1. Split config module into typed submodules.
2. Split clean boundary rules by layer/package/context/cycle.
3. Split diagnostics by rule family and add an internal builder.
4. Split Feature System rules by policy and extract shared path classification.
5. Add focused regression tests for module-split-sensitive behavior if any coverage gaps appear during movement.

