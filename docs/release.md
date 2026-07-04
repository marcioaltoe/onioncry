# Release Process

OnionCry releases are tag-driven. Maintainers publish the Rust crate by pushing a
version tag that matches `Cargo.toml`.

## Crates.io Setup

Add a repository secret named `CARGO_REGISTRY_TOKEN` with a crates.io API token
that can publish the `onioncry` crate. The release workflow exposes this token
only to the final `cargo publish` step.

## Release Steps

1. Update `version` in `Cargo.toml`.
2. Run `rtk cargo update -p onioncry` only if the lockfile needs a package
   metadata refresh for the version change.
3. Run `rtk make verify`.
4. Run `rtk make publish-dry-run`.
5. Commit the version change with a Conventional Commit.
6. Create and push a matching version tag:

```bash
rtk git tag v0.1.0
rtk git push origin HEAD
rtk git push origin v0.1.0
```

The `release-crates.yml` workflow fails before publishing if the tag version and
`Cargo.toml` version disagree. For example, tag `v0.2.0` requires
`version = "0.2.0"`.
