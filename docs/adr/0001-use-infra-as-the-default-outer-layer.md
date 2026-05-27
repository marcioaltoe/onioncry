# Use infra as the default outer layer

OnionCry's default architecture preset uses `domain`, `application`, `infra`, and `shared` as its layer vocabulary. `infra` is the outer layer and intentionally groups interface adapters with framework and driver details because this matches how many repositories organize controllers, repository implementations, database clients, queues, SDKs, and runtime bootstrapping. Projects that need stricter Clean Architecture granularity can define separate configured layers such as `adapters` and `frameworks`.
