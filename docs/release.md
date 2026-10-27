# Release Process

OnionCry releases are tag-driven. Maintainers publish crates.io and npm packages
by pushing a version tag that matches `Cargo.toml` and every npm package
manifest under `npm/`.

## Trusted Publishing Setup

The `publish` job authenticates to both registries through GitHub Actions OIDC
trusted publishing; the repository stores no registry tokens. The job runs in
the `release-crates` GitHub environment with `id-token: write`; that environment only accepts `v*` tags.

- **crates.io**: the `onioncry` crate trusts `marcioaltoe/onioncry`, workflow
  `release-crates.yml`, environment `release-crates`. The
  `rust-lang/crates-io-auth-action` step exchanges the OIDC token for a
  short-lived publish token used only by `cargo publish`.
- **npm**: `onioncry` and every `@onioncry/cli-*` platform package trust
  `marcioaltoe/onioncry`, workflow `release-crates.yml`, with direct publish
  allowed. Trusted publishing requires npm 11.5.1 or later, so the job installs
  npm 11 before publishing; npm attaches provenance automatically.

Adding a new platform package requires configuring its trusted publisher on
npmjs.com before the first release that publishes it.

## Release Steps

1. Update `version` in `Cargo.toml`.
2. Update `version` in `npm/package.json`, every
   `npm/platforms/*/package.json`, and every `optionalDependencies` entry in
   `npm/package.json`.
3. Run `rtk node npm/scripts/check-versions.mjs`.
4. Run `rtk cargo update -p onioncry` only if the lockfile needs a package
   metadata refresh for the version change.
5. Run `rtk make verify`.
6. Run `rtk make publish-dry-run`.
7. Commit the version change with a Conventional Commit.
8. Create and push a matching version tag:

```bash
rtk git tag v0.1.0
rtk git push origin HEAD
rtk git push origin v0.1.0
```

The `release-crates.yml` workflow fails before publishing if the tag version,
`Cargo.toml` version, main npm package version, platform package versions, or
main package `optionalDependencies` versions disagree. For example, tag `v0.2.0`
requires every package to use `0.2.0`.

On tag push, the workflow builds platform packages for macOS arm64, macOS x64,
Linux arm64, Linux x64, and Windows x64. It verifies a clean npm install from
the generated Linux x64 tarballs with `npx onioncry --help`, publishes all npm
platform packages, publishes the main npm package, then publishes the Rust crate
to crates.io.
