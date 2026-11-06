---
status: approved
granted: 2026-11-06
action: correct cleanarch/artifact-placement so a suffix-only artifact under the Context-First Clean Architecture Layout must sit in its role folder, verify and deliver the fix, then publish a stable release through the existing release process
consuming:
  - 0002-suffixed-artifacts-require-their-role-folder
paths:
  - src/rules/clean_architecture/artifact_policy.rs
  - tests/clean_architecture.rs
  - .agents/skills/onioncry/SKILL.md
  - Cargo.toml
  - Cargo.lock
  - npm/package.json
  - npm/platforms/darwin-arm64/package.json
  - npm/platforms/darwin-x64/package.json
  - npm/platforms/linux-arm64/package.json
  - npm/platforms/linux-x64/package.json
  - npm/platforms/win32-x64/package.json
operations:
  - implement
  - commit
  - push
  - pull_request
  - merge
  - release
---

# Authorization

On 2026-11-06 the maintainer answered, through the question tool in a `roundfix --follow-up` session, that the correction of `cleanarch/artifact-placement` for suffixed files may go up to the release. That covers implementing, testing, committing, pushing, opening the PR, merging with approved CI and publishing the patch release on crates.io and npm. Keep the configured reviewer and every required check.

The release manifest paths above authorize only the own-package version changes that `docs/release.md` requires, after a conclusive public release plan. Regenerate the own-package `Cargo.lock` entry through Cargo. Dependency updates and unrelated configuration changes are outside this grant. Version selection follows the release plan. Do not push a tag before the documented verification and the publish dry-runs pass.

Land this grant in `main` before the consuming delivery. It authorizes no credential change and no implementation in another repository.
