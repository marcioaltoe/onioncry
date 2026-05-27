# OnionCry MVP Handoff

Last updated: 2026-05-27

## Focus

Continue from the local MVP planning artifacts under `.scratch/onioncry-mvp`. The next session is expected to start implementation or restart tracked execution from the approved MVP issues.

## Current State

- Branch: `ma/context-plan-scaffold`.
- Last committed/pushed planning commit: `3497c05 docs(repo): define onioncry mvp architecture`.
- `make verify` passed after the planning docs were created; Rust checks are skipped until `Cargo.toml` exists.
- The repository currently has unrelated uncommitted skill changes in the worktree: `skills-lock.json`, `.agents/skills/{handoff,prototype,triage}/`, and `.claude/skills/{handoff,prototype,triage}`. Do not revert them unless the user explicitly asks.
- A previous attempt initialized local Codex Loop tracking under `.codex/loop/onioncry-mvp`, but that directory is not currently present. `detect-next.py onioncry-mvp` currently reports `action=bootstrap`.

## Primary Artifacts

Use these instead of reconstructing decisions from chat:

- MVP PRD: `.scratch/onioncry-mvp/PRD.md`
- MVP implementation issues: `.scratch/onioncry-mvp/issues/`
- Domain glossary: `CONTEXT.md`
- ADRs:
  - `docs/adr/0001-use-infra-as-the-default-outer-layer.md`
  - `docs/adr/0002-use-jsonc-for-configuration.md`
  - `docs/adr/0003-use-linter-style-configuration.md`
  - `docs/adr/0004-use-oxc-parser-for-javascript-and-typescript.md`
- Agent rules: `AGENTS.md`

## Important Decisions

Do not duplicate full PRD or ADR content here. The key constraints to preserve are:

- Config is `.onioncryrc.jsonc`.
- Config and JSON output use camelCase.
- Rules use `onion/...` names with `off`, `warn`, and `error`.
- Overrides only change rule severity/options, never file universe, aliases, layers, or contexts.
- Default layers are `domain`, `application`, `infra`, and `shared`.
- `infra` intentionally groups adapters and framework/driver details.
- Contexts mean bounded contexts or capability boundaries.
- Cross-context imports are allowed only through configured public surface segments.
- External packages are default-deny in sensitive layers: `domain` errors, `application` warns, `infra` is off by default.
- Use `oxc_parser` for JavaScript and TypeScript scanning.

## Suggested Next Step

Start with `.scratch/onioncry-mvp/issues/01-bootstrap-cli-check.md`.

If using Codex Loop/Ralph Loop, bootstrap tracking from the issue list because `.codex/loop/onioncry-mvp` is absent:

```text
[[CODEX_LOOP name="onioncry-mvp" rounds="1"]]
Use the codex-loop skill. Bootstrap or continue tracked execution from .scratch/onioncry-mvp, then run exactly one tracked action.
```

If not using the loop, implement issue `01-bootstrap-cli-check.md` directly and keep the diff scoped to that slice.

## Suggested Skills

- `codex-loop:codex-loop`: use if testing Ralph/Codex Loop or doing restart-safe tracked execution.
- `tdd`: use for implementation slices, especially once the Rust crate is scaffolded.
- `diagnose`: use when `make verify`, Rust checks, parser behavior, or CLI behavior fails.
- `grill-with-docs`: use only if a new architecture decision needs clarification against `CONTEXT.md` and ADRs.
- `to-issues`: use only if the MVP issue breakdown needs to be changed.

## Guardrails For The Next Agent

- Read `AGENTS.md`, `CONTEXT.md`, and relevant ADRs before implementation.
- Use `rtk` for repository shell commands when available.
- Do not create commits unless the user asks in the current turn.
- Do not hand-edit dependency manifests; use Cargo commands after the Rust crate exists.
- Run `make verify` before claiming completion.
- Preserve unrelated worktree changes.

## Redactions

No secrets or personal credentials are included in this handoff.
