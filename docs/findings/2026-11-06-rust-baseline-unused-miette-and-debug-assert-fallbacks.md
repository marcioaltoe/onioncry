---
status: pending
created_at: 2026-11-06
updated_at: 2026-11-06
---

# Rust conventions — unused `miette` and panicking `debug_assert!` fallbacks (2026-11-06)

The Roundfix project validated its Rust Baseline rules (its Spec 0206) by reading this repository and reported what an adoption review would question. The report arrived as ai-memory message `01a17436-c560-78e1-bc97-3957f372442d` from `marcioaltoe-roundfix`, written on 2026-09-30 and relayed on 2026-10-25. This Finding re-checked both observations on `main` at `887bc93` on 2026-11-06. The library already returns typed errors and production code calls no `unwrap`, `expect` or `panic!`, so those parts of the report need no action.

## 1. `miette` is declared but no code uses it

- Symptom / evidence: `Cargo.toml:27` declares `miette = "7.6.0"`, and `rg -ln miette src` returns no file. `CLAUDE.md` lists `miette` as the established crate for terminal diagnostics and says "render diagnostics through `miette`", so the instructions describe a dependency the code does not use.
- Root cause: the dependency arrived with the MVP (`a82abaf`), but the diagnostics renderer never adopted it.
- Action / suggestion: decide through an ADR, because `CLAUDE.md` says established crate decisions change only that way. Either render diagnostics through `miette`, or remove the dependency with `cargo remove miette` and update `CLAUDE.md`. Route the work to a Spec once the ADR decision is made.

## 2. Four `debug_assert!(false, …)` fallbacks panic in debug and test builds

- Symptom / evidence: `src/rules/clean_boundaries/cycle.rs:150`, `:160`, `:170` and `:180`, inside the Tarjan strongly connected component walk, guard states the algorithm should never reach. Release builds skip the branch and continue, while debug and test builds panic.
- Root cause: these are defensive invariant checks. No input that reaches them is known, so they are not a defect today.
- Action / suggestion: when the Spec for item 1 runs, decide whether to keep them as documented invariants or to make the walk unable to reach those states by construction, for example by tracking lowlinks in the same structure that marks a node visited. No user-facing failure is known.

## Addendum — 2026-11-06 — item 1 resolved

The maintainer chose to remove `miette`. ADR-0015 records the decision, `cargo remove miette` dropped the dependency, and `AGENTS.md` no longer names it. Item 2 stays pending.
