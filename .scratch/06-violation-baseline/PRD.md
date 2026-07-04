# Violation Baseline

Status: ready-for-agent

## Summary

OnionCry will support a committed violation baseline so existing repositories can adopt strict rules gradually. Known violations are recorded once, reported as baselined without failing the run, and only new violations block automation. This is explicit, reviewable grandfathering — the Forward-Looking Default stays intact because the debt is named in a committed file instead of weakening rule defaults.

## Decisions

- A baseline entry fingerprint is `rule + file + target`, where `target` is the violating import specifier or rule-specific subject. Line and column are excluded so editing a file does not invalidate its baseline entries.
- Identical violations in the same file collapse into one entry with a `count` field. When a run produces more matches than `count`, the excess is reported as new violations.
- The baseline file is `.onioncry-baseline.json`, resolved relative to the configuration file. `onioncry check --baseline <path>` overrides the location.
- `onioncry check --write-baseline` writes the current violations to the baseline file, sorted deterministically for stable diffs.
- When a baseline file exists, `onioncry check` consumes it automatically. `--no-baseline` disables it for one run.
- Baselined violations do not affect the failure threshold. They appear in the summary as a separate `baselined` count and are listed in JSON output with a `baselined: true` marker.
- Stale baseline entries (entries matching no current violation) produce a stderr warning that names the count and suggests rerunning `--write-baseline`. They never fail the run.
- The baseline never changes boundary classification or the file universe; it only changes how matched violations are reported.
- The baseline format version is recorded in the file (`version: 1`) and is public API.
- The decision is captured as ADR `docs/adr/0010-add-a-violation-baseline.md`.
- New CONTEXT.md glossary entries are added for Violation Baseline, Baseline Fingerprint, Baselined Violation, and Stale Baseline Entry.

## Behavior Summary

| Situation | Result |
| --- | --- |
| Violation matches a baseline entry within `count` | Reported as baselined, does not fail |
| Violation matches no baseline entry | Normal violation, may fail the run |
| More matches than `count` for one fingerprint | Excess reported as new violations |
| Baseline entry matches nothing | Stderr stale warning, run still passes |

## Issue Breakdown

1. Baseline file format, fingerprinting, and `--write-baseline`
2. Baseline consumption during `check`
3. Baseline reporting in pretty, LLM, and JSON output
