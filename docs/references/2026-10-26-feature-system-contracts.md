# Feature-system contract evidence (2026-10-26)

## Existing policy contract

ADR-0003 states that matching overrides replace the whole rule value for a file and later matches win. At onioncry@db73399, src/rules/feature_system/mod.rs instead selects the first enabled setting across files for all five collectors. RulePolicy::effective_rule already resolves file-specific values correctly, and RuleSetting supports equality. All diagnostic owners are available as Violation.file; import findings belong to edge.source. Whole-project policy analysis must retain the complete files and edges.

## Outside source: ECMAScript named exports

Source read on 2026-10-26: https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#sec-exports-static-semantics-exportentries . The normative ExportEntries operation represents exports as records linking the exported name to a local binding or a requested module/imported name. The specification names these fields [[ExportName]], [[LocalName]], [[ModuleRequest]] and [[ImportName]]. Named reexport syntax therefore denotes a binding relationship, not a new class declaration in the exporting module. This supports following the exact requested named binding through the existing local module graph; it does not require executing a module or implementing a general type checker.

QA outside-evidence criterion: a local named Error subclass exported through a renamed named reexport maps the public name to that class and produces no adapter Error-contract violation; renaming the exported target to an ordinary value remains a violation. Inspect real CLI JSON and the alias mapping, rather than accepting any text containing an Error subclass.

## Parser and repository boundary

ADR-0004 selects Oxc. Existing src/imports.rs scans static imports/reexports with Oxc and returns typed parsing errors. ImportEdge resolves local module paths but does not retain binding names. Add only a private AST export summary; preserve the public graph shape. Existing source_exports_error_class is a substring heuristic and is not proof of the exported declaration.

Context7 source consulted for Rust API orientation: /oxc-project/oxc. The repository's pinned Oxc crate sources and existing parser adapter determine exact compatible APIs; no dependency migration is part of the correction.

## Consultation limits

Secondbrain index read: secondbrain/wiki/index.md. A qmd query for OnionCry per-file overrides and Error reexports was initiated; no returned page is claimed as validation here. Exa MCP is unavailable in this session's exposed tools. Independent local source exploration confirmed the two defects and the absence of a reusable export-binding resolver. No Secondbrain file was written.

## Entry verification

make verify passed on db73399 before this correction: formatting, Cargo check, clippy with warnings denied, Conventional Commit checks and 128 tests across 17 suites. New regression tests must be observed failing before each implementation.

Secondbrain follow-up: qmd returned the existing OnionCry ADR mirrors. Read secondbrain/projects/onioncry/mirror/docs/adr/0003-use-linter-style-configuration.md and secondbrain/projects/onioncry/mirror/docs/adr/0004-use-oxc-parser-for-javascript-and-typescript.md; they confirmed last-match per-file replacement and the Oxc choice, matching the current local originals. No mirror code or configuration was copied.
