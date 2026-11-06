# Query contract filter keys and mutation meta evidence (2026-11-06)

## Reported limitation

While updating the Vortex frontend, `bun run onioncry` reported 15 `frontend/feature-system-query-contract` warnings. Twelve hooks combine queries built from existing option factories with mutations that invalidate those queries. Two components reload queries. One hook uses `useMutation` to download a file without modifying server data. The report arrived as ai-memory message `01a17436-c585-7ae2-ad32-c86fb643132d` from `vortex`, dated 2026-10-25. Its other two limitations, per-file overrides and named Error reexports, were delivered by Spec 0001 (`docs/history/specs/0001-feature-system-overrides-and-error-reexports.md`).

## Root cause on `main` at `887bc93`

`src/rules/feature_system/helpers.rs` decides ownership by text. `source_declares_query_key` returns true for any source containing `queryKey`, so the key inside `invalidateQueries({ queryKey: keys.list() })` counts as a declaration. In a hook that also reads a query, that yields "query hooks should reuse factories"; in a component it yields "components should not own query keys". `source_has_mutation_invalidation` requires `onSuccess` or `onSettled` together with `invalidateQueries` for any file containing `useMutation(`, so a mutation that writes nothing has no compliant form.

## Outside source: meta-driven invalidation

Source read on 2026-11-06: Dominik Dorfmeister (TkDodo, TanStack Query maintainer), "Automatic Query Invalidation after Mutations", https://tkdodo.eu/blog/automatic-query-invalidation-after-mutations , and his reply in the article's comment thread, https://github.com/TkDodo/blog-comments/discussions/105 . The article says: "We can use `meta` to store arbitrary, static information about a Mutation. As an example, we can add an `invalidates` field to give “tags” to our mutation." Its example declares `meta: { invalidates: [['issues'], ['labels']] }` and invalidates from a global `MutationCache` `onSuccess`. In the comment thread the maintainer writes: "What I did is to invalidate everything, except if `invalidates` tags are passed. For some mutations, like `logout`, it just pass: `{ invalidates: [] }` to not invalidate anything." The TanStack Query reference (Context7 `/tanstack/query`, `UseMutationOptions`) documents `meta` as "Additional payload to be stored on the mutation cache entry" that the `MutationCache` callbacks can read.

This supports ADR-0014: `meta.invalidates` is a published, in-code declaration of a mutation's invalidation, and an empty list is the published way to say that a mutation invalidates nothing.

QA outside-evidence criterion: a feature-system hook whose `useMutation` declares `meta: { invalidates: [] }` and calls no `invalidateQueries` produces no mutation-invalidation diagnostic. The same hook without `meta`, or with a `meta` object that lacks `invalidates`, keeps the diagnostic. Inspect the real CLI JSON for all three.

## Consultation limits

Secondbrain index read: `~/dev/secondbrain/wiki/index.md`. The session's `qmd query` on OnionCry rules returned only this project's mirrors, byte-identical to the local originals; no Secondbrain page records a prior query-contract decision. Exa MCP returned the sources above. No Secondbrain file was written.
