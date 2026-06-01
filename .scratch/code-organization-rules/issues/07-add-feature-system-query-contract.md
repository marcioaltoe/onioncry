# Add `frontend/feature-system-query-contract`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a frontend feature system query contract rule for TanStack Query ownership. The rule should keep query keys and query options under `lib`, require hooks to reuse option factories, propagate cancellation signals to adapters, and enforce explicit mutation cache handling.

## Acceptance criteria

- [ ] `frontend/feature-system-query-contract` is accepted in the `rules` map with normal severity and option shapes.
- [ ] `lib/query-keys.ts` is required when a system owns query hooks, route loaders, prefetching, cache reads, or adapter-backed reads.
- [ ] `lib/query-options.ts` is required when a system uses `useQuery`, route loader queries, prefetching, or cache reads.
- [ ] Query option files import and use `queryOptions` from `@tanstack/react-query`.
- [ ] Query option factories co-locate `queryKey` and `queryFn`.
- [ ] Query functions pass the query context `signal` to adapter calls when applicable.
- [ ] Query hooks reuse factories from `lib/query-options.ts` instead of declaring query keys and query functions inline.
- [ ] Components and routes do not own duplicate query keys by default.
- [ ] Mutation hooks invalidate relevant queries in `onSuccess` or `onSettled`.
- [ ] Optimistic cache updates that use `onMutate` include outgoing query cancellation, previous data snapshot or rollback, and settlement invalidation.
- [ ] Cache scope sufficiency remains out of scope unless encoded as explicit rule options.
- [ ] CLI tests cover each automatic check and at least one assisted-review boundary that should not fail.
- [ ] `make verify` passes.

## Blocked by

- 03-add-feature-system-layout.md
- 05-add-feature-system-dependency-flow.md
- 06-add-feature-system-adapter-contract.md
