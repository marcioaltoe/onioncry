# Add `onioncry check --files`

Status: done

## Parent

../PRD.md

## What to build

Add a `--files <path>...` option to `check` that filters the report to the given files while keeping the analysis whole-project, so pre-commit hooks and agent loops get fast, focused output that never disagrees with a full run.

## Acceptance criteria

- [ ] `onioncry check --files <path>...` accepts one or more project-relative paths (repeated flag or space-separated values, matching clap conventions).
- [ ] The file universe, import graph, and boundary classification are computed exactly as in an unscoped run; scoping filters only the reported violations.
- [ ] File-located violations are reported only when their file is in the `--files` list.
- [ ] Project-level violations without a single file location, such as `cleanarch/no-context-cycle`, are always reported.
- [ ] Paths not in the file universe are listed on stderr as skipped without failing the run.
- [ ] Exit code follows the filtered report and the effective failure threshold; a scoped run with no reported violations exits 0 even when other files have violations.
- [ ] `--files` composes with `--format json`, `--llm-mode`, `--tips`, `--fail-on`, and the baseline flags; JSON summary counts reflect the filtered report.
- [ ] CLI integration tests cover scoped pass, scoped fail, out-of-universe paths, project-level violations, and format composition.
- [ ] README documents the flag with a pre-commit example.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately

## Comments

Implemented as report scoping in `run_check_with_options` after baseline
application; violations with a `cyclePath` are treated as project-level and
always reported. One deliberate deviation from the acceptance criteria:
`--files` conflicts with `--write-baseline` (clap-level error) instead of
composing with it, because writing a baseline from a scoped run would silently
drop entries for out-of-scope files. `--baseline` and `--no-baseline` compose
normally. Glossary entry: File-Scoped Check.
