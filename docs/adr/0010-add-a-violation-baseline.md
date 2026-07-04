# Add a violation baseline

OnionCry will support a committed violation baseline file named `.onioncry-baseline.json` so existing repositories can adopt strict architecture rules without weakening the Forward-Looking Default. The baseline is explicit grandfathering: current debt is named in a reviewable artifact, while new violations still fail according to the configured threshold.

Baseline entries use the fingerprint `rule + file + target`. The `file` is stored relative to the analyzed project root so the baseline can be committed across machines. The `target` is the violating import specifier for import diagnostics, or the most stable rule-specific subject for file-level diagnostics. Line and column are intentionally excluded because routine edits should not invalidate a baseline entry.

Identical current violations with the same fingerprint collapse into one entry with a `count` field. If a later run finds more current violations than the recorded count, the excess remains new debt and can fail the run. Entries are serialized as JSON with `version: 1` and sorted by file, then rule, then target for stable diffs.

When a baseline entry matches no current violation, OnionCry treats it as stale debt metadata: the run emits a non-blocking stderr warning and suggests rerunning `--write-baseline`. Stale entries do not fail the run because removing fixed debt from the baseline should be a ratchet workflow, not a blocker that hides the original architecture result.
