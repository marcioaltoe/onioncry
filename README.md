# Onioncry

Onioncry is a Rust CLI for inspecting JavaScript and TypeScript dependency structure and catching architecture drift early.

The project is in its initial planning stage. The intended shape is a fast command-line tool that reads a YAML configuration, matches project paths, parses imports, builds a dependency graph, and reports actionable terminal diagnostics.

## Planned Stack

- `clap` for the command-line interface
- `serde_yaml` for configuration
- `globset` for path matching
- `oxc_parser` or `swc_ecma_parser` for JavaScript and TypeScript parsing
- `petgraph` for dependency graph modeling
- `miette` for human-friendly terminal diagnostics

## Goals

- Analyze TypeScript and JavaScript projects from the terminal.
- Make architectural boundaries visible through a dependency graph.
- Report violations with clear file locations and useful remediation hints.
- Keep configuration small enough to understand at a glance.

## Status

Pre-alpha. The repository currently contains project setup and agent workflow configuration. The Rust crate, CLI contract, and architecture rules are still being designed.

## Development

The implementation branch will define the initial context, project plan, and scaffold.

Once the Rust project is scaffolded, expected development commands will include:

```bash
rtk cargo test
rtk cargo clippy
rtk cargo fmt --check
```

## Contributing

Contributions are welcome once the initial project plan is published. See [CONTRIBUTING.md](CONTRIBUTING.md) for the starting workflow.

## License

Onioncry is licensed under the [MIT License](LICENSE).
