# Publish npm packages with prebuilt binaries

Status: done

## Parent

../PRD.md

## What to build

Ship OnionCry to its JavaScript/TypeScript audience as an npm package with prebuilt platform binaries, so `npx onioncry` works without a Rust toolchain. Binaries are built in the release workflow and published directly as npm platform packages — GitHub Releases is not a distribution channel.

## Acceptance criteria

- [x] A main `onioncry` npm package provides the `onioncry` bin entry through a small launcher that resolves the installed platform package and executes the native binary, forwarding args, stdio, and exit codes unchanged.
- [x] Platform packages (at minimum `darwin-arm64`, `darwin-x64`, `linux-x64`, `linux-arm64`, `win32-x64`) carry the prebuilt binary and correct `os`/`cpu` fields, wired from the main package via `optionalDependencies`.
- [x] The release workflow builds each platform binary on tag push (native runners or cross-compilation) and publishes the platform packages and main package in one run.
- [x] The workflow fails before publishing when the tag, Cargo.toml version, and npm package versions disagree.
- [x] The launcher prints a clear error naming the platform and the supported platform list when no platform package is available.
- [x] `npx onioncry@<version> --help` works on a clean machine for at least one platform, verified in CI.
- [x] npm packaging sources live in a dedicated directory (for example `npm/`) without affecting `make verify` for the Rust crate.
- [x] README installation section documents npm as the primary channel and crates.io as the alternative, replacing the alpha `make install` guidance.
- [x] Least-privilege workflow permissions; the npm token is used only in publish steps.
- [x] `make verify` passes.

## Blocked by

04-publish-crates-io (extends the tag-driven release workflow)

## Comments

- 2026-07-04: Added the `npm/` package layout with a main `onioncry` launcher, optional platform packages for darwin-arm64, darwin-x64, linux-arm64, linux-x64, and win32-x64, version lockstep checks, launcher tests, and README install docs. Extended the tag-driven release workflow to build all platform tarballs, smoke-test Linux x64 with local npm tarballs and `npx onioncry --help`, publish npm packages with `NPM_TOKEN` scoped to the npm publish step, and publish crates.io with `CARGO_REGISTRY_TOKEN` scoped to the cargo publish step. Verification: `rtk node npm/scripts/check-versions.mjs` passed; `rtk npm test --prefix npm` passed; `rtk npm pack ./npm --dry-run` produced `onioncry-0.1.0.tgz`; workflow YAML parsed with Ruby; `rtk make verify` passed with clippy clean and 115 tests. The exact commit SHA is reported by the implementation loop after the slice commit is created.
