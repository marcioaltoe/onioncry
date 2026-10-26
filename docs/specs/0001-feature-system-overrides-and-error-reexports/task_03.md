---
task: task_03
spec: 0001-feature-system-overrides-and-error-reexports
status: completed
type: qa
complexity: high
---

# Task 03: Run terminal CLI QA

## Overview

Execute qa-gate against the assembled correction using the Run-built OnionCry binary and isolated temporary source projects. Preserve the seeded report, mechanical rows and typed inputs; record real commands, files, exits and JSON diagnostics.

## Requirements

1. MUST execute the qa-gate skill and cover every PRD goal, Core Feature, Success Metric and TechSpec API Contract with auditable real CLI evidence. Test check and explain using their actual --help contract; no new flags are assumed.
2. MUST cover per-file off/warn/error and last-match option replacement across five families, stable aggregate owners and cross-file query/adapter relationships. Prove exempt files coexist with invalid production imports under --fail-on warning.
3. MUST cover every declared valid/invalid runtime Error export form, the negative adapter import boundary and bounded cycle/unresolved/excluded cases. Use generic local fixtures; do not copy private consumer source into this public repository.
4. MUST include a named outside-evidence row comparing an alias/reexport fixture with ECMAScript ExportEntries semantics recorded in docs/references/2026-10-26-feature-system-contracts.md, then inspect the binary's diagnostic output.
5. MUST verify operative Project Constraints, exact changed-file scope and tooling authorization ancestry. Confirm the canonical skill describes actual behavior. Run make verify and record warnings/failures without weakening checks.

## Acceptance Criteria

- [ ] Terminal report proves the assembled behavior and preserved CLI contracts.
- [ ] Production-negative, aggregate ownership, bounded export proof and outside-source rows have auditable evidence.
- [ ] Repository gate passes and all scope/authorization rows are reconciled.

## Unreachable Acceptance

- The eventual code PR and its current-head CI do not exist during terminal QA; the Supervisor owns review and CI before merge.
- Registry and release-workflow evidence is unavailable before the later release; publication is verified through docs/release.md after code delivery.

## Verification

- `newest="$(find 'docs/specs/0001-feature-system-overrides-and-error-reexports/qa' -type f -name "qa-report-*.md" -print 2>/dev/null | awk "{ report=\$0; name=\$0; parts=split(name, path, \"/\"); name=path[parts]; name=substr(name, 11, length(name)-13); date=substr(name, 1, 10); suffix=substr(name, 11); shape=(length(date) == 10 && substr(date, 5, 1) == \"-\" && substr(date, 8, 1) == \"-\"); for (i=1; i <= 10 && shape; i++) { if (i != 5 && i != 8 && index(\"0123456789\", substr(date, i, 1)) == 0) shape=0 } year=substr(date, 1, 4)+0; month=substr(date, 6, 2)+0; day=substr(date, 9, 2)+0; leap=(year%400 == 0 || (year%4 == 0 && year%100 != 0)); days=(month == 2 ? 28+leap : ((month == 4 || month == 6 || month == 9 || month == 11) ? 30 : 31)); dated=(shape && month >= 1 && month <= 12 && day >= 1 && day <= days); if (!dated) date=\"\"; if (suffix == \"\") { sequenced=1; sequence=-1 } else { sequenced=(length(suffix) > 1 && substr(suffix, 1, 1) == \"-\"); for (i=2; i <= length(suffix) && sequenced; i++) { if (index(\"0123456789\", substr(suffix, i, 1)) == 0) sequenced=0 } sequence=(sequenced ? substr(suffix, 2)+0 : 0) } if (dated && sequenced) printf \"%s\\t%s\\t%s\\t%s\\t%s\\n\", dated, date, sequenced, sequence, report }" | sort -k1,1n -k2,2 -k3,3n -k4,4n -k5,5 | tail -1 | cut -f5-)"; test -n "$newest" || exit 1; awk "BEGIN { whitespace=\" \t\r\n\f\v\" } NR == 1 && \$0 == \"---\" { frontmatter=1; next } frontmatter && \$0 == \"---\" { closed=1; exit } frontmatter && index(\$0, \"verdict:\") == 1 { verdict=substr(\$0, 9); while (length(verdict) > 0 && index(whitespace, substr(verdict, 1, 1)) > 0) verdict=substr(verdict, 2); while (length(verdict) > 0 && index(whitespace, substr(verdict, length(verdict), 1)) > 0) verdict=substr(verdict, 1, length(verdict)-1); verdicts++ } END { exit(closed && verdicts == 1 && length(verdict) > 0 ? 0 : 1) }" "$newest"` — expected: exit 0.

Comando derivado pelo Roundfix, reproduzido sem alteração além do caminho da Spec. Inspeciona somente o relatório de QA mais novo da Spec.

## References

- `_prd.md` → Goals 1–3; Core Features 1–3; Success Metrics 1–3.
- `_techspec.md` → API Contract 1; Build Order 3; Testing Approach.
