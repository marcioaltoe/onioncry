# Render baseline results in pretty, LLM, and JSON output

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Surface baselined violations distinctly in every output format so debt stays visible without blocking, and document the baseline workflow.

## Acceptance criteria

- [ ] The pretty summary shows a separate `baselined` count alongside errors and warnings; baselined findings are visually distinct from active violations.
- [ ] `--llm-mode` groups baselined findings separately so agents do not treat them as actionable failures.
- [ ] JSON output marks baselined violations with `baselined: true` and adds the baselined count to the summary; existing JSON fields remain unchanged for runs without a baseline.
- [ ] `check --tips` output for baselined findings explains how to shrink the baseline (fix and rerun `--write-baseline`).
- [ ] README documents the adoption workflow: enable strict rules, write the baseline, commit it, ratchet down over time.
- [ ] CLI integration tests assert each format's baseline presentation.
- [ ] `make verify` passes.

## Blocked by

02-consume-baseline-during-check
