# Use oxc_parser for JavaScript and TypeScript

OnionCry uses `oxc_parser` for the MVP JavaScript and TypeScript scanner because the product is shaped like a fast linter and is already aligned with Oxc/Oxlint configuration conventions. The MVP needs reliable import and re-export extraction with good TypeScript support, not code transformation. `swc_ecma_parser` remains a viable alternative, but switching parsers later would affect AST traversal, spans, diagnostics, and scanner tests.
