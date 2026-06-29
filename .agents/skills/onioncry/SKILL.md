---
name: onioncry
description: Run and interpret the OnionCry Rust CLI for architecture-boundary checks in JavaScript and TypeScript repositories. Use when the user asks to check architecture boundaries, generate or review .onioncryrc.jsonc, run onioncry check or explain, inspect llm-mode reports, add repo/package scripts, or automate OnionCry in agent workflows. Do NOT use for generic Rust implementation work.
metadata:
  category: architecture
  tags: [architecture, cli, rust, javascript, typescript, qa, code-review]
  version: 0.1.0
  author: Marcio Altoé
  source: https://github.com/marcioaltoe/skills
---

# OnionCry

Use OnionCry as an architecture-boundary checker for JavaScript and TypeScript repositories. Treat it like a deterministic verification tool: run it from the target repository, read its output, and make the smallest architecture or configuration change that resolves the reported boundary issue.

## Workflow

1. Confirm the CLI is available:

   ```bash
   onioncry --help
   ```

2. If the target repository has no config, create one from the repository root:

   ```bash
   onioncry init
   ```

   Review `.onioncryrc.jsonc` before running checks. Use `onioncry init --force` only when the user explicitly wants to overwrite an existing config.

3. Run the normal check:

   ```bash
   onioncry check
   ```

4. For agent-readable output, prefer one of:

   ```bash
   onioncry check --llm-mode
   onioncry check --format json
   ```

5. When diagnosing one file, use:

   ```bash
   onioncry explain <file>
   onioncry explain <file> --format json
   onioncry explain <file> --tips
   ```

6. After changing architecture, imports, paths, or config, rerun the same OnionCry command that exposed the issue.

## Configuration

- OnionCry discovers `.onioncryrc.jsonc` first, then `.onioncryrc.json`.
- Keep alias resolution explicit in config. Do not infer path aliases from unrelated build tooling unless OnionCry documents that behavior.
- Use package-local configs for monorepos when packages have different boundary rules.
- In Bun/Turbo workspaces, add package scripts that call `onioncry check --llm-mode`, then fan out from the root with the workspace runner.

Example package script:

```json
{
  "scripts": {
    "onioncry": "PATH=\"$HOME/.cargo/bin:$PATH\" onioncry check --llm-mode"
  }
}
```

## Output and exit codes

- Pretty output is for humans.
- `--llm-mode` groups diagnostics for agents.
- `--format json` is the preferred contract when downstream automation must parse results.
- A failing architecture report exits non-zero. Treat that as a real verification failure, not a flaky lint warning.
- Usage or configuration errors must be fixed before interpreting architecture diagnostics.

## Fix strategy

- Fix the code boundary when the diagnostic points to a real architecture violation.
- Fix `.onioncryrc.jsonc` only when the rule scope, alias, or package boundary is wrong.
- Keep Oxlint, TypeScript, Biome, and generic formatting issues out of OnionCry fixes. OnionCry owns architecture-boundary checks, not every JavaScript or TypeScript rule.
- Prefer moving code toward the intended dependency direction over suppressing the rule.

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
