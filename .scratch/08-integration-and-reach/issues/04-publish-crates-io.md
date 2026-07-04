# Publish to crates.io with tag-driven release CI

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Prepare the crate for publication and add a tag-driven release workflow so pushing a version tag publishes `onioncry` to crates.io. This issue also establishes the shared release workflow that the npm issue extends.

## Acceptance criteria

- [ ] Cargo.toml carries complete publish metadata: description, license, repository, keywords, categories, and an `exclude`/`include` set that keeps `.scratch/`, fixtures, and local tooling out of the package.
- [ ] `cargo publish --dry-run` succeeds locally and runs as a verification step in CI.
- [ ] A release workflow triggers on `v*` tags, runs the full verification suite, and publishes to crates.io using a repository secret.
- [ ] The workflow fails before publishing when the tag version and Cargo.toml version disagree.
- [ ] Workflow permissions follow least privilege; the crates.io token is used only in the publish step.
- [ ] The release process (bump, tag, push) is documented in the README or a `docs/release.md`.
- [ ] Workflow files use kebab-case names and pass any repository workflow checks.
- [ ] `make verify` passes.

## Blocked by

None - can start immediately (publishing a tag requires maintainer action; the issue delivers the automation)
