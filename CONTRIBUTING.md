# Contributing

Thanks for your interest in Onioncry.

This project is in its initial planning stage. Before contributing code, check the current project plan and open issues so changes stay aligned with the intended CLI behavior, configuration model, and dependency graph design.

## Development Workflow

1. Open an issue or comment on an existing one before starting larger changes.
2. Keep pull requests focused and small enough to review.
3. Add tests for behavior changes.
4. Run the relevant checks before opening a pull request.

Once the Rust crate is scaffolded, the expected local checks will be:

```bash
rtk cargo test
rtk cargo clippy
rtk cargo fmt --check
```

## Project Direction

The first implementation milestones are:

- Define the CLI contract.
- Define the YAML configuration format.
- Parse JavaScript and TypeScript imports.
- Build a dependency graph.
- Report architecture violations with useful diagnostics.
