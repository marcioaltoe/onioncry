# Add cross-slice internal import rule

Status: ready-for-agent

## Parent

../PRD.md

## User stories covered

- 25, 26, 27, 28, 32, 33

## What to build

Add `verticalslice/no-cross-slice-internal-import` to report imports from one slice into another slice's internal files. Cross-slice imports must target the imported slice's configured public surface.

## Acceptance criteria

- [ ] `verticalslice/no-cross-slice-internal-import` is accepted in `rules` with `off`, `warn`, `error`, and `[severity, options]`.
- [ ] Imports from one slice to another slice's `index.ts` are valid by default.
- [ ] Imports from one slice to another slice's `contracts/` files are valid by default.
- [ ] Default slice identity uses `features/<domain>/<operation>` when `sliceDepth` is `2`.
- [ ] Imports from one slice to another slice's internal `handlers/`, `adapters/`, `domain/`, `__tests__/`, or internal service files report violations.
- [ ] Imports within the same slice are valid.
- [ ] Imports from allowed global folders are handled according to existing local import rules and do not become cross-slice violations by default.
- [ ] Type-only imports and re-exports count as slice dependencies.
- [ ] Pretty and JSON violations include source slice, target slice, imported path, public surface expectation, configured architecture mode, and rule name.
- [ ] Tests cover valid public imports, invalid internal imports, same-slice imports, type-only imports, re-exports, custom public surface options, and disabled rule behavior.
- [ ] `make verify` passes.

## Blocked by

- 04-add-vertical-slice-layout-config.md
