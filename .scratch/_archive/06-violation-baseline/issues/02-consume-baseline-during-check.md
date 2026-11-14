# Consume the baseline during `check`

Status: done

## Parent

../PRD.md

## What to build

Make `onioncry check` consume an existing baseline: violations matching baseline entries are reclassified as baselined and stop affecting the failure threshold, while stale entries produce a non-blocking stderr warning.

## Acceptance criteria

- [x] When `.onioncry-baseline.json` exists next to the resolved configuration file, `onioncry check` consumes it automatically; `--baseline <path>` overrides the location and `--no-baseline` disables consumption for one run.
- [x] A violation matching a baseline fingerprint within its `count` is marked baselined and does not affect the failure threshold or exit code.
- [x] When a fingerprint matches more current violations than `count`, the excess violations are reported as new violations that may fail the run.
- [x] Baseline entries matching no current violation produce one stderr warning naming the stale entry count and suggesting `--write-baseline`; the run still passes.
- [x] A missing `--baseline <path>` file or an unsupported baseline `version` is a configuration error (exit 2) naming the path and the next useful action.
- [x] The baseline never changes boundary classification, the file universe, or which rules run.
- [x] `--fail-on warning` interacts correctly: baselined violations never fail; new warnings fail only under that threshold.
- [x] CLI integration tests cover matched, excess-count, stale, `--no-baseline`, and error scenarios.
- [x] `make verify` passes.

## Blocked by

01-add-baseline-format-and-write

## Comments

- 2026-07-04: Implemented automatic baseline consumption for normal `check` runs, explicit `--baseline <path>` reads, `--no-baseline`, stale-entry warnings, baseline version and missing-file errors, active-only failure thresholds, and baselined violation markers. Verification: `rtk cargo test --test cli_check baseline` passed with 9 tests; `rtk make verify` passed with clippy clean and 97 tests. The exact commit SHA is reported by the implementation loop after the slice commit is created.
