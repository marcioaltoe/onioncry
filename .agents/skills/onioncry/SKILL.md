---
name: onioncry
description: Run and interpret the OnionCry Rust CLI for architecture-boundary checks in JavaScript and TypeScript repositories. Use when the user asks to check architecture boundaries, generate or review .onioncryrc.jsonc, run onioncry check or explain, adopt a violation baseline, inspect llm-mode reports, render boundary graphs, add repo/package scripts, or automate OnionCry in agent workflows. Do NOT use for generic Rust implementation work.
metadata:
  category: architecture
  tags: [architecture, cli, rust, javascript, typescript, qa, code-review]
  version: 0.1.0
  author: Marcio Altoé
  source: https://github.com/marcioaltoe/skills
---

# OnionCry

Use OnionCry as an architecture-boundary checker for JavaScript and TypeScript repositories. Treat it like a deterministic verification tool: run it from the target repository, read its output, and make the smallest architecture or configuration change that resolves the reported boundary issue.

This skill documents the published v0.1.0 command surface. The copy in `.agents/skills/onioncry/` of the OnionCry repo is canonical and must be updated whenever CLI behavior changes.

## Availability

Pick whichever the target project already uses:

```bash
bunx onioncry --help        # ad-hoc, no install (npx works too)
bun add -d onioncry         # per-project devDependency
cargo install onioncry      # global binary from crates.io
```

The npm package installs only the platform binary that matches the machine via `optionalDependencies`. There is no postinstall script.

## Commands

Every subcommand accepts `--help`.

| Command | Purpose | Key flags |
| --- | --- | --- |
| `check` | Check the configured file universe against architecture rules | `--format pretty\|json\|sarif`, `--fail-on error\|warning`, `--llm-mode`, `--tips`, `--files <PATH>...`, `--baseline <PATH>`, `--write-baseline`, `--no-baseline`, `--config` |
| `explain <FILE>` | Explain one file's classification, imports, and violations | `--format pretty\|json`, `--tips`, `--config` |
| `graph` | Render the boundary dependency graph | `--format mermaid\|json`, `--config` |
| `rules` | List built-in rules | `--format pretty\|json` |
| `schema` | Print the `.onioncryrc` JSON Schema | `--write <PATH>` |
| `init` | Create a conservative `.onioncryrc.jsonc` template | `--force`, `--from-tsconfig [<PATH>]` |

`--files` scopes the report to the listed files; analysis stays whole-project.

## Exit codes

The exit-code contract is public API:

- `0` — clean at the configured threshold (`--fail-on` defaults to `error`, so warnings alone still exit 0)
- `1` — violations at or above the `--fail-on` threshold
- `2` — operational error (missing config, bad usage); fix it before interpreting architecture diagnostics

## Workflow

1. Confirm the CLI is available: `onioncry --help` (or `bunx onioncry --help`).
2. If the target repository has no config, create one from the repository root with `onioncry init`. Review `.onioncryrc.jsonc` before running checks. Use `--force` only when the user explicitly wants to overwrite an existing config. In TypeScript projects with path aliases, `onioncry init --from-tsconfig` generates the aliases block from `compilerOptions.paths` for review.
3. Run `onioncry check`.
4. For agent-readable output, prefer `onioncry check --llm-mode` or `onioncry check --format json`. Use `--format sarif` for code-scanning integrations.
5. When diagnosing one file, use `onioncry explain <file>` (add `--format json` or `--tips`). To see the boundary topology, use `onioncry graph`.
6. After changing architecture, imports, paths, or config, rerun the same OnionCry command that exposed the issue.

## The llm-mode report

`check --llm-mode` prints a summary header (`status`, `filesChecked`, `problemCount`, `errorCount`, `warningCount`, `groupCount`) followed by diagnostic groups, each with `rule`, `severity`, `message`, `why`, `tip`, and `locations`. The report ends with a versioned footer:

```text
onioncry-llm-report v1 revision: <hash>
```

Treat the footer as the completeness marker: if it is missing, the report was truncated. The footer version is a public contract.

## Violation baseline (adopting OnionCry in a legacy repo)

The baseline grandfathers existing debt so new violations still fail while known ones are tracked in a reviewable artifact:

```bash
onioncry check --write-baseline   # record current violations in .onioncry-baseline.json
onioncry check                    # consumes the baseline by default; only new debt fails
onioncry check --no-baseline      # full, unfiltered architecture result
onioncry check --baseline <PATH>  # use a baseline at a custom path
```

Baseline entries are fingerprinted by `rule + file + target` (no line/column, so routine edits do not invalidate entries). Stale entries — baseline items matching no current violation — emit a non-blocking stderr warning suggesting a `--write-baseline` rerun. Commit the baseline file and shrink it over time (ratchet), never grow it to silence new violations.

## Configuration

- OnionCry discovers `.onioncryrc.jsonc` first, then `.onioncryrc.json`. JSONC comments are a public contract.
- Keep alias resolution explicit in config. `init --from-tsconfig` is the supported way to derive aliases from a tsconfig; review the generated block before committing.
- Use package-local configs for monorepos when packages have different boundary rules.
- In Bun/Turbo workspaces, add package scripts that call OnionCry, then fan out from the root with the workspace runner:

```json
{
  "scripts": {
    "onioncry": "bunx onioncry check --llm-mode"
  }
}
```

## Fix strategy

- Fix the code boundary when the diagnostic points to a real architecture violation.
- Fix `.onioncryrc.jsonc` only when the rule scope, alias, or package boundary is wrong.
- Keep Oxlint, TypeScript, Biome, and generic formatting issues out of OnionCry fixes. OnionCry owns architecture-boundary checks, not every JavaScript or TypeScript rule.
- Prefer moving code toward the intended dependency direction over suppressing the rule.
- Prefer fixing violations over baselining them; `--write-baseline` is for adoption, not for burying new debt.

## Gotchas

- `explain` on a path outside the configured universe — including a nonexistent file — reports `layer: unclassified` with exit 0. Verify the path before trusting an "unclassified" answer.
- Exit 0 does not mean zero findings: warnings pass under the default `--fail-on error`. Read `warningCount` in the report, or run with `--fail-on warning` when warnings must block.

## Development checks

When modifying OnionCry itself, run the project verify command from the OnionCry repository:

```bash
make verify
```

For focused Rust work, use the narrower checks from that repo:

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
