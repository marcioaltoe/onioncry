# OnionCry

OnionCry is a Rust CLI for checking architectural boundaries in JavaScript and
TypeScript projects. It names architecture drift with linter-style diagnostics so
teams can see when layers, bounded contexts, public surfaces, or core package
policies are being crossed.

The project is currently in alpha. The CLI is usable locally, but distribution is
not published through Homebrew or npm yet. Maintainers publish crates.io
releases through the tag-driven process in [docs/release.md](docs/release.md).

## Install During Alpha

For now, build and install from this checkout:

```bash
cd ~/dev/onioncry
make install
```

`make install` builds the Rust project and installs the `onioncry` binary into
Cargo's bin directory, usually `~/.cargo/bin/onioncry`.

After pulling new OnionCry changes, run it again:

```bash
cd ~/dev/onioncry
git pull
make install
```

Make sure Cargo's bin directory is on your shell path:

```bash
echo $PATH
which onioncry
onioncry --help
```

After the first crates.io release, users can install with `cargo install
onioncry`. Until then, `make install` is the supported local testing path.

## Quick Start

From the project you want to check:

```bash
cd ~/dev/my-project
onioncry init
```

Review `.onioncryrc.jsonc`, then run:

```bash
onioncry check
```

For automation or agent workflows:

```bash
onioncry check --llm-mode
onioncry check --format json
onioncry check --format sarif
onioncry rules --format json
```

To inspect one file:

```bash
onioncry explain packages/backend/src/application/use-cases/example.ts
```

For monorepos, keep separate configs in the workspaces that have different
aliases or architecture rules:

```text
packages/backend/.onioncryrc.json
packages/frontend/.onioncryrc.json
```

Then add an `onioncry` script in each workspace package and fan out from the
root with Turbo:

```json
{
  "scripts": {
    "onioncry": "PATH=\"$HOME/.cargo/bin:$PATH\" onioncry check --llm-mode"
  }
}
```

```json
{
  "scripts": {
    "onioncry": "turbo run onioncry"
  }
}
```

## Commands

```bash
onioncry init
```

Creates a conservative `.onioncryrc.jsonc` template. Use `--force` to overwrite
an existing config.

```bash
onioncry check
```

Checks all configured files and prints grouped, clickable diagnostics. By
default, warnings are counted but only errors fail the run.

Useful options:

```bash
onioncry check --config path/to/.onioncryrc.jsonc
onioncry check --format json
onioncry check --format sarif
onioncry check --fail-on warning
onioncry check --tips
onioncry check --llm-mode
onioncry check --write-baseline
onioncry check --baseline path/to/.onioncry-baseline.json
onioncry check --no-baseline
```

### Violation Baselines

Use a violation baseline when a repository needs to adopt stricter rules before
all existing architecture drift is fixed. The baseline keeps known debt visible
without weakening rule defaults or hiding new violations.

Adoption workflow:

1. Enable the strict rules you want in `.onioncryrc.jsonc`.
2. Run `onioncry check --write-baseline` to write the current violations to
   `.onioncry-baseline.json` next to the resolved config file.
3. Commit `.onioncry-baseline.json` with the config change so the grandfathered
   debt is explicit and reviewable.
4. Run `onioncry check` in automation. Baselined violations stay visible, but
   only new violations affect the failure threshold.
5. Fix baselined violations over time, then rerun
   `onioncry check --write-baseline` to shrink the baseline.

`--baseline <path>` reads or writes a custom baseline path. `--no-baseline`
disables baseline consumption for one run, which is useful when auditing all
current violations.

### SARIF Output

`onioncry check --format sarif` emits SARIF 2.1.0 for code review and code
scanning integrations. The SARIF result set includes active and baselined
violations; baselined violations are marked with external suppressions so
review tools can keep them visible without treating them as new findings.

```bash
onioncry check --format sarif > onioncry.sarif
sarif="$(gzip -c onioncry.sarif | base64 | tr -d '\n')"
gh api \
  -X POST \
  "repos/OWNER/REPO/code-scanning/sarifs" \
  -f commit_sha="$(git rev-parse HEAD)" \
  -f ref="$(git symbolic-ref -q HEAD)" \
  -f sarif="$sarif"
```

OnionCry does not ship a maintained GitHub Action. In CI, generate the SARIF
file and upload it with the integration surface your repository already uses.

### Inline Suppressions

Use an inline suppression for a narrow, reviewed source-level exception:

```ts
// onioncry-disable-next-line cleanarch/no-layer-leak -- legacy adapter migration
import { repo } from "../infra/repo";
```

The reason after `--` is mandatory. Suppressions apply only to named rules on
the next line, stay visible in reports, and do not affect the failure threshold.
Unused suppressions are reported as `repo/unused-suppression`; malformed
comments are reported as `repo/invalid-suppression`.

```bash
onioncry graph
```

Prints a Mermaid ownership graph: Clean Architecture mode groups imports by
architectural context, while Vertical Slice mode groups imports by slice.

Useful options:

```bash
onioncry graph --format json
onioncry graph --config path/to/.onioncryrc.jsonc
```

The JSON structure is public API for automation:

```json
{
  "nodes": [{ "id": "sales", "label": "sales", "kind": "context" }],
  "edges": [{ "from": "sales", "to": "billing", "via": "contracts", "importCount": 2 }]
}
```

`via` is `null` when the target boundary is not entered through a configured
public surface segment.

```bash
onioncry explain <file>
```

Shows how one file is classified, which imports OnionCry resolved, and which
violations apply to that file.

Useful options:

```bash
onioncry explain <file> --format json
onioncry explain <file> --tips
```

```bash
onioncry rules
```

Lists the built-in rule catalog without reading a project config. The output
includes each rule name, default severity, architecture family, explanation, and
legacy aliases where they exist.

Useful options:

```bash
onioncry rules --format json
```

```bash
onioncry schema
```

Prints the JSON Schema for `.onioncryrc.jsonc` so editors and automation can
discover the supported configuration shape.

Useful options:

```bash
onioncry schema --write docs/schema/onioncryrc.schema.json
```

## Configuration

OnionCry auto-discovers `.onioncryrc.jsonc` first, then `.onioncryrc.json`.
JSONC remains the default because the configuration works like a linter config,
with comments, rules, severities, options, and overrides.

The `onioncry init` template includes a `$schema` field pointing at the schema
published from `main`, so editors that understand JSON Schema can validate and
autocomplete the config. Regenerate the committed schema after config type
changes with:

```bash
onioncry schema --write docs/schema/onioncryrc.schema.json
```

Minimal shape:

```jsonc
{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["packages/backend/src/**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs}"],
    "exclude": ["node_modules/**", "dist/**", "build/**", "coverage/**", "**/__tests__/**"]
  },
  "architecture": {
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "contexts",
      "layerPathAliases": {
        "infra": ["infra", "infrastructure"]
      },
      "artifactFolders": {
        "domain": ["entities", "value-objects", "ports"],
        "application": ["use-cases", "ports"],
        "infra": ["repositories", "adapters", "controllers", "database", "workflows", "bootstrap"]
      },
      "artifactSuffixes": {
        "repository": [".repository.ts", "-repository.ts", "-catalog.ts", ".writer.ts", "-writer.ts", "-writers.ts"],
        "service": [".service.ts", "-service.ts"],
        "useCase": [".use-case.ts", "-use-case.ts"],
        "entity": [".entity.ts", "-entity.ts"],
        "valueObject": [".value-object.ts", "-value-object.ts"],
        "adapter": [".adapter.ts", "-adapter.ts", ".gateway.ts", "-gateway.ts", "/client.ts", ".client.ts", "-client.ts", "/handler.ts", ".mapper.ts", "-mapper.ts", "-mappers.ts", ".parser.ts", "-parser.ts", ".provider.ts", "-provider.ts", ".request.ts", "-request.ts", "-requests.ts", ".schema.ts", "-schema.ts", "-schemas.ts", "-normalization.ts", "-resilience.ts", "-composition.ts", "-scenario.ts", "-scenarios.ts", "-snapshot.ts", "-snapshots.ts"],
        "handler": [".handler.ts", "-handler.ts"],
        "port": [".port.ts", "-port.ts", "-ports.ts"]
      },
      "groupedArtifactFolders": [
        "use-cases",
        "entities",
        "value-objects",
        "ports",
        "repositories",
        "adapters",
        "controllers",
        "database",
        "workflows",
        "bootstrap"
      ]
    }
  },
  "aliases": {
    "@/": "packages/backend/src/"
  },
  "layers": {
    "domain": {
      "patterns": ["packages/backend/src/domain/**"],
      "mayImport": ["domain", "shared"]
    },
    "application": {
      "patterns": ["packages/backend/src/application/**"],
      "mayImport": ["application", "domain", "shared"]
    },
    "infra": {
      "patterns": ["packages/backend/src/infra/**"],
      "mayImport": ["infra", "application", "domain", "shared"]
    },
    "shared": {
      "patterns": ["packages/backend/src/shared/**"],
      "mayImport": ["shared"]
    }
  },
  "contextRules": {
    "default": {
      "allowSameContext": true,
      "allowCrossContext": ["contracts", "events", "ports", "shared"]
    }
  },
  "rules": {
    "cleanarch/no-layer-leak": "error",
    "cleanarch/no-cross-context-internal-import": "error",
    "cleanarch/no-forbidden-imports": [
      "error",
      {
        "layers": [
          {
            "fromLayer": "domain",
            "severity": "error",
            "allow": ["uuid"]
          },
          {
            "fromLayer": "application",
            "severity": "warn",
            "allow": []
          },
          {
            "fromLayer": "infra",
            "severity": "off",
            "allow": []
          }
        ]
      }
    ],
    "cleanarch/no-framework-in-core": "warn",
    "cleanarch/no-outer-data-format-in-core": "warn",
    "cleanarch/no-public-surface-internal-reexport": "warn",
    "cleanarch/no-context-cycle": "warn",
    "cleanarch/no-unowned-schema-import": "warn",
    "cleanarch/artifact-placement": "warn",
    "solid/no-concrete-dependency": "warn",
    "codesmells/feature-envy": "warn",
    "codesmells/shotgun-surgery": "off",
    "repo/test-placement": "warn",
    "repo/path-naming": "warn",
    "frontend/feature-system-layout": "warn",
    "frontend/feature-system-public-api": "warn",
    "frontend/feature-system-dependency-flow": "warn",
    "frontend/feature-system-adapter-contract": "warn",
    "frontend/feature-system-query-contract": "warn",
    "cleanarch/unclassified-file": "warn"
  },
  "overrides": []
}
```

## Rule Scope

OnionCry focuses on architecture rules that need project-specific knowledge:

- Layer direction, such as `application` importing `infra`.
- Cross-context imports that bypass a public surface.
- External packages in sensitive layers, such as domain imports.
- Framework or data-format dependencies from core layers.
- Public-surface re-exports of internal implementation details.
- Context-level cycles and ownership checks.
- Context-first Clean Architecture artifact placement or Vertical Slice
  feature-local boundaries, public entry points, and shared-layer drift,
  depending on `architecture.mode`.

See [`docs/architecture-modes.md`](docs/architecture-modes.md) for the
`cleanArchitecture` and `verticalSlice` configuration contracts. OnionCry runs
only the architecture-specific rule family selected by `architecture.mode`;
architecture-neutral rules can run in either mode.

Generic JavaScript and TypeScript checks belong to Oxlint, Biome, TypeScript, or
similar tools. OnionCry does not report generic unresolved imports, file-level
cycles, max lines, max parameters, warning comments, or generic restricted
imports as first-class diagnostics.

## Code Organization Rules

OnionCry also checks observable repository conventions when you enable the rule.
These rules are adoption-friendly: configure them as `warn` first, then raise
them to `error` when the repository already follows the contract.

```jsonc
{
  "rules": {
    "cleanarch/artifact-placement": "warn",
    "verticalslice/no-cross-slice-internal-import": "warn",
    "verticalslice/no-global-slice-artifacts": "warn",
    "verticalslice/slice-entry-point": "warn",
    "verticalslice/no-shared-layer-artifacts": "warn",
    "repo/test-placement": "warn",
    "repo/path-naming": "warn",
    "frontend/feature-system-layout": "warn",
    "frontend/feature-system-public-api": "warn",
    "frontend/feature-system-dependency-flow": "warn",
    "frontend/feature-system-adapter-contract": "warn",
    "frontend/feature-system-query-contract": "warn"
  }
}
```

`repo/test-placement` separates source-level unit tests from workspace-level
integration and e2e tests. By default, unit tests live in co-located `__tests__`
folders, integration tests live under `tests/integration`, and e2e tests live
under `tests/e2e`.

`repo/path-naming` checks path casing, plural collection directories such as
`repositories`, singular feature directories, the configured layer directory
vocabulary, and optional suffixes for collection-owned files.

The `verticalslice/*` rules run only when `architecture.mode` is
`verticalSlice`. The default slice layout is
`src/features/<domain>/<operation>` (`sliceRoot: "features"`,
`sliceDepth: 2`). Set `sliceDepth: 1` for projects that intentionally use
`src/features/<feature>` or root-level feature folders.

The `frontend/feature-system-*` rules check frontend systems under
`packages/frontend/src/systems/<domain>` by default:

- `frontend/feature-system-layout` requires the system folders, root `index.ts`,
  allowed shared UI roots, and surface CSS placement.
- `frontend/feature-system-public-api` keeps external callers on explicit system
  barrels and rejects wildcard re-exports by default.
- `frontend/feature-system-dependency-flow` enforces import direction inside a
  system and between systems.
- `frontend/feature-system-adapter-contract` checks domain-named API adapters,
  typed API errors, abort-aware reads, and adapter isolation from upper frontend
  layers.
- `frontend/feature-system-query-contract` keeps TanStack Query keys and options
  in `lib`, makes hooks reuse option factories, and checks cache invalidation
  ownership when it is observable.

## Output Formats

Pretty output is meant for humans:

```bash
onioncry check
```

LLM output groups repeated diagnostics and keeps locations explicit:

```bash
onioncry check --llm-mode
```

During alpha, the final lines include the report format version and source
revision of the installed binary:

```text
---------------------------------------------------------------------------------------
onioncry-llm-report v1 revision: abc1234-dirty
```

JSON output is for scripts and CI:

```bash
onioncry check --format json
```

## Development

Common repository commands:

```bash
make install       # build and install the local CLI for alpha testing
make verify        # run all repository checks
make fmt           # format Rust code
make fmt-check     # check Rust formatting
make check         # cargo check
make lint          # clippy with warnings denied
make test          # Rust tests
make build         # build the Rust project
```

Before finishing any code or documentation change, run:

```bash
make verify
```

## Status

Alpha. The MVP supports `init`, `check`, and `explain` for JavaScript and
TypeScript import graphs using configured aliases, layers, contexts, rules, and
overrides.

Known alpha constraints:

- Distribution is local-only through `make install`.
- Alias resolution is explicit in `.onioncryrc.jsonc`; OnionCry does not infer
  `tsconfig` paths yet.
- Local import resolution handles common JS/TS source extensions and index
  files, but it is not a full TypeScript or Node resolver.
- Generic JS/TS lint rules are intentionally delegated to JS linters.

## License

OnionCry is licensed under the [MIT License](LICENSE).
