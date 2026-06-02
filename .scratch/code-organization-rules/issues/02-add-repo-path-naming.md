# Add `repo/path-naming`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a built-in, configurable code organization rule that checks file and directory names. The rule should focus on observable path conventions such as kebab-case, descriptive suffixes, plural collection directories, singular feature directories, and `infra` as the default outer layer spelling.

## Acceptance criteria

- [ ] `repo/path-naming` is accepted in the `rules` map with normal severity and option shapes.
- [ ] File and directory path checks do not inspect class names, function names, variable names, constants, or interfaces.
- [ ] Files are checked for kebab-case and configured suffix conventions where the file category is identifiable.
- [ ] Collection directories such as entities, repositories, value-objects, use-cases, events, services, gateways, and dtos are expected to be plural by default.
- [ ] Feature or context directory segments are expected to be singular and kebab-case by default.
- [ ] The default layer directory vocabulary uses domain, application, infra, and shared.
- [ ] Explicit rule options can customize suffixes, collection directories, feature roots, and layer directory names.
- [ ] CLI tests cover valid paths, invalid paths, and customized options.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately
