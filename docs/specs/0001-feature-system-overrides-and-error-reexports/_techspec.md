---
spec: 0001-feature-system-overrides-and-error-reexports
prd: _prd.md
created: 2026-10-26
---

# Feature-system policy and Error export corrections — Technical Spec

## Executive Summary

Repair policy selection at the collector boundary and prove the exported Error binding at the adapter boundary. Reuse each policy's existing whole-project analysis; group equal RuleSetting values rather than partitioning the file graph or rescanning once per file. For Error exports, accept only syntax and local resolved bindings the existing analysis can prove; unknown and cyclic targets remain violations.

## Project Constraints

- Identifier strategy: not applicable — no persistent identifiers are introduced. Source: `docs/agents/domain.md`.
- Authentication and HTTP: not applicable — analysis remains a local, read-only CLI operation without authentication or HTTP API changes. Source: `docs/agents/cli.md`.
- Active ADR obligations: applicable — preserve ADR-0003 last-matching per-file rule replacement and ADR-0004 Oxc parsing; do not replace the parser or change configuration shape. Source: `docs/agents/domain.md`; `docs/adr/0003-use-linter-style-configuration.md`; `docs/adr/0004-use-oxc-parser-for-javascript-and-typescript.md`.
- Tooling authority: applicable — express maintainer authorization on 2026-10-26 through the question tool to correct the two confirmed defects, test, review, commit, push, open PRs, merge with approved CI and publish a stable release for the consumer. Source: `docs/agents/agent-instructions.md`; Spec-contained authorization record: `docs/specs/0001-feature-system-overrides-and-error-reexports/_authorization.md`; bounded files: `src/rules/feature_system/mod.rs`, `src/rules/feature_system/helpers.rs`, `src/rules/feature_system/adapter_contract.rs`, `src/rules/feature_system/error_exports.rs`, `tests/feature_system.rs`, `.agents/skills/onioncry/SKILL.md`, `Cargo.toml`, `Cargo.lock`, `npm/package.json`, `npm/platforms/darwin-arm64/package.json`, `npm/platforms/darwin-x64/package.json`, `npm/platforms/linux-arm64/package.json`, `npm/platforms/linux-x64/package.json`, `npm/platforms/win32-x64/package.json`. Regenerate Cargo.lock through Cargo; follow the public release plan and existing release choreography for all version pins. No dependency replacement or checker-configuration relaxation is proposed.

## System Architecture

All five collectors in src/rules/feature_system/mod.rs currently select the first non-Off setting. RulePolicy::effective_rule already implements last-match replacement and RuleSetting supports equality. Evaluate each distinct enabled setting against the complete file universe and edge graph, retaining diagnostics only where Violation.file has that effective setting. A file with an Off setting owns no diagnostic for that rule.

Keep existing representative ownership for aggregate layout, adapter and query-file findings. Do not repartition files/edges, fabricate absent adapters or query files, or move aggregate findings to an arbitrary enabled sibling. Test the representative-file exception explicitly.

The adapter contract replaces only source_exports_error_class. A private error_exports module may summarize Oxc declarations and exports and resolve the requested binding using existing ImportEdge resolution. Keep public ImportEdge and all serialized schemas unchanged.

## Implementation Design

### Interfaces

Preserve public library and CLI signatures. Private helpers may group settings and resolve a named Error export from a file, an analyzed-file set and local edges. The resolver tracks (file, binding) visits and caches parsed summaries within one analysis; it never reads a target outside the analyzed universe.

Support direct exported named classes extending Error, local class plus named export, named reexports with renamed bindings or multiple hops, and an imported binding subsequently exported locally. Track type-only syntax so it never proves a runtime class. Match the exact declaration behind the requested export; an unrelated Error subclass does not validate an ordinary exported class. Reject missing, unresolved, external, excluded, cyclic, ordinary-value and non-Error bindings conservatively. Wildcard/default exports and indirect inheritance are not newly promised by this patch.

### Data Models

None. Any export summary is private transient analysis state, not a public graph or report format.

### API Contracts

1. API Contract: check and explain preserve their existing commands, streams, schemas, rule names and 0/1/2 exit-code contracts; corrected diagnostics follow the actual file policy and named Error export. --fail-on warning remains strict, and production import boundaries remain enforced.

### Surface Transcripts

None. No new literal CLI output is promised; existing diagnostic message and report layouts remain unchanged while their conditions are corrected.

## Coverage Map

- Goal 1; Core Feature 1; Success Metric 1 → distinct-setting collector evaluation and owner-based diagnostic filtering.
- Goal 2; Core Feature 2; Success Metric 2 → private Oxc export summary and bounded named-binding resolution.
- Goal 3; Core Feature 3; Success Metric 3 → terminal CLI QA, configured review, current-head CI, and the existing release process after graph completion.
Deviations from the PRD: None.

## Absence Evidence

1. Private named-export resolver — command: `rg -n 'source_exports_error_class|exported_name|export_entries|resolve.*export' src/rules/feature_system src/imports.rs`; result: the helper is a source-substring check, and ImportEdge lacks binding names. Existing import scanning already uses Oxc and resolves module paths; reuse those boundaries without a new public graph model.

## ADR Alternatives

- ADR-0003: keep the existing per-file, last-override-wins configuration contract; do not broaden file selection or merge option objects.
- ADR-0004: keep Oxc AST parsing; do not introduce a different parser or string heuristics as an alternative proof.

## Integration Points

Analysis remains local and read-only. Registry publication occurs only through docs/release.md after approved review, required checks, synchronized versions and dry-runs. No live external API or customer fixture is part of implementation tests.

## Testing Approach

Extend tests/feature_system.rs through its existing TempDir/config/run_json_check helpers and the real binary. Task 01 covers all five families: off/warn/error, last override wins, option replacement, an earlier unrelated file, stable aggregate owners and complete cross-file facts. Co-locate an exempt test/export hook and a forbidden production component in one fixture and assert exact file, rule, severity, counts and strict exit behavior.

Task 02 covers direct/local/renamed/multiline/multihop/import-then-export Error classes and rejects values, non-Error classes plus decoys, comments, strings, type-only syntax, missing/unresolved/excluded targets and cycles. Existing adapter import-boundary findings must survive a valid Error reexport. Pure private helpers may receive in-module tests; public behavior is always tested through the binary.

QA has a named outside-evidence row: compare the named-binding/alias fixture with ECMAScript ExportEntries semantics quoted and summarized in docs/references/2026-10-26-feature-system-contracts.md, then inspect the real check/explain reports. Registry/publication evidence is declared unreachable before the release and remains owned by the release runbook.

## Build Order

1. Correct distinct per-file policy evaluation for all five collectors and add real CLI regression coverage.
2. Correct named Error export proof and update the canonical usage skill (depends on: 1).
3. Run the authored terminal CLI QA (depends on: 2).
4. After the graph, review and merge the correction, then run the public release plan and existing release choreography. Archive only after release, as required by the repository-owned instruction.

## Risks & Considerations

Do not group by a reduced file set: that breaks structural checks. Do not scan once per file when settings are equal. Keep aggregate owners stable, avoid unbounded cyclic export traversal, and never read excluded files. Do not accept unrelated class text or execute analyzed modules. No generic type inference, baseline, rule suppression or new dependency is needed.

## Glossary

None.

## Research

- Local source inspection and independent read-only exploration at db73399 established both root causes and the absence of export-binding data in ImportEdge.
- Secondbrain index consulted: /Users/marcio/dev/secondbrain/wiki/index.md; qmd query for OnionCry overrides/reexports was attempted. Results, if any, are reference evidence rather than authority.
- Exa MCP is unavailable in the exposed tool set; no Exa validation is claimed.
- Context7 Oxc documentation and the ECMAScript module specification are recorded in the committed reference. The installed Oxc source version remains authoritative for exact Rust APIs.

## Decisions

- 2026-10-26: preserve diagnostic ownership and full graph; deduplicate equal policy settings. Accept only proven named runtime Error exports, with unsupported/unresolved forms remaining violations.

Secondbrain follow-up: qmd returned the existing OnionCry ADR mirrors. Read /Users/marcio/dev/secondbrain/projects/onioncry/mirror/docs/adr/0003-use-linter-style-configuration.md and /Users/marcio/dev/secondbrain/projects/onioncry/mirror/docs/adr/0004-use-oxc-parser-for-javascript-and-typescript.md; they confirmed last-match per-file replacement and the Oxc choice, matching the current local originals. No mirror code or configuration was copied.
