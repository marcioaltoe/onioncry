# Add inline suppressions with mandatory reasons

Status: done

## Parent

../PRD.md

## What to build

Support `// onioncry-disable-next-line <rule>[, <rule>] -- <reason>` comments that suppress the named rules on the next line, with the reason mandatory so every exception is justified in the code.

## Acceptance criteria

- [x] A valid suppression comment suppresses violations of the named rules on the next line; suppressed violations do not affect the failure threshold.
- [x] The `-- <reason>` part is mandatory; a suppression without a non-empty reason is reported under `repo/invalid-suppression` (default severity: error) at the comment location.
- [x] Naming an unknown rule is reported under `repo/invalid-suppression` with the unknown name and the closest known rule names.
- [x] A suppression matching no violation is reported under `repo/unused-suppression` (default severity: warn).
- [x] Multiple rules per comment are supported with comma separation; legacy rule aliases resolve to canonical names.
- [x] Both new rules are architecture-neutral catalog entries with explanations, configurable severities, and override support; `onioncry rules` lists them.
- [x] The summary shows a separate `suppressed` count; JSON output marks suppressed violations with `suppressed: true`; `--llm-mode` groups them separately.
- [x] Suppressions work in `.ts`, `.tsx`, `.js`, `.jsx`, `.mts`, `.cts`, `.mjs`, and `.cjs` sources using comment spans from the existing parser.
- [x] Only `disable-next-line` exists; file-level or block-level disables are not implemented.
- [x] CONTEXT.md gains glossary entries for Inline Suppression and Suppression Reason.
- [x] CLI integration tests cover valid suppression, missing reason, unknown rule, unused suppression, multi-rule comments, and severity overrides.
- [x] `make verify` passes.

## Blocked by

cli-introspection/01-add-rules-command (new rules must appear in the rules listing)

## Comments

- 2026-07-04: Added `// onioncry-disable-next-line <rule>[, <rule>] -- <reason>` parsing from OXC line comment spans, canonical rule/legacy alias resolution, mandatory reason validation, unknown-rule closest-name diagnostics, unused suppression diagnostics, suppressed report state, SARIF in-source suppressions, README syntax docs, and CONTEXT.md glossary entries. Verification: `rtk cargo test --test cli_suppressions` passed with 7 tests; `rtk cargo test --test cli_check baselined` passed with 3 tests; `rtk cargo test --test cli_sarif` passed with 3 tests; `rtk make verify` passed with clippy clean and 110 tests. The exact commit SHA is reported by the implementation loop after the slice commit is created.
