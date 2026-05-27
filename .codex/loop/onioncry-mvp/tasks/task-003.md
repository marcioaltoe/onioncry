# task-003: Enforce layer boundaries

## Description

Add layer classification and enforce onion/no-layer-leak using explicit mayImport rules. Projects can define domain, application, infra, and shared patterns and receive actionable diagnostics when local imports violate allowed layer policy.

## Acceptance

- Files are classified against configured layers.*.patterns.
- A file matching more than one layer produces an ambiguous classification diagnostic.
- A file matching no layer produces onion/unclassified-file according to rule severity.
- onion/no-layer-leak compares local resolved import edges with the source layer mayImport list.
- Type-only imports and re-exports count as layer dependencies.
- Unclassified files do not participate in onion/no-layer-leak.
- Pretty and JSON output include fromLayer, toLayer, import string, file path, line/column when available, and an optional suggestion.
- Tests cover valid layer imports, forbidden layer imports, ambiguous classification, unclassified files, and type-only imports.
- make verify passes.
