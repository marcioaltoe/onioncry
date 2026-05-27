# Apply linter-style rule policy

Status: ready-for-agent

## What to build

Implement the rule configuration engine used by all checks: linter-style rule names, severities, `[severity, options]` values, per-file overrides, final status calculation, and JSON summary. This slice should make policy behavior predictable before adding more rules.

## Acceptance criteria

- [ ] Rule values support `off`, `warn`, `error`, and `[severity, options]`.
- [ ] Unknown rule names and invalid severities produce clear config errors.
- [ ] Overrides match files by glob and can replace rule severity/options for matching files.
- [ ] Overrides apply in array order; the last matching override wins for the same rule.
- [ ] Overrides do not change file universe, aliases, layers, or contexts.
- [ ] `status` reflects the effective `--fail-on` threshold while `summary` keeps raw warning/error counts.
- [ ] JSON output uses the same `onion/...` rule names as config.
- [ ] Tests cover base rules, disabled rules, rule options, override replacement, last-override-wins, and `--fail-on` behavior.
- [ ] `make verify` passes.

## Blocked by

- 03-enforce-layer-boundaries.md
