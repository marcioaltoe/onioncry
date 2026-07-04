---
name: cut-release
description: Cut a release for a CLI or library — verify version sync across every manifest, run the full verification gate and publish dry-runs, tag, push, and watch the release workflow to completion with evidence at each step.
disable-model-invocation: true
argument-hint: "<version, e.g. 0.4.0> [--dry-run]"
metadata:
  category: release
  tags: [workflow, process, git, cli]
  version: 0.1.0
  author: Marcio Altoé
  source: https://github.com/marcioaltoe/skills
---

# Cut Release

Drive a release from "the code is ready" to "the artifacts are published", with evidence at every step. Releases fail late and expensively — a version mismatch discovered by the publish workflow wastes a tag and a CI run — so this skill front-loads every check that can fail before anything irreversible happens.

## 0. Learn this repo's release contract first

Read, in order of authority: the repo's release doc (`docs/release.md`, `RELEASE.md`, or a `## Release` section in README/CONTRIBUTING), the release workflow under `.github/workflows/` (what triggers it — usually a `v*` tag — and what it publishes), and the previous release tag (`git tag --sort=-creatordate | head`) for naming conventions. The repo's documented process always overrides the generic steps below; this skill fills gaps, it does not replace a written contract.

## 1. Pre-flight — everything that can fail cheaply

Run all of these before touching a version number; stop on the first failure:

- **Clean state**: working tree clean, on the default branch, up to date with origin.
- **Version-sync inventory**: find every file that carries the version (`Cargo.toml`, `package.json` and per-platform `npm/platforms/*/package.json` with their `optionalDependencies`, `pyproject.toml`, a `version.go` constant, docs). If the repo ships a checker (e.g. a `check-versions` script), it is the authority — run it. If not, grep the current version and list every hit; that list is the bump checklist.
- **Verification gate**: the repo's full verify pipeline (`make verify` or equivalents). Any warning is a blocker — release CI usually runs the same gate and fails late.
- **Changelog/release notes**: if the repo maintains one, confirm the new version's entry exists (generate from Conventional Commits history when that is the convention).

## 2. Bump and prove

- Bump the version in **every** file from the inventory — a partial bump is the classic late failure.
- Re-run the version-sync check and the verify gate on the bumped tree.
- Run every publish **dry-run** the ecosystem offers: `cargo publish --dry-run --locked`, `npm publish --dry-run` (per package), `goreleaser release --snapshot --clean`. A dry-run failure caught here costs seconds; the same failure after tagging costs a broken tag.
- Commit the bump as its own Conventional Commit (e.g. `chore: release v0.4.0` — check `cog.toml` for scope rules). Do not mix release bumps with feature changes.

With `--dry-run`, stop here and report what would happen.

## 3. Tag and push — the point of no return

Confirm with the user before this step unless they already named the version in the request — the tag is what triggers publication.

```bash
git push                    # the bump commit must be on the remote first
git tag v<version>
git push origin v<version>
```

## 4. Watch and verify

- Watch the release workflow to completion (`gh run watch` or poll `gh run list`). A triggered workflow is not a released artifact.
- Verify the artifacts exist where users get them: the crates.io/npm/registry page shows the new version, `npx <tool>@latest --version` / `cargo install` resolves it, the GitHub Release (when the repo creates one) carries the binaries.
- Report with evidence: workflow run URL and conclusion, artifact URLs, installed-version check output.

## Failure protocol

- Workflow fails **before** anything published → fix the root cause, delete the tag only if the repo's convention allows re-tagging (`git push --delete origin v<version>`), and restart from step 1.
- Workflow fails **after** partial publication (npm out, crates.io failed) → never delete published artifacts; publish the missing half manually per the repo's release doc, and record what happened in the release notes.
- Never force-push tags over a published version — registries cache; a moved tag creates unverifiable builds.

## Anti-patterns

- Tagging before the dry-runs — the tag is the trigger, not the test.
- Bumping only the "main" manifest when platform sub-packages carry the version too.
- Claiming "released" when the workflow started — released means the artifact is fetchable.
- Skipping the release doc because the generic steps look sufficient — repos encode hard-won exceptions there.
- Mixing the release bump with unrelated changes in one commit.
