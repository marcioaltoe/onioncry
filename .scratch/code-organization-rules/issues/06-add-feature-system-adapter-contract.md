# Add `frontend/feature-system-adapter-contract`

Status: ready-for-agent

## Parent

../PRD.md

## What to build

Add a frontend feature system adapter contract rule that checks the observable shape of API adapter files. The rule should verify domain-named adapter files, namespace API exports, typed API error exports, cancellation-aware operations where applicable, and adapter import boundaries.

## Acceptance criteria

- [ ] `frontend/feature-system-adapter-contract` is accepted in the `rules` map with normal severity and option shapes.
- [ ] When a system has an adapter layer, the default adapter file is recognized as `adapters/<domain>-api.ts`.
- [ ] The adapter file exports a namespace object named `<domain>Api` by default.
- [ ] The adapter file exports a typed error class named `<Domain>ApiError` by default.
- [ ] Adapter operations that call configured HTTP clients or `fetch` report a violation when a cancellable read operation does not accept or pass an `AbortSignal`.
- [ ] Adapter files cannot import from upper frontend layers.
- [ ] Semantic response normalization is not enforced as a blocking violation.
- [ ] CLI tests cover missing adapter files, missing exports, cancellation shape, import boundary violations, and option customization.
- [ ] `make verify` passes.

## Blocked by

- 03-add-feature-system-layout.md
- 05-add-feature-system-dependency-flow.md
