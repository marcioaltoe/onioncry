# General Agent Instructions

OnionCry is a Rust CLI that names and checks architectural boundaries in
JS/TS source projects. These instructions apply to developing the tool
itself: a lib + thin clap binary, JSON/SARIF/LLM output contracts, and a
multi-manifest release (crates.io plus npm launcher packages).

## High priority

- **MANDATORY**: Use the relevant local skills before changing code, docs,
  tests, workflows, or agent instructions. Skill activation comes BEFORE any
  planning or code generation for that domain.
- **ALWAYS** prefix shell commands with `rtk` when it is available. In command
  chains, prefix each command.
- Before planning, implementing, reviewing, or documenting repository work,
  read `CONTEXT.md` and relevant `docs/adr/` entries. Use the glossary
  vocabulary and flag conflicts with ADRs instead of silently overriding them.
- **MUST** use `rg` / `rg --files` for local code search. Use `context7` for
  external library/API docs and `exa-web-search` for broader web/source
  research. **NEVER** use web research tools to search local code.
- **MUST** run `make verify` before claiming completion. Any format issue,
  clippy warning, build failure, or test failure is **blocking** — zero
  tolerance. PR CI validates conventions only; the local gate is the ONLY
  test gate.
- **NEVER** use workarounds in production code or tests. Fix the root cause.
- **NEVER** hand-edit `Cargo.toml` dependencies. Use `rtk cargo add` /
  `rtk cargo remove`; respect the pinned `rust-version` (MSRV 1.93).
- **NEVER** create commits unless the user explicitly asks for a commit in
  the current turn.
- **ABSOLUTELY FORBIDDEN**: `git reset`, `git checkout --`, `git restore`,
  `git clean`, or any command that discards working-directory changes
  **WITHOUT EXPLICIT USER PERMISSION**. These can permanently lose code.
- Agent-created branches **MUST** use the `ma/` prefix.

## Agent docs

Read these only when relevant to the task:

- `CONTEXT.md` — project vocabulary, command concepts, domain rules, and
  product decisions
- `docs/adr/` — architectural decisions; flag conflicts before overriding them
- `docs/release.md` — the release choreography (version sync across
  Cargo.toml and every npm manifest, dry-runs, tag)
- `docs/specs/<feature-slug>/` — spec artifacts (`_idea.md`, `_prd.md`,
  `_techspec.md`, `_tasks.md`, `task_NN.md`, `qa/`); shipped specs move to
  `docs/specs/_archived/`. Run `setup-workflow` once if the layout is
  missing. Legacy PRDs/issues still live under `.scratch/<feature>/` until
  migrated.
- `docs/agents/issue-tracker.md` — tracker conventions (local files are
  canonical)
- `docs/agents/triage-labels.md` — label mapping for issue triage skills
- `docs/agents/domain.md` — how agents consume `CONTEXT.md` and ADRs

**ALWAYS** use canonical terms from `CONTEXT.md` in command names, help text,
issue titles, test names, and user-facing explanations. If the right term is
missing, call out the gap instead of inventing new language.

## Skill dispatch

Before editing, identify the task domain and **activate every matching skill**:

- **Feature discovery or product idea**: Use `brainstorming`; product-level
  ideas go through `write-idea` (scored by `business-analyst`, debated by
  `council`, challenged by `the-fool`)
- **PRD, tech spec, or task breakdown**: Use `write-prd`, `write-techspec`,
  `write-tasks` (with `grill-with-docs`/`grilling` + `domain-modeling` for
  glossary and ADR capture)
- **Executing spec tasks**: Use `implement-task` (one task) or `implement-spec`
  (the whole graph in dependency order)
- **Final QA of a completed spec**: Use `qa-gate`; archive after release with
  `archive-spec`
- **MANDATORY** for CLI behavior, flags, stdout/stderr, exit codes, JSON
  output, dry-run behavior, non-interactive mode, or introspection:
  `agentic-cli-design`
- **ALWAYS USE** `rust` and `rust-cli` before Rust command behavior, error
  handling, output contracts, integration tests, or Cargo workflows
- **ALWAYS USE** `clap-rust` before touching clap parser code: derive
  patterns, subcommands, flag groups, value parsers, completions
- **Cutting a release**: Use `cut-release` together with `docs/release.md` —
  version sync across Cargo.toml and every npm manifest, `check-versions`,
  publish dry-runs, tag, watch the workflow
- **Tests, fixtures, golden files, integration tests**: Use `testing-boss`
  plus the relevant Rust/CLI skill
- **Implementation**: Use `coding-guidelines`
- **Bug fix or failing test**: Use `no-workarounds` plus `systematic-debugging`
- **Code review before delivery**: Use `review`
- **Docs, PRDs, ADRs, issues, PR descriptions**: Use `tech-writer`
- **Commits or PR titles**: Use `conventional-commits`
- **Completion claim**: Use `evidence-gate`
- **Session handoff**: Use `handoff`
- **OnionCry dogfooding or fixture checks**: Use `onioncry` when running
  OnionCry against architecture boundaries or validating CLI behavior on
  sample projects

## CLI behavior

- Design commands for humans **and** agents: deterministic output,
  non-interactive flags, stable exit codes, machine-readable modes.
- **MUST** keep stdout for requested command output only. Diagnostics,
  progress, and warnings go to stderr.
- Command names, flags, JSON/SARIF fields, text output relied on by tests,
  exit-code contracts (0 clean / 1 violations per `--fail-on` / 2 operational
  error), and the JSONC config shape of `.onioncryrc.jsonc` are **public
  API** — never change them casually. Versioned output formats (the
  `onioncry-llm-report` footer) keep their version markers accurate.
- Help text **MUST** be concise, truthful, and backed by implemented behavior.
- Errors **MUST** name the failed operation and the next useful action when
  one is known.

## Rust conventions

- Prefer a small, explicit Rust module structure over early abstraction.
  Keep CLI parsing, config loading, path matching, source parsing, graph
  construction, and diagnostics in separate modules; `src/main.rs` stays a
  thin clap layer over the library.
- Prefer explicit domain types over stringly typed command plumbing.
- Established crate decisions (change only via ADR): `clap` v4 derive for
  the CLI, `jsonc-parser` + `serde` for the `.onioncryrc.jsonc` config
  (comments are a public contract — ADR-0002), `schemars` for the published
  config JSON Schema, `oxc_parser`/`oxc_ast` for JS/TS parsing (ADR-0004),
  `globset` + `walkdir` for file selection, `miette` for terminal
  diagnostics, `thiserror` for typed errors.
- **NEVER** `unwrap`/`expect`/`panic!` on user-facing paths. Return typed
  errors and render diagnostics through `miette`.
- **MUST** build structured output with `serde`/`serde_json`. Hand-built
  JSON is rejected. Schema-validate SARIF output against the bundled 2.1.0
  fixture.
- **NEVER** add dependencies casually. Each dependency needs a clear job and
  is used directly by the crate that imports it. Respect MSRV 1.93.
- **Integration tests drive the real binary**: `assert_cmd` + `predicates`
  in `tempfile::TempDir` sandboxes, sharing helpers through
  `tests/support/mod.rs` (config builders, `git()` wrapper, report
  assertions). User-facing contracts are tested through the binary; pure
  parsing and domain logic get unit tests in-module.

## Testing rules

- Use Arrange, Act, Assert.
- Test observable behavior, not private implementation details.
- **NEVER** add test-only production hooks, branches, or helper methods.
- **NEVER** test mock behavior instead of system behavior.
- Parser fixtures and graph edge cases are first-class tests.
- Flaky tests are **blocking failures**, not acceptable debt.

## Commands

```bash
make help                 # List available targets
make verify               # Full gate: check-conventions + rust-verify
make rust-verify          # fmt-check, check, clippy -D warnings, test
make fmt                  # Format Rust code
make fmt-check            # Check Rust formatting
make check                # Run cargo check
make lint                 # Run clippy with warnings denied
make test                 # Run Rust tests
make build                # Build the Rust project
make doc                  # Build Rust docs without dependencies
make check-conventions    # Validate Conventional Commit history
make check-pr-title       # Validate a PR title via TITLE="feat(cli): add command"
make install              # Local alpha build install
make publish-dry-run      # cargo publish --dry-run --locked
make update               # Update Cargo.lock
make clean                # Remove Rust build artifacts
make skills-link          # Recreate .claude/skills symlinks from .agents/skills
```

## Coding style

- Rust code **MUST** be formatted with `cargo fmt`; clippy passes with zero
  warnings (`-D warnings`).
- Prefer clear names over abbreviations.
- Keep comments sparse and useful; explain non-obvious decisions, not syntax.
- Use kebab-case for markdown and workflow files.
- **NEVER** rewrite unrelated files or reformat the whole repo. Keep diffs
  limited to the task.

## Commit and pull request guidelines

Allowed commit and PR title types:

- `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

Allowed scopes (**closed list** — enforced by cocogitto in CI):

- `agents`, `ci`, `cli`, `config`, `deps`, `diagnostics`, `docs`, `graph`, `parser`, `release`, `repo`, `test`

Scopes are optional, but if a scope is used it **MUST** come from the allowed
list. PR titles follow the same format because squash-merge titles become
release-history entries.

Before opening a PR, run `make verify`. PR bodies summarize changes, call out
risk, and list validation commands run.

## Release

- **NEVER** tag a release by hand outside the `cut-release` choreography
  (`docs/release.md`): the version must agree across `Cargo.toml`,
  `npm/package.json`, and every `npm/platforms/*/package.json` (including
  `optionalDependencies`), or the publish workflow fails late.
- Sequence: bump everywhere → `npm/scripts/check-versions.mjs` →
  `make verify` → `make publish-dry-run` → commit → tag `v*` → push → watch
  the release workflow to completion → verify npm and crates.io artifacts.

## Security and configuration

- Keep secrets in `.env` and never commit them; mirror required keys in
  `.env.example`.
- **NEVER** commit generated credentials, tokens, local caches, or
  machine-specific paths.
- Prefer least-privilege GitHub Actions permissions.

## Anti-patterns (immediate rejection)

1. Skipping relevant skill activation
2. Claiming completion without `make verify` — fresh evidence or it didn't
   happen
3. `unwrap`/`expect` on user-facing paths, or hand-built JSON strings
4. Writing tests that test mocks instead of behavior
5. Fixing symptoms without diagnosing the root cause
6. Type assertions, ignored errors, timing hacks, or broad suppressions as
   workarounds
7. Hand-editing dependency manifests instead of using Cargo tooling
8. Changing exit codes, flags, output formats, or the JSONC config shape
   without treating it as a breaking API change
9. Tagging before the publish dry-runs pass — the tag is the trigger, not
   the test
10. Marking a spec task `completed` without fresh verification evidence;
    tracking progress in `_tasks.md` (status lives only in each `task_NN.md`)
11. Asking for confirmation before running spec tasks — invocation is the
    authorization
12. Reformatting unrelated files

## Agent skills

Skills are installed from the `rust-cli` setup in
[marcioaltoe/skills](https://github.com/marcioaltoe/skills), plus the
project-local `onioncry` skill. Reinstall with:

```bash
curl -fsSL https://raw.githubusercontent.com/marcioaltoe/skills/main/install.sh | bash -s -- rust-cli
bunx skills add marcioaltoe/skills --agent universal --copy -y --skill onioncry
make skills-link
```

### Issue tracker

Spec tasks live as local markdown under `docs/specs/<feature-slug>/` (the
canonical source). Legacy issues remain under `.scratch/` until migrated.
See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the default five-role label vocabulary. See
`docs/agents/triage-labels.md`.

### Domain docs

Domain docs use a single-context layout. See `docs/agents/domain.md`.
