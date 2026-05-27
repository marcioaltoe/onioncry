# General Agent Instructions

## HIGH PRIORITY (read first — every task)

- Agent-created branches must use the active user's initials as the prefix. For Marcio Altoe, use `ma/`, for example `ma/context-plan-scaffold`.
- Prefix repository shell commands with `rtk` when available.
- Always use Conventional Commits for commits and PR titles.
- You can only finish a task when `make verify` passes. If a check fails, fix it and rerun the full command.
- Always check dependent file APIs before writing tests.
- Never use workarounds in fixes or tests. Diagnose the root cause and test observable behavior.
- Research external libraries before integrating them. Do not use web research for local project code; use local search instead.
- Do not install dependencies by editing manifest files by hand. Use Cargo commands once the Rust crate exists.

## Mandatory Requirements

- Run `make verify` before completing any implementation, docs, or repository-configuration task.
- Conventional Commits are mandatory for every commit:
  - Format: `<type>[optional scope][!]: <description>`
  - Examples: `feat(cli): add check command`, `fix(parser): handle type-only imports`, `ci: validate PR titles`
- PR titles must follow the same format because squash-merge titles become release-history entries.
- Commit descriptions and PR title descriptions should be imperative, lowercase when natural, and concise.
- Do not rewrite unrelated files or reformat the whole repo. Keep diffs limited to the task.

## Commands

```bash
make verify               # Full repository verification
make check-conventions    # Validate Conventional Commit history
make rust-verify          # Run Rust checks when Cargo.toml exists
```

Once the Rust crate is scaffolded, `make verify` includes:

```bash
rtk cargo fmt --all -- --check
rtk cargo clippy --all-targets --all-features -- -D warnings
rtk cargo test --all-features
```

## CRITICAL: Git Commands Restriction

- Never run `git restore`, `git checkout`, `git reset`, `git clean`, `git rm`, or any other git command that discards working directory changes without explicit user permission.
- If a change must be reverted or discarded, ask the user first and wait for permission.
- Use non-destructive commands for inspection: `git status`, `git diff`, `git log`, `git show`.
- Use `--force-with-lease` instead of `--force` when a history rewrite has been explicitly approved.

## Code Search and Discovery

- Use local search first for project code: `rg`, `rg --files`, and targeted file reads.
- Use external documentation tools only for libraries, frameworks, services, or APIs.
- Never use external web search to understand local code.
- When researching external libraries, verify package names, current versions, and official documentation before adding dependencies.

## Rust Project Rules

- Prefer a small, explicit Rust module structure over early abstraction.
- Keep CLI parsing, config loading, path matching, source parsing, graph construction, and diagnostics in separate modules once the crate is scaffolded.
- Use `clap` for CLI definitions, `serde_yaml` for config, `globset` for path matching, `oxc_parser` or `swc_ecma_parser` for TS/JS parsing, `petgraph` for graph modeling, and `miette` for terminal diagnostics unless an ADR changes that decision.
- Treat parser fixtures and graph edge cases as first-class tests.
- Avoid panics in user-facing CLI paths. Return typed errors and render diagnostics through `miette`.
- Do not add dependencies casually. Each dependency needs a clear job and should be used directly by the crate that imports it.

## Testing Rules

- Use Arrange, Act, Assert.
- Test observable behavior, not private implementation details.
- Do not add test-only production hooks, branches, or helper methods.
- Do not test mock behavior instead of system behavior.
- Reset shared state in `before_each` / `after_each` equivalents.
- Flaky tests are blocking failures, not acceptable debt.

## Coding Style and Naming

- Rust code must be formatted with `cargo fmt`.
- `cargo clippy --all-targets --all-features -- -D warnings` must pass with zero warnings.
- Prefer clear names over abbreviations.
- Keep comments sparse and useful; explain non-obvious decisions, not syntax.
- Use kebab-case for markdown and workflow files.

## Commit and Pull Request Guidelines

- Allowed commit and PR title types:
  - `feat`
  - `fix`
  - `docs`
  - `style`
  - `refactor`
  - `perf`
  - `test`
  - `build`
  - `ci`
  - `chore`
  - `revert`
- Allowed scopes:
  - `agents`
  - `ci`
  - `cli`
  - `config`
  - `deps`
  - `diagnostics`
  - `docs`
  - `graph`
  - `parser`
  - `release`
  - `repo`
  - `test`
- Scopes are optional, but if a scope is used it must come from the allowed list.
- Before opening a PR, run `make verify`.
- PRs should include a clear description, linked issue or local ticket when available, and any relevant architecture decisions.

## Security and Configuration

- Keep secrets in `.env` and never commit them.
- Mirror required environment keys in `.env.example` once environment configuration exists.
- Do not commit generated credentials, tokens, local caches, or machine-specific paths.
- Prefer least-privilege GitHub Actions permissions.

## Agent Skill Dispatch Protocol

Every agent must identify the task domain before writing code:

- Specs, PRDs, or planning: use `to-prd`, `to-issues`, or `grill-with-docs` as appropriate.
- Test-first implementation: use `tdd`.
- Bugs, failures, or regressions: use `diagnose`.
- Architecture review or refactoring analysis: use `improve-codebase-architecture`.
- README or public docs: use `crafting-effective-readmes`.
- Broad codebase orientation: use `zoom-out`.

Before claiming completion:

1. Run `make verify`.
2. Read the output.
3. Fix all failures.
4. Rerun `make verify`.

## Anti-Patterns (immediate rejection)

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

### Issue tracker

Issues are tracked as local markdown under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the default five-role label vocabulary. See `docs/agents/triage-labels.md`.

### Domain docs

Domain docs use a single-context layout. See `docs/agents/domain.md`.
