# Add global slice artifacts rule

Status: ready-for-agent

## Parent

../PRD.md

## User stories covered

- 29, 30, 32, 33

## What to build

Add `verticalslice/no-global-slice-artifacts` to warn when files that look like slice artifacts live outside the configured slice root. The rule must avoid treating approved global bootstrap, shared, config, library, or technical infrastructure folders as automatic violations.

## Acceptance criteria

- [ ] `verticalslice/no-global-slice-artifacts` is accepted in `rules` with `off`, `warn`, `error`, and `[severity, options]`.
- [ ] The generated starter configuration sets the rule to `warn` for Vertical Slice mode.
- [ ] Files with configured slice artifact suffixes outside the slice root report violations unless they live in an allowed global folder.
- [ ] Files inside the configured slice root do not report global artifact violations.
- [ ] Allowed global folders are configurable.
- [ ] Global `domain`, `application`, or `infra` folders are not rejected automatically without matching slice artifact evidence.
- [ ] Pretty and JSON violations include detected artifact role, current path, configured slice root, configured architecture mode, and rule name.
- [ ] Tests cover default allowed globals, custom allowed globals, root-level slice mode, valid slice artifacts, misplaced artifact suffixes, and disabled rule behavior.
- [ ] `make verify` passes.

## Blocked by

- 04-add-vertical-slice-layout-config.md
