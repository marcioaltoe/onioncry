# task-002: Detect local import graph

## Description

Build the scanner and resolver path that parses JavaScript and TypeScript files with oxc_parser, extracts import edges, resolves local relative and configured-alias imports, and reports unresolved local imports.

## Acceptance

- The scanner extracts static imports, type-only imports, re-exports, string-literal dynamic imports, and string-literal require calls.
- Non-literal dynamic imports and non-literal require calls are ignored in the MVP.
- Relative imports and configured aliases resolve using common TS/JS extensions and index files.
- tsconfig.json, package exports, package main, project references, and .d.ts resolution are out of scope.
- onion/unresolved-import reports only unresolved relative and configured-alias imports, not external package imports.
- Diagnostics include file path, import specifier, and line/column when available.
- Tests cover import extraction forms, alias resolution, extension/index resolution, unresolved local imports, and ignored external packages.
- make verify passes.
