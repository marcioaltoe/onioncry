---
schema: roundfix/archive-record/v1
spec: 0002-suffixed-artifacts-require-their-role-folder
title: Suffixed artifacts require their role folder
status: archived
created: "2026-11-06"
archived: "2026-11-06"
disposition: partial
source: docs/specs/0002-suffixed-artifacts-require-their-role-folder
source_revision: 3f5db7d903760c8a97629d703f9c36e38bcaf928
qa_task: task_02
qa_report: qa-report-2026-11-06.md
qa_verdict: partial
unproven: []
adrs:
  - ADR-0008
  - ADR-0003
  - ADR-0012
sources: []
regeneration: []
promoted: []
pull_request: "21"
---

# Suffixed artifacts require their role folder

This gate leaves a qualifying pre-PR `partial`: all runnable checks pass and the only unmet row is the Pull Request row before a PR exists. An eventual Archive Record must preserve this report, the seven passing rows and the unproven review/CI/publication boundary; this QA Agent neither archives nor performs downstream delivery.
