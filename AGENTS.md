# General Agent Instructions

OnionCry is a Rust CLI that names and checks architectural boundaries in source projects. These instructions apply to developing the tool itself.

## High priority

- Use the relevant local skills before changing code, docs, tests, workflows, or agent instructions.
- Agent-created branches must use the `ma/` prefix.
- Prefix shell commands with `rtk` when available. In command chains, prefix each command.
- Never create commits unless the user explicitly asks for a commit in the current turn.
- Always use Conventional Commits for commits and PR titles.
- Before planning, implementing, reviewing, or documenting repository work, read `CONTEXT.md` and relevant `docs/adr/` entries. Use the glossary vocabulary and flag conflicts with ADRs instead of silently overriding them.
- Use `rg` / `rg --files` for local code search. Use Context7 for external library/API docs. Do not use web research tools to search local code.
- Run `make verify` before claiming completion. Treat any format issue, clippy warning, build failure, or test failure as blocking.
- Do not use workarounds in production code or tests. Fix the root cause.
- Do not install dependencies by editing manifest files by hand. Use Cargo commands once the Rust crate exists.
- Do not run destructive git commands such as `git reset`, `git checkout --`, `git restore`, `git clean`, or forced deletion commands unless the user explicitly asks for that operation.

## Agent docs

Read these only when relevant to the task:

- `docs/agents/issue-tracker.md` — local issue/PRD conventions under `.scratch/<feature>/`
- `docs/agents/triage-labels.md` — label mapping for issue triage skills
- `docs/agents/domain.md` — how agents consume `CONTEXT.md` and ADRs
- `CONTEXT.md` — project vocabulary, command concepts, domain rules, and product decisions
- `docs/adr/` — architectural decisions; flag conflicts before overriding them

Use canonical terms from `CONTEXT.md` in command names, help text, issue titles, test names, and user-facing explanations. If the right term is missing, call out the gap instead of inventing new language.

## Skill dispatch

Before editing, identify the task domain and load every matching skill.

- Planning, glossary, or ADR capture: `grill-with-docs`, `grilling`, `domain-modeling`
- PRDs and specs: `to-prd`
- Issue breakdown: `to-issues`
- CLI behavior, flags, stdout/stderr, exit codes, JSON output, dry-run behavior, non-interactive mode, or introspection: `agentic-cli-design`
- Rust command behavior, clap parser code, error handling, output contracts, integration tests, or Cargo workflows: `rust` and `rust-cli`
- Tests, fixtures, golden files, integration tests: `testing-boss` plus the relevant Rust/CLI skill
- Implementation: `coding-guidelines` and `implement`
- Bug fix or failing test: `no-workarounds` and `systematic-debugging`
- Code review before delivery: `review`
- Commits or PR titles: `conventional-commits`
- Completion claim: `evidence-gate`
- OnionCry dogfooding or fixture checks: `onioncry` when running OnionCry against architecture boundaries or validating CLI behavior on sample projects

## CLI behavior

- Design commands for humans and agents. Prefer deterministic output, non-interactive flags, stable exit codes, and machine-readable modes when the workflow needs automation.
- Keep stdout for requested command output. Send diagnostics, progress, and warnings to stderr.
- Treat command names, flags, JSON fields, text output relied on by tests, and exit-code contracts as public API.
- Help text must be concise, truthful, and backed by implemented behavior.
- Errors must name the failed operation and the next useful action when one is known.

## Rust conventions

- Prefer a small, explicit Rust module structure over early abstraction.
- Keep CLI parsing, config loading, path matching, source parsing, graph construction, and diagnostics in separate modules.
- Prefer explicit domain types over stringly typed command plumbing.
- Keep parsing, domain execution, and output rendering separable enough to test without spawning the binary for every case.
- Use `clap` for CLI definitions, `serde_yaml` for config, `globset` for path matching, `oxc_parser` or `swc_ecma_parser` for TS/JS parsing, `petgraph` for graph modeling, and `miette` for terminal diagnostics unless an ADR changes that decision.
- Use `thiserror` or the repo's established error pattern for typed errors.
- Use `serde`/`serde_json` for structured output. Do not hand-build JSON.
- Avoid panics in user-facing CLI paths. Return typed errors and render diagnostics through `miette`.
- Do not add dependencies casually. Each dependency needs a clear job and should be used directly by the crate that imports it.
- Integration tests should assert CLI behavior through the binary when the contract is user-facing. Unit tests should cover pure parsing and domain logic directly.

## Testing rules

- Use Arrange, Act, Assert.
- Test observable behavior, not private implementation details.
- Do not add test-only production hooks, branches, or helper methods.
- Do not test mock behavior instead of system behavior.
- Treat parser fixtures and graph edge cases as first-class tests.
- Flaky tests are blocking failures, not acceptable debt.

## Commands

```bash
make help                 # List available targets
make bootstrap            # Prepare the workspace and fetch Rust deps when scaffolded
make verify               # Full repository verification
make rust-verify          # Run Rust fmt-check, check, clippy, and tests
make fmt                  # Format Rust code
make fmt-check            # Check Rust formatting
make check                # Run cargo check
make lint                 # Run clippy with warnings denied
make test                 # Run Rust tests
make build                # Build the Rust project
make doc                  # Build Rust docs without dependencies
make check-conventions    # Validate Conventional Commit history
make check-pr-title       # Validate a PR title via TITLE="feat(cli): add command"
make update               # Update Cargo.lock when the Rust crate exists
make clean                # Remove Rust build artifacts
make skills-link          # Recreate .claude/skills symlinks from .agents/skills
```

Once the Rust crate is scaffolded, `make verify` includes:

```bash
rtk cargo fmt --all -- --check
rtk cargo clippy --all-targets --all-features -- -D warnings
rtk cargo test --all-features
```

## Coding style

- Rust code must be formatted with `cargo fmt`.
- `cargo clippy --all-targets --all-features -- -D warnings` must pass with zero warnings.
- Prefer clear names over abbreviations.
- Keep comments sparse and useful; explain non-obvious decisions, not syntax.
- Use kebab-case for markdown and workflow files.
- Do not rewrite unrelated files or reformat the whole repo. Keep diffs limited to the task.

## Commit and pull request guidelines

Allowed commit and PR title types:

- `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

Allowed scopes:

- `agents`, `ci`, `cli`, `config`, `deps`, `diagnostics`, `docs`, `graph`, `parser`, `release`, `repo`, `test`

Scopes are optional, but if a scope is used it must come from the allowed list. PR titles must follow the same format because squash-merge titles become release-history entries.

Before opening a PR, run `make verify`. PR bodies should summarize changes, call out risk, and list validation commands run.

## Security and configuration

- Keep secrets in `.env` and never commit them.
- Mirror required environment keys in `.env.example` once environment configuration exists.
- Do not commit generated credentials, tokens, local caches, or machine-specific paths.
- Prefer least-privilege GitHub Actions permissions.

## Anti-patterns (immediate rejection)

1. Skipping relevant skill activation.
2. Claiming completion without `make verify`.
3. Writing tests that test mocks instead of behavior.
4. Fixing symptoms without diagnosing the root cause.
5. Using type assertions, ignored errors, timing hacks, or broad suppressions as workarounds.
6. Hand-editing dependency manifests instead of using Cargo tooling.
7. Using external search for local code.
8. Running destructive git commands without explicit permission.
9. Adding dependencies without a clear reason.
10. Reformatting unrelated files.

## Agent skills

Skills are installed from the `rust-cli` setup in [marcioaltoe/skills](https://github.com/marcioaltoe/skills), plus the project-local `onioncry` skill. Reinstall with:

```bash
curl -fsSL https://raw.githubusercontent.com/marcioaltoe/skills/main/install.sh | bash -s -- rust-cli
bunx skills add marcioaltoe/skills --agent universal --copy -y --skill onioncry
make skills-link
```

### Issue tracker

Issues are tracked as local markdown under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the default five-role label vocabulary. See `docs/agents/triage-labels.md`.

### Domain docs

Domain docs use a single-context layout. See `docs/agents/domain.md`.
