# Enforce bounded context public surface

Status: ready-for-agent

## What to build

Add bounded context classification and enforce `onion/no-cross-context-internal-import`. This slice should ensure that cross-context local imports target the imported context's public surface segments rather than internal details.

## Acceptance criteria

- [ ] Files are classified against configured `contexts.*.patterns`.
- [ ] A file matching more than one context produces an ambiguous classification diagnostic.
- [ ] A file matching no context is allowed in the MVP.
- [ ] Cross-context checks run only when both source and target files have contexts.
- [ ] `contextRules.default.allowSameContext` allows same-context imports when true.
- [ ] `contextRules.default.allowCrossContext` allows imports into configured public surface path segments such as `contracts`, `events`, `ports`, and `shared`.
- [ ] Imports into another context's internal details produce `onion/no-cross-context-internal-import`.
- [ ] `shared` is not treated as a context by default.
- [ ] Tests cover same-context imports, public surface imports, internal cross-context imports, contextless files, ambiguous context classification, and shared segment behavior.
- [ ] `make verify` passes.

## Blocked by

- 04-apply-linter-style-rule-policy.md
