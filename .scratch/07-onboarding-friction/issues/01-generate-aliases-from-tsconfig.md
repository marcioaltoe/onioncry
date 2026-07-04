# Add `onioncry init --from-tsconfig`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Let `onioncry init` generate the `aliases` block of the configuration template from a TypeScript config's `compilerOptions.paths` and `baseUrl`. This is explicit generation for team review at init time; runtime alias resolution still reads only the OnionCry configuration. Capture the decision as ADR `docs/adr/0011-generate-aliases-from-tsconfig-at-init.md`.

## Acceptance criteria

- [ ] `onioncry init --from-tsconfig` reads `tsconfig.json` from the project root; `--from-tsconfig <path>` reads the given file.
- [ ] Wildcard mappings translate to prefix aliases: `"@/*": ["./src/*"]` with the tsconfig `baseUrl` becomes `"@/": "src/"`, normalized relative to the OnionCry project root.
- [ ] Non-wildcard keys, multi-target mappings, and targets outside the project root are skipped and listed in a comment inside the generated template so the team resolves them manually.
- [ ] tsconfig files are parsed with the same tolerant JSONC parsing used for `.onioncryrc.jsonc`; `extends` is not followed, and its presence is noted in the generated comment.
- [ ] A missing or unparseable tsconfig is an error (exit 2) that names the path and the next useful action; no partial config file is left behind.
- [ ] Without the flag, `init` output is byte-identical to today's template.
- [ ] `--force` semantics are unchanged and compose with the new flag.
- [ ] ADR 0011 records that generation happens at init for review and runtime inference remains out of scope.
- [ ] CLI integration tests cover default path, explicit path, skipped-entry comments, error handling, and the unchanged default template.
- [ ] README documents the flag.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately
