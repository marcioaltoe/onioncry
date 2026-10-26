---
status: approved
granted: 2026-10-26
action: correct feature-system per-file overrides and genuine named Error class reexports, verify and deliver the fixes, then publish a stable release through the existing release process
consuming:
  - 0001-feature-system-overrides-and-error-reexports
paths:
  - src/rules/feature_system/mod.rs
  - src/rules/feature_system/helpers.rs
  - src/rules/feature_system/adapter_contract.rs
  - src/rules/feature_system/error_exports.rs
  - tests/feature_system.rs
  - .agents/skills/onioncry/SKILL.md
  - Cargo.toml
  - Cargo.lock
  - npm/package.json
  - npm/platforms/darwin-arm64/package.json
  - npm/platforms/darwin-x64/package.json
  - npm/platforms/linux-arm64/package.json
  - npm/platforms/linux-x64/package.json
  - npm/platforms/win32-x64/package.json
  - .agents/skills/ai-memory-handoff/LICENSE
  - .agents/skills/ai-memory-handoff/SKILL.md
  - .agents/skills/ai-memory-messaging/LICENSE
  - .agents/skills/ai-memory-messaging/SKILL.md
  - .agents/skills/roundfix/SKILL.md
  - .agents/skills/roundfix/references/baseline.md
  - .agents/skills/roundfix/references/follow-up.md
  - .agents/skills/roundfix/references/modes.md
  - .agents/skills/roundfix/references/setup.md
  - .agents/skills/roundfix/references/storage.md
  - .agents/skills/setup-context-driven/SKILL.md
  - .agents/skills/write-idea/SKILL.md
  - .agents/skills/write-prd/SKILL.md
  - .agents/skills/write-techspec/SKILL.md
  - docs/agents/agent-instructions.md
  - docs/agents/docs-layout.md
  - docs/agents/secondbrain.md
  - docs/agents/setup-context.json
  - docs/agents/skill-dispatch.md
  - docs/agents/spec-routing.md
  - skills-lock.json
operations:
  - implement
  - commit
  - push
  - pull_request
  - merge
  - release
---

# Authorization

The maintainer explicitly approved correcting and publishing OnionCry through the question tool on 2026-10-26, including tests, review, PR and merge with approved CI. Preserve the configured reviewer and all required checks. Source implementation and real CLI regression tests are authorized for the two defects only. The canonical shipped skill must describe the corrected behavior in the same PR.

The exact release manifest paths above authorize the own-package version changes required by docs/release.md after a conclusive public release plan; dependency updates and unrelated configuration changes are outside this grant. Regenerate the own-package Cargo.lock entry through Cargo. Version selection follows the release plan; a required minor/major or version-zero breaking decision remains a maintainer decision rather than an inferred waiver. Do not publish a tag before the documented verification and publish dry-runs pass.

Land this grant in main before the consuming delivery. No credential change, production data write, unrelated package removal or other repository's implementation is authorized by this record.


## Managed Baseline refresh on 2026-10-26

The maintainer requested renewed skills/Baseline maintenance and instructed this session to continue after the exact Vortex and OnionCry plans were presented. The approved OnionCry Plan is sha256:99f2e7110276c8cd1e22779d7364671dd243ce0e5aca42b649be0fc5b7f87904. The added exact paths are the six managed guide/manifest outputs, changed binary-carried skill files, the two missing required ai-memory skills and the generated skills lock. They authorize only the public Roundfix managed-refresh and restoration operations; preserve the profile, selected models/reviewer, repository-authored bytes and repository-canonical OnionCry skill. Do not hand-edit managed content or version metadata as part of the refresh.

Land this amendment in main before the consuming Baseline commit. Keep its commit separate from the generated update. The existing functional correction, delivery and conditional stable-publication authority remains unchanged; required version decisions, review, CI and release dry-runs remain mandatory.
