# Render baseline results in pretty, LLM, and JSON output

Status: done

## Parent

../PRD.md

## What to build

Surface baselined violations distinctly in every output format so debt stays visible without blocking, and document the baseline workflow.

## Acceptance criteria

- [x] The pretty summary shows a separate `baselined` count alongside errors and warnings; baselined findings are visually distinct from active violations.
- [x] `--llm-mode` groups baselined findings separately so agents do not treat them as actionable failures.
- [x] JSON output marks baselined violations with `baselined: true` and adds the baselined count to the summary; existing JSON fields remain unchanged for runs without a baseline.
- [x] `check --tips` output for baselined findings explains how to shrink the baseline (fix and rerun `--write-baseline`).
- [x] README documents the adoption workflow: enable strict rules, write the baseline, commit it, ratchet down over time.
- [x] CLI integration tests assert each format's baseline presentation.
- [x] `make verify` passes.

## Blocked by

02-consume-baseline-during-check

## Comments

- 2026-07-04: Implemented distinct baselined rendering for pretty output, `--tips`, LLM groups, and JSON compatibility, and documented the baseline adoption and ratchet workflow in README.md. Verification: `rtk cargo test --test cli_check baselined` passed with 3 tests; `rtk make verify` passed with clippy clean and 100 tests. The exact commit SHA is reported by the implementation loop after the slice commit is created.
