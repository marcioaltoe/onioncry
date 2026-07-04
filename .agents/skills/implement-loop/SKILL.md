---
name: implement-loop
description: Implement every issue of a published PRD in dependency order, one at a time — read PRD + issue, follow the implement methodology, run make verify, commit once per issue, update Linear with commit evidence, and continue without between-issue confirmations. Use when the user points at a PRD/Project and wants its issues built in a loop.
argument-hint: "<prd: Linear Project name|ID|URL, or local PRD path> [--from DEV-###]"
disable-model-invocation: true
---

# Implement PRD Loop

Drive the implementation of a whole PRD by looping over its issues **in dependency order**, one
issue end-to-end at a time. The PRD is the **input** (`$ARGUMENTS`); the issues are its slices on the
tracker.

## Critical constraint

`implement`, `to-issues`, and `to-prd` are `disable-model-invocation` (user-triggered slash
commands). This skill **does not call them** — it **follows the `implement` methodology inline**
(TDD at the agreed seams → frequent typecheck/single-file tests → full suite at the end → review →
commit) using the engineering skills that *are* invocable (`coding-guidelines`, `clean-code`,
`solid`, `no-workarounds`, `testing-boss`, plus the domain skills for each layer).

## 1. Resolve the PRD

`$ARGUMENTS` first token is the PRD reference:

- **Linear Project** (preferred — `to-prd` publishes the PRD as a Project): resolve via Linear MCP
  (`list_projects` by name, or `get_project` by ID/URL) and read the **full overview** (the PRD body:
  Problem / Solution / User Stories / Implementation Decisions / Testing Decisions / Out of Scope).
- **Local PRD file path**: read the file; then find its Project on the tracker for the issues.

Echo the resolved PRD (title + one-line summary) and loop plan, then **confirm with the user**
before starting the loop. After this initial confirmation, **do not ask for confirmation between
issues**.

Flag: `--from DEV-###` resumes at a specific issue.

## 2. Build the ordered work list

- List the issues under the PRD's **Project** (`list_issues` filtered by project). Capture for each:
  identifier, title, status, **blocked-by** relations, and Type/Scope labels.
- Implement only issues that are **`ready-for-agent`** and not yet **Done**.
- Order them by **topological sort of blocked-by**: an issue is *ready* only when **all its blockers
  are Done/merged**. Independent issues (no blockers) can run any time.
- Print the resulting order and the dependency edges. This is the loop plan.

## 3. Loop — per issue cycle

For the next *ready* issue (respecting `--from`):

1. **Read context**: `get_issue DEV-###` (What to build / Acceptance criteria / Blocked by) + re-read
   the PRD's Implementation/Testing Decisions and Out of Scope. Respect the **domain glossary**
   (`CONTEXT.md`) and any **ADRs** in the touched area.
2. **Activate skills** (Skill Dispatch Protocol): always `coding-guidelines` + `clean-code` +
   `solid` + `no-workarounds`; then per layer — `hono-api-best-practices`+`hono`+`zod` (endpoints),
   `drizzle-orm` (DB/schema/migrations), `feature-systems-pattern`+`react`+`tanstack-*`+`shadcn`+
   read `DESIGN.md` (UI), `external-api-adapters`+`integration-contract-testing` (adapters),
   `logtape`+`observability-audit` (logging), `testing-boss`+`vitest` (tests).
3. **Branch**: stay on the user-specified/current implementation branch when the user directs that.
   Otherwise, create or continue a single `ma/` implementation branch for the PRD/scope. Do **not**
   create one branch per issue unless the user explicitly asks.
4. **Implement** following the `implement` methodology: write tests first at the **seams named in the
   PRD's Testing Decisions** (prefer existing seams), typecheck + run the relevant single test files
   frequently, run the full suite at the end. Stay **inside the slice** — honor the PRD's Out of
   Scope. Root-cause only — no workarounds, no lint/type suppressions.
5. **Verify**: `make verify` until **100%** (zero errors, zero warnings). Husky/cog also gate the
   commit.
6. **Evidence gate**: apply `evidence-gate` — re-read the issue's Acceptance criteria and confirm
   each is met by the actual diff/tests before declaring the issue done.
7. **Commit**: stage only this slice's files; commit with **Conventional Commits**
   (`conventional-commits`; this repo's `cog.toml` has `scopes = []` → **no scope**; the commit
   **type must match the issue's Type label**) and include `DEV-###` in the commit body when useful
   for traceability. Do **not** open a PR per issue. Push and PR creation are separate publish
   actions, performed only when the user explicitly requests them or when the loop is intentionally
   being published as a batch.
8. **Update Linear**: after the commit succeeds, add a comment to the issue with the commit SHA,
   branch, verification commands/results, and a short implementation summary. Then mark the issue
   **Done** in Linear (`save_issue` with `state: "Done"`; if status names differ, resolve the
   completed status via `list_issue_statuses` first). Do not mark Done before the commit and
   verification evidence exist.
9. **Continue automatically**: advance to the next ready issue in dependency order. Do **not** ask
   the user for confirmation between issues. Continue until all ready-for-agent issues are done or a
   real HITL/blocker condition is reached.

## 4. Stop / fail loudly

Stop and surface to the user (never silently skip, fake, or weaken a test) when:

- `make verify` can't reach green after honest root-cause attempts;
- a blocker isn't Done and the required code is not present in the current branch;
- an Acceptance criterion needs a product/design decision not in the PRD;
- the work would exceed the slice's scope (the PRD marks it Out of Scope).
- Linear cannot be updated after a completed commit; report the commit SHA, verification evidence,
  and the failed tracker action so the user can decide whether to retry or repair manually.

If an issue is `ready-for-agent`, unblocked by the current dependency graph, and its requirements are
specified by the PRD/issue, continuing is **not** HITL. Keep going without confirmation.

Report progress as a checklist (`DEV-### ✓ / in-progress / blocked`) only at natural stop points
(all ready issues done, HITL, blocker, verification failure, or Linear update failure). Resume later
with `--from DEV-###`.

## Guardrails

- One **commit per issue**. Do **not** open one PR per issue; PRs are explicit publish/batch actions.
- After the initial loop confirmation, do **not** ask for confirmation between ready issues.
- Do **not** create per-issue branches unless the user explicitly asks; any agent-created branch must
  still use the mandatory `ma/` prefix.
- **Never** run destructive git (`reset`/`restore`/`clean`/`checkout -- `) without explicit user
  permission.
- Do **not** modify the parent Project/PRD or close other issues.
- Process in dependency order; for this repo the `to-issues` output usually means
  `prefactor → first vertical slice → parallel slices → docs`.
