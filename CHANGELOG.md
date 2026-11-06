# Changelog

All notable changes to OnionCry are documented in this file. Entries are
generated from Conventional Commit history with
[cocogitto](https://github.com/cocogitto/cocogitto).

## v0.3.1 - 2026-11-06

Removes the unused `miette` dependency, so installing from crates.io compiles
fewer crates. No CLI behavior change.

#### Build system

- (**deps**) remove the unused miette dependency (#24) - (40070f0) - Marcio Altoé

## v0.3.0 - 2026-11-06

`cleanarch/artifact-placement` now reports a file classified only by its
artifact suffix, such as `application/x.use-case.ts`, when it sits outside its
role folder under the context-first layout. Projects that raised the rule to
`error` may see new diagnostics; a per-file override relaxes it during a
migration. `frontend/feature-system-query-contract` treats key-factory keys in
QueryClient filters as uses of existing keys and accepts a mutation's
`meta.invalidates` (with `[]` for a mutation that writes nothing) as its
invalidation contract. The minimum supported Rust version is now 1.97.

#### Features

- deliver 0003-query-contract-filter-keys-and-mutation-meta (#22) - (e562392) - Marcio Altoé
- deliver 0002-suffixed-artifacts-require-their-role-folder (#21) - (fbc0dbc) - Marcio Altoé

#### Documentation

- archive Spec 0001 and plan Specs 0002 and 0003 (#19) - (03df5cd) - Marcio Altoé

#### Build system

- raise MSRV to 1.97 to update oxc to 0.155 (#18) - (887bc93) - Marcio Altoé

#### Miscellaneous Chores

- (**agents**) refresh the managed Baseline and Roundfix skills (#20) - (9c251d6) - Marcio Altoé
- (**repo**) update dependencies, skills, baseline and tooling (#17) - (a528df8) - Marcio Altoé

## v0.2.1 - 2026-10-26

Releases now publish to npm and crates.io through OIDC trusted publishing; npm
packages carry provenance. No CLI behavior change.

#### Continuous Integration

- (**release**) publish to npm and crates.io through trusted publishing (#15) - (f022fa2) - Marcio Altoé

## v0.2.0 - 2026-10-26

The `frontend/feature-system-*` rules now honor per-file overrides, and the
adapter contract proves named `Error` class exports through reexports.

#### Features

- follow second brain move to marcioaltoe/secondbrain (#9) - (7a84ed7) - Marcio Altoé
- join Tabularium mirror sync (#8) - (7e0ab4a) - Marcio Altoé

#### Bug Fixes

- (**cli**) honor file overrides and resolve Error reexports (#12) - (bf8862f) - Marcio Altoé

#### Documentation

- authorize feature-system fixes and stable publication (#11) - (93c9e80) - Marcio Altoé

#### Miscellaneous Chores

- (**agents**) refresh managed baseline and required skills (#13) - (5a96e4e) - Marcio Altoé
- (**repo**) update dependencies, baseline and tooling (#10) - (db73399) - Marcio Altoé
- (**agents**) add canonical onioncry skill and pre-PR update rule - (b2bcfb7) - Marcio Altoe

## v0.1.0 - 2026-07-04

First public release.

#### Features

- (**cli**) add introspection, baseline, onboarding, and integration cycles (#6) - (e624b09) - Marcio Altoé
- (**cli**) add code organization rules and source revision metadata (#4) - (c83b08d) - Marcio Altoé
- ship onioncry mvp - (a82abaf) - Marcio Altoé

#### Bug Fixes

- (**config**) discover json config fallback (#3) - (1a44cc4) - Marcio Altoé

#### Documentation

- (**repo**) scaffold onioncry mvp context and conventions (#1) - (3a82d63) - Marcio Altoé

#### Miscellaneous Chores

- (**agents**) refresh skills setup and agent instructions (#7) - (12204f1) - Marcio Altoé
- (**agents**) migrate to rust-cli skills setup (#5) - (f1b3c59) - Marcio Altoé
- initial project setup - (c141d4a) - Marcio Altoe
