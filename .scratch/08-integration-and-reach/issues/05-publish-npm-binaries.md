# Publish npm packages with prebuilt binaries

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Ship OnionCry to its JavaScript/TypeScript audience as an npm package with prebuilt platform binaries, so `npx onioncry` works without a Rust toolchain. Binaries are built in the release workflow and published directly as npm platform packages — GitHub Releases is not a distribution channel.

## Acceptance criteria

- [ ] A main `onioncry` npm package provides the `onioncry` bin entry through a small launcher that resolves the installed platform package and executes the native binary, forwarding args, stdio, and exit codes unchanged.
- [ ] Platform packages (at minimum `darwin-arm64`, `darwin-x64`, `linux-x64`, `linux-arm64`, `win32-x64`) carry the prebuilt binary and correct `os`/`cpu` fields, wired from the main package via `optionalDependencies`.
- [ ] The release workflow builds each platform binary on tag push (native runners or cross-compilation) and publishes the platform packages and main package in one run.
- [ ] The workflow fails before publishing when the tag, Cargo.toml version, and npm package versions disagree.
- [ ] The launcher prints a clear error naming the platform and the supported platform list when no platform package is available.
- [ ] `npx onioncry@<version> --help` works on a clean machine for at least one platform, verified in CI.
- [ ] npm packaging sources live in a dedicated directory (for example `npm/`) without affecting `make verify` for the Rust crate.
- [ ] README installation section documents npm as the primary channel and crates.io as the alternative, replacing the alpha `make install` guidance.
- [ ] Least-privilege workflow permissions; the npm token is used only in publish steps.
- [ ] `make verify` passes.

## Blocked by

04-publish-crates-io (extends the tag-driven release workflow)
