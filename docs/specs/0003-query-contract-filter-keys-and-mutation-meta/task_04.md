---
task: task_04
spec: 0003-query-contract-filter-keys-and-mutation-meta
status: completed
type: qa
complexity: high
---

# Task 04: Run terminal CLI QA

## Overview

Execute qa-gate against the assembled change using the Run-built OnionCry binary and isolated temporary source projects. Record real commands, files, exits and JSON diagnostics. This node also carries the Spec's glossary check.

## Requirements

1. MUST execute the qa-gate skill and cover every PRD Goal, User Story, Core Feature, Success Metric and TechSpec API Contract 1 with auditable real CLI evidence. Exercise `check` and `explain` through their actual `--help` contract; no new flags are assumed.
2. MUST reproduce the reported shape with generic local fixtures: hooks that read through factories and invalidate through key factories, components that reload through key factories, and an export mutation with `meta: { invalidates: [] }`, producing zero `frontend/feature-system-query-contract` diagnostics. Do not copy consumer source.
3. MUST cover the negative controls of Success Metric 2: inline keys in hooks, components and routes, a mutation with no invalidation contract, a mutation whose `meta` lacks `invalidates`, an optimistic update without rollback, and a production component importing an adapter, each keeping its diagnostic and failing under `--fail-on warning`.
4. MUST include a named outside-evidence row that compares the `meta: { invalidates: [] }` fixture with the TanStack maintainer guidance quoted in `docs/references/2026-11-06-query-contract-filters-and-mutation-meta.md`, then inspects the binary's JSON for the three variants it names.
5. MUST verify the operative Project Constraints, the exact changed-file scope and the tooling authorization ancestry, and confirm that README, `CONTEXT.md` and `.agents/skills/onioncry/SKILL.md` describe the shipped behavior. Run `make verify` and record warnings or failures without weakening checks.
6. MUST run the glossary check from `docs/agents/domain.md`: confirm the changed **Feature System Query Contract** definition matches the shipped behavior and that no other term was coined, changed or dropped.

## Acceptance Criteria

- [ ] The terminal report proves both contracts and the preserved CLI contracts.
- [ ] The positive, negative and outside-evidence rows have auditable evidence.
- [ ] The repository gate passes and every scope and authorization row is reconciled.

## Unreachable Acceptance

- The code PR and its current-head CI do not exist during terminal QA; the Supervisor owns review and CI before merge.
- Registry and release-workflow evidence is unavailable before the later release; publication is verified through `docs/release.md` after code delivery.

## Verification

- `newest="$(find 'docs/specs/0003-query-contract-filter-keys-and-mutation-meta/qa' -type f -name "qa-report-*.md" -print 2>/dev/null | awk "{ report=\$0; name=\$0; parts=split(name, path, \"/\"); name=path[parts]; name=substr(name, 11, length(name)-13); date=substr(name, 1, 10); suffix=substr(name, 11); shape=(length(date) == 10 && substr(date, 5, 1) == \"-\" && substr(date, 8, 1) == \"-\"); for (i=1; i <= 10 && shape; i++) { if (i != 5 && i != 8 && index(\"0123456789\", substr(date, i, 1)) == 0) shape=0 } year=substr(date, 1, 4)+0; month=substr(date, 6, 2)+0; day=substr(date, 9, 2)+0; leap=(year%400 == 0 || (year%4 == 0 && year%100 != 0)); days=(month == 2 ? 28+leap : ((month == 4 || month == 6 || month == 9 || month == 11) ? 30 : 31)); dated=(shape && month >= 1 && month <= 12 && day >= 1 && day <= days); if (!dated) date=\"\"; if (suffix == \"\") { sequenced=1; sequence=-1 } else { sequenced=(length(suffix) > 1 && substr(suffix, 1, 1) == \"-\"); for (i=2; i <= length(suffix) && sequenced; i++) { if (index(\"0123456789\", substr(suffix, i, 1)) == 0) sequenced=0 } sequence=(sequenced ? substr(suffix, 2)+0 : 0) } if (dated && sequenced) printf \"%s\\t%s\\t%s\\t%s\\t%s\\n\", dated, date, sequenced, sequence, report }" | sort -k1,1n -k2,2 -k3,3n -k4,4n -k5,5 | tail -1 | cut -f5-)"; test -n "$newest" || exit 1; awk "BEGIN { whitespace=\" \t\r\n\f\v\" } NR == 1 && \$0 == \"---\" { frontmatter=1; next } frontmatter && \$0 == \"---\" { closed=1; exit } frontmatter && index(\$0, \"verdict:\") == 1 { verdict=substr(\$0, 9); while (length(verdict) > 0 && index(whitespace, substr(verdict, 1, 1)) > 0) verdict=substr(verdict, 2); while (length(verdict) > 0 && index(whitespace, substr(verdict, length(verdict), 1)) > 0) verdict=substr(verdict, 1, length(verdict)-1); verdicts++ } END { exit(closed && verdicts == 1 && length(verdict) > 0 ? 0 : 1) }" "$newest"` — expected: exit 0.

Command derived by Roundfix, reproduced unchanged apart from the Spec path. It inspects only the Spec's newest QA report.

## References

- `_prd.md` → Goals 1–3; User Stories 1–4; Core Features 1–4; Success Metrics 1–3.
- `_techspec.md` → API Contract 1; Build Order 4; Testing Approach.
