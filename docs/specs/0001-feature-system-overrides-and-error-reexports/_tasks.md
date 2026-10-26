---
schema: spec-tasks/v1
spec: 0001-feature-system-overrides-and-error-reexports
qa: task_03
graph:
  nodes:
    - id: task_01
      file: task_01.md
      needs: []
    - id: task_02
      file: task_02.md
      needs: [task_01]
    - id: task_04
      file: task_04.md
      needs: [task_02]
    - id: task_03
      file: task_03.md
      needs: [task_04]
---

# Tasks — Feature-system corrections

| id | title | type | complexity | needs |
| --- | --- | --- | --- | --- |
| task_01 | Honor each diagnostic owner's effective rule | backend | high | — |
| task_02 | Prove named Error class exports with the parser | backend | high | task_01 |
| task_04 | Reject runtime namespaces that shadow Error | backend | medium | task_02 |
| task_03 | Run terminal CLI QA | qa | high | task_04 |

Waves: 1 → task_01 · 2 → task_02 · 3 → task_04 · 4 → task_03.
