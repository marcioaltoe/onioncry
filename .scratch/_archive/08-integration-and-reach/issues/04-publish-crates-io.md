# Publish to crates.io with tag-driven release CI

Status: done

## Parent

../PRD.md

## What to build

Prepare the crate for publication and add a tag-driven release workflow so pushing a version tag publishes `onioncry` to crates.io. This issue also establishes the shared release workflow that the npm issue extends.

## Acceptance criteria

- [x] Cargo.toml carries complete publish metadata: description, license, repository, keywords, categories, and an `exclude`/`include` set that keeps `.scratch/`, fixtures, and local tooling out of the package.
- [x] `cargo publish --dry-run` succeeds locally and runs as a verification step in CI.
- [x] A release workflow triggers on `v*` tags, runs the full verification suite, and publishes to crates.io using a repository secret.
- [x] The workflow fails before publishing when the tag version and Cargo.toml version disagree.
- [x] Workflow permissions follow least privilege; the crates.io token is used only in the publish step.
- [x] The release process (bump, tag, push) is documented in the README or a `docs/release.md`.
- [x] Workflow files use kebab-case names and pass any repository workflow checks.
- [x] `make verify` passes.

## Blocked by

None - can start immediately (publishing a tag requires maintainer action; the issue delivers the automation)

## Comments

- 2026-07-04: Added crates.io package metadata and an explicit Cargo include list, `make publish-dry-run`, `docs/release.md`, README release references, and `.github/workflows/release-crates.yml` for `v*` tag publishing with tag/Cargo version validation and a publish-step-only `CARGO_REGISTRY_TOKEN`. Verification: `rtk make publish-dry-run` passed; `rtk cargo package --list --allow-dirty --locked` confirmed `.scratch/`, tests/fixtures, `.agents`, and workflow/local tooling are excluded; workflow YAML parsed with Ruby; `actionlint` was unavailable locally; `rtk make verify` passed with clippy clean and 115 tests. The exact commit SHA is reported by the implementation loop after the slice commit is created.
