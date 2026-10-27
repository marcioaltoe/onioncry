# Changelog

All notable changes to OnionCry are documented in this file. Entries are
generated from Conventional Commit history with
[cocogitto](https://github.com/cocogitto/cocogitto).

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
