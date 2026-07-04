# Add baseline file format and `--write-baseline`

Status: done

## Parent

../PRD.md

## What to build

Define the violation baseline file format, the fingerprint model, and the `onioncry check --write-baseline` flag that records current violations. Capture the design as ADR `docs/adr/0010-add-a-violation-baseline.md` and add the new glossary entries to CONTEXT.md.

## Acceptance criteria

- [x] A baseline entry fingerprint is `rule + file + target`, where `target` is the violating import specifier or rule-specific subject; line and column are excluded.
- [x] Identical violations in one file collapse into a single entry with a `count` field.
- [x] The baseline file is JSON with a `version: 1` field, serialized with serde and sorted deterministically (by file, then rule, then target) for stable diffs.
- [x] `onioncry check --write-baseline` writes `.onioncry-baseline.json` next to the resolved configuration file and reports the created path and entry count on stderr.
- [x] `onioncry check --baseline <path> --write-baseline` writes to the given path instead.
- [x] Writing a baseline does not change the run's report, exit code, or failure threshold.
- [x] Running `--write-baseline` on a clean project writes an empty baseline (`version` plus empty entries) rather than failing.
- [x] ADR 0010 records the fingerprint choice, the stale-entry policy, and the explicit-grandfathering rationale.
- [x] CONTEXT.md gains glossary entries for Violation Baseline, Baseline Fingerprint, Baselined Violation, and Stale Baseline Entry.
- [x] CLI integration tests cover writing, rewriting, custom path, and deterministic ordering.
- [x] `make verify` passes.

## Blocked by

None - can start immediately

## Comments

- 2026-07-04: Implemented the baseline serde model, deterministic fingerprint aggregation, `check --write-baseline`, custom `--baseline <path>` writes, ADR 0010, and CONTEXT.md glossary entries. Verification: `rtk cargo test --test cli_check write_baseline` passed with 3 tests; `rtk make verify` passed with clippy clean and 91 tests. The exact commit SHA is reported by the implementation loop after the slice commit is created.
