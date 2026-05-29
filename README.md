# OnionCry

OnionCry is a Rust CLI for checking architectural boundaries in JavaScript and
TypeScript projects. It names architecture drift with linter-style diagnostics so
teams can see when layers, bounded contexts, public surfaces, or core package
policies are being crossed.

The project is currently in alpha. The CLI is usable locally, but distribution is
not published through Homebrew, Cargo, or package managers yet.

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

Later releases should be distributed through a Homebrew tap or another published
installer. Until then, `make install` is the supported local testing path.

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
```

To inspect one file:

```bash
onioncry explain packages/backend/src/application/use-cases/example.ts
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
onioncry check --fail-on warning
onioncry check --tips
onioncry check --llm-mode
```

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

## Configuration

OnionCry auto-discovers `.onioncryrc.jsonc` first, then `.onioncryrc.json`.
JSONC remains the default because the configuration works like a linter config,
with comments, rules, severities, options, and overrides.

Minimal shape:

```jsonc
{
  "version": 1,
  "project": {
    "root": ".",
    "include": ["packages/backend/src/**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs}"],
    "exclude": ["node_modules/**", "dist/**", "build/**", "coverage/**", "**/__tests__/**"]
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
    "solid/no-concrete-dependency": "warn",
    "codesmells/feature-envy": "warn",
    "codesmells/shotgun-surgery": "off",
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

Generic JavaScript and TypeScript checks belong to Oxlint, Biome, TypeScript, or
similar tools. OnionCry does not report generic unresolved imports, file-level
cycles, max lines, max parameters, warning comments, or generic restricted
imports as first-class diagnostics.

## Output Formats

Pretty output is meant for humans:

```bash
onioncry check
```

LLM output groups repeated diagnostics and keeps locations explicit:

```bash
onioncry check --llm-mode
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
