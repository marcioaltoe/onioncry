# task-007: Detect circular dependencies

## Description

Add onion/circular-dependency by detecting cycles in the resolved local import graph across all analyzed local files, not only cross-layer or cross-context cycles.

## Acceptance

- Cycle detection uses resolved local import edges within the file universe.
- External package imports are ignored for cycle detection.
- Unresolved imports are ignored for cycle detection.
- The default rule severity is warn.
- Diagnostics report a canonical readable cycle path.
- Overrides and --fail-on affect circular dependency reporting consistently with other rules.
- Tests cover simple cycles, longer cycles, acyclic graphs, external imports, unresolved imports, and override suppression.
- make verify passes.
