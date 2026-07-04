# Add `repo/test-placement`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a built-in, configurable code organization rule that reports misplaced test files. The default should be forward-looking: source-level unit tests belong in co-located `__tests__` folders, while integration and E2E tests use dedicated workspace-level roots.

## Acceptance criteria

- [ ] `repo/test-placement` is accepted in the linter-style `rules` map with `off`, `warn`, `error`, and `[severity, options]`.
- [ ] With only `"repo/test-placement": "error"`, unit test files under source are valid only when they are inside a `__tests__` directory.
- [ ] Integration tests under `tests/integration/<context>/` are valid by default.
- [ ] E2E tests under `tests/e2e/<context>/` are valid by default.
- [ ] Misplaced `*.test.*` and `*.spec.*` files report violations with useful paths and suggestions.
- [ ] Overrides can lower or disable the rule for legacy test placement without changing the file universe.
- [ ] CLI tests cover passing, failing, and override behavior.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately
