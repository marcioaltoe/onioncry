---
status: approved
granted: 2026-11-06
action: teach frontend/feature-system-query-contract that query keys in QueryClient filters are key uses and that a mutation's meta.invalidates declares its invalidation contract, verify and deliver the change, then publish a stable release through the existing release process
consuming:
  - 0003-query-contract-filter-keys-and-mutation-meta
paths:
  - src/rules/feature_system/query_contract.rs
  - src/rules/feature_system/helpers.rs
  - src/rules/feature_system/query_syntax.rs
  - src/rules/feature_system/mod.rs
  - tests/feature_system.rs
  - README.md
  - CONTEXT.md
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

On 2026-11-06 the maintainer answered, through the question tool in a `roundfix --follow-up` session, that the Vortex query-contract request becomes Spec 0003 and is delivered up to the release. That covers implementing, testing, committing, pushing, opening the PR, merging with approved CI and publishing on crates.io and npm. The maintainer also chose `meta.invalidates` in code, read from the syntax tree, as the contract for mutations that write nothing, with no configuration change. Keep the configured reviewer and every required check.

The release manifest paths above authorize only the own-package version changes that `docs/release.md` requires, after a conclusive public release plan. Regenerate the own-package `Cargo.lock` entry through Cargo. Dependency updates, `.onioncryrc.jsonc` shape changes and unrelated configuration changes are outside this grant. Version selection follows the release plan. Do not push a tag before the documented verification and the publish dry-runs pass.

Land this grant in `main` before the consuming delivery. It authorizes no credential change and no implementation in another repository.
