# Add inline suppressions with mandatory reasons

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Support `// onioncry-disable-next-line <rule>[, <rule>] -- <reason>` comments that suppress the named rules on the next line, with the reason mandatory so every exception is justified in the code.

## Acceptance criteria

- [ ] A valid suppression comment suppresses violations of the named rules on the next line; suppressed violations do not affect the failure threshold.
- [ ] The `-- <reason>` part is mandatory; a suppression without a non-empty reason is reported under `repo/invalid-suppression` (default severity: error) at the comment location.
- [ ] Naming an unknown rule is reported under `repo/invalid-suppression` with the unknown name and the closest known rule names.
- [ ] A suppression matching no violation is reported under `repo/unused-suppression` (default severity: warn).
- [ ] Multiple rules per comment are supported with comma separation; legacy rule aliases resolve to canonical names.
- [ ] Both new rules are architecture-neutral catalog entries with explanations, configurable severities, and override support; `onioncry rules` lists them.
- [ ] The summary shows a separate `suppressed` count; JSON output marks suppressed violations with `suppressed: true`; `--llm-mode` groups them separately.
- [ ] Suppressions work in `.ts`, `.tsx`, `.js`, `.jsx`, `.mts`, `.cts`, `.mjs`, and `.cjs` sources using comment spans from the existing parser.
- [ ] Only `disable-next-line` exists; file-level or block-level disables are not implemented.
- [ ] CONTEXT.md gains glossary entries for Inline Suppression and Suppression Reason.
- [ ] CLI integration tests cover valid suppression, missing reason, unknown rule, unused suppression, multi-rule comments, and severity overrides.
- [ ] `make verify` passes.

## Blocked by

cli-introspection/01-add-rules-command (new rules must appear in the rules listing)
