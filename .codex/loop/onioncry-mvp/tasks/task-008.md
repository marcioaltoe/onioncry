# task-008: Generate init template

## Description

Implement onioncry init to create a conservative commented .onioncryrc.jsonc starter configuration with the agreed MVP preset and project-specific TODOs.

## Acceptance

- onioncry init creates .onioncryrc.jsonc in the current project root.
- Existing .onioncryrc.jsonc is not overwritten unless --force is provided.
- The template includes $schema, version, project, aliases, layers, contexts, contextRules, rules, and overrides.
- The template uses domain, application, infra, and shared layers with mayImport examples.
- The template includes default rules: layer leak and cross-context internal import as errors; external package policy with domain: error, application: warn, infra: off; unresolved imports, circular dependencies, and unclassified files as warnings.
- The template uses JSONC comments to mark TODOs for project-specific patterns, aliases, contexts, and allowlists.
- Tests cover creation, no-overwrite behavior, force overwrite, and JSONC parseability after stripping comments.
- make verify passes.
