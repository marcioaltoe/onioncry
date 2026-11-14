# Integration and Reach

Status: ready-for-agent

## Summary

OnionCry will meet teams where architecture drift is actually discussed — code review and CI — and become installable by its JavaScript/TypeScript audience without a Rust toolchain. This covers SARIF output, inline suppressions with mandatory reasons, a context graph command, and publishing to npm and crates.io.

## Decisions

### SARIF output

- `onioncry check --format sarif` emits SARIF 2.1.0 built with `serde` types, not hand-assembled JSON.
- Severity maps `error` to SARIF `error` and `warn` to SARIF `warning`. Rule metadata (id, explanation) comes from the rule catalog.
- Baselined violations are emitted with SARIF `suppressions` entries so code-scanning UIs show them as suppressed instead of new.
- A GitHub Action and maintained workflow examples are out of scope for this cycle.

### Inline suppressions

- `// onioncry-disable-next-line <rule>[, <rule>] -- <reason>` suppresses the named rules on the next line. The `-- <reason>` part is mandatory.
- A suppression without a reason, or naming an unknown rule, is reported under `repo/invalid-suppression` (default severity: error) at the comment location.
- A suppression that matches no violation is reported under `repo/unused-suppression` (default severity: warn) so dead suppressions do not accumulate.
- Suppressed violations do not affect the failure threshold and appear in the summary as a separate `suppressed` count; JSON output marks them with `suppressed: true`.
- Only `disable-next-line` is supported. File-level and block-level disables are out of scope.
- Both new rules are architecture-neutral catalog entries and work in either architecture mode.

### Context graph

- `onioncry graph [--format mermaid|json]` renders the dependency graph at the ownership-boundary level; `mermaid` is the default format.
- In Clean Architecture mode, nodes are architectural contexts; in Vertical Slice mode, nodes are slices. Edges are import edges between boundaries, labeled with the public surface segment they enter through when applicable.
- Contextless files are aggregated into a single node so the graph stays readable.
- JSON output is a stable `nodes` / `edges` contract for scripts and agents; Mermaid output is for READMEs and pull requests.
- `graph` is diagnostic and always exits 0 unless the run itself fails (config error, parse failure).

### Distribution

- OnionCry publishes to crates.io (`cargo install onioncry`) and to npm as `onioncry` with prebuilt platform binaries.
- The npm layout is a main `onioncry` package with a small launcher plus platform packages (for example `@onioncry/cli-darwin-arm64`, `@onioncry/cli-linux-x64`, `@onioncry/cli-win32-x64`) wired through `optionalDependencies`.
- Release CI is tag-driven: pushing a version tag builds the platform binaries and publishes crates.io and npm in one workflow. GitHub Releases and Homebrew are not distribution channels in this cycle.
- Version numbers stay in lockstep between Cargo.toml and the npm packages; the release workflow fails when they disagree.
- New CONTEXT.md glossary entries are added for Inline Suppression, Suppression Reason, and Context Graph.

## Issue Breakdown

1. SARIF output format
2. Inline suppressions with mandatory reasons
3. `onioncry graph`
4. Publish to crates.io with tag-driven release CI
5. Publish npm packages with prebuilt binaries
