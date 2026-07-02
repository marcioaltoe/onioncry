# Architecture modes

OnionCry checks one project architecture mode at a time. If a project does not configure a mode, OnionCry uses Clean Architecture.

## Configuration

Use `architecture.mode` to select the mode:

```jsonc
{
  "architecture": {
    "mode": "cleanArchitecture"
  }
}
```

Valid values:

| Value | Meaning |
| --- | --- |
| `cleanArchitecture` | Layered dependency validation with Clean Architecture organization rules. This is the default. |
| `verticalSlice` | Slice-based validation for feature-local code and cross-slice boundaries. |

Architecture-specific rule families are mutually exclusive. A `verticalSlice` project cannot enable `cleanarch/*` rules, and a `cleanArchitecture` project cannot enable `verticalslice/*` rules. OnionCry treats that as a configuration error.

Architecture-neutral rules, such as repository path naming or test placement rules, are independent of the selected mode.

## Clean Architecture

Clean Architecture mode uses context-first organization by default:

```txt
src/
  contexts/<context>/
    domain/
      entities/
      value-objects/
      aggregates/
      events/
      services/
      errors/
    application/
      use-cases/
      ports/
      dtos/
      mappers/
      services/
      events/
    infra/
      repositories/
      adapters/
      controllers/
      database/
      workflows/
      bootstrap/

  domain/
    entities/
    value-objects/
  application/
    use-cases/
    ports/
  infra/
    repositories/
    adapters/
    bootstrap/
```

The root `domain/`, `application/`, and `infra/` folders are for contextless base code. Contextual code belongs under the configured context root.

The default mode options are:

```jsonc
{
  "architecture": {
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "contexts",
      "layerPathAliases": {
        "infra": ["infra", "infrastructure"]
      },
      "artifactFolders": {
        "domain": ["entities", "value-objects", "aggregates", "events", "services", "errors"],
        "application": ["use-cases", "ports", "dtos", "mappers", "services", "events"],
        "infra": ["repositories", "adapters", "controllers", "database", "workflows", "bootstrap"]
      },
      "artifactSuffixes": {
        "repository": [".repository.ts", "-repository.ts", "-catalog.ts", ".writer.ts", "-writer.ts", "-writers.ts"],
        "service": [".service.ts", "-service.ts"],
        "useCase": [".use-case.ts", "-use-case.ts"],
        "entity": [".entity.ts", "-entity.ts"],
        "valueObject": [".value-object.ts", "-value-object.ts"],
        "adapter": [".adapter.ts", "-adapter.ts", ".gateway.ts", "-gateway.ts", "/client.ts", ".client.ts", "-client.ts", "/handler.ts", ".mapper.ts", "-mapper.ts", "-mappers.ts", ".parser.ts", "-parser.ts", ".provider.ts", "-provider.ts", ".request.ts", "-request.ts", "-requests.ts", ".schema.ts", "-schema.ts", "-schemas.ts", "-normalization.ts", "-resilience.ts", "-composition.ts", "-scenario.ts", "-scenarios.ts", "-snapshot.ts", "-snapshots.ts"],
        "handler": [".handler.ts", "-handler.ts"],
        "port": [".port.ts", "-port.ts", "-ports.ts"]
      },
      "groupedArtifactFolders": [
        "use-cases",
        "entities",
        "value-objects",
        "ports",
        "repositories",
        "adapters",
        "controllers",
        "database",
        "workflows",
        "bootstrap"
      ]
    }
  }
}
```

Clean Architecture structure rules are presence-based. OnionCry validates where artifacts belong when they exist, but it does not require empty folders in every context.

The `groupedArtifactFolders` option identifies contextless base artifact folders that should not become flat file lists. A single direct file such as `domain/entities/classification-export.ts` is accepted; two or more direct files in the same grouped artifact folder produce warnings that suggest a `<group>` folder. Direct capability folders under a layer, such as `domain/classification-exports/classification-export.ts`, `application/reviews/get-tax-review.ts`, or `infra/reviews/drizzle-tax-review-repository.ts`, also produce warnings when the configured layer artifact folders are `entities`, `value-objects`, `ports`, `use-cases`, `repositories`, `adapters`, or `bootstrap`. If a capability folder contains one source file, OnionCry suggests moving that file directly under the artifact folder; if it contains multiple source files, OnionCry suggests preserving the capability folder under the artifact folder.

The generated Clean Architecture starter config sets `cleanarch/artifact-placement` to `warn`. Existing projects can expose misplaced use cases, ports, repositories, adapters, entities, and value objects before making the rule a blocking gate.

The `.service.ts` suffix is ambiguous in Clean Architecture. OnionCry must use folder placement to decide whether the file is a domain service, application service, or infra service.

## Vertical Slice

Vertical Slice mode uses `features/<domain>/<operation>` by default:

```txt
src/
  features/<domain>/<operation>/
    index.ts
    contracts/
    handlers/
    adapters/
    domain/
    __tests__/
```

The default public surface is the slice root `index.ts` and `contracts/`. Other slice files are internal unless the project configures them as public.

The default mode options are:

```jsonc
{
  "architecture": {
    "mode": "verticalSlice",
    "verticalSlice": {
      "sliceRoot": "features",
      "sliceDepth": 2,
      "publicSurface": ["index.ts", "contracts"],
      "artifactFolders": ["handlers", "adapters", "domain", "__tests__"],
      "artifactSuffixes": {
        "repository": [".repository.ts"],
        "service": [".service.ts"],
        "handler": [".handler.ts"],
        "adapter": [".adapter.ts"],
        "entity": [".entity.ts"],
        "valueObject": [".value-object.ts"],
        "useCase": [".use-case.ts"]
      },
      "allowedGlobalFolders": ["app", "config", "lib", "shared", "platform"],
      "entryPointNames": ["setup", "Setup", "map", "Map", "register", "Register"],
      "sharedLayerFolders": ["controllers", "handlers", "services", "repositories", "use-cases"]
    }
  }
}
```

Projects can set `sliceRoot` to `slices`, `modules`, or `.`. Projects that prefer `features/<feature>` or root-level feature folders should set `sliceDepth` to `1`; root-level slices are explicit because they are more ambiguous to classify.

In Vertical Slice mode, `.service.ts`, `.repository.ts`, `.handler.ts`, and similar artifact suffixes are slice-internal details by default. Other slices must import through the configured public surface.

Global `domain`, `application`, or `infra` folders are not invalid by
themselves in Vertical Slice mode. The
`verticalslice/no-global-slice-artifacts` rule can warn when files that look
like slice artifacts live outside the configured slice root. The stricter
`verticalslice/no-shared-layer-artifacts` rule reports global technical layers
such as `repositories`, `services`, `handlers`, or `use-cases` even when they
appear under configured global folders. The `verticalslice/slice-entry-point`
rule reports slices that do not expose a configured entry point such as
`setup`, `Map`, or `register`.

## Choosing a mode

Use Clean Architecture when the project needs protected domain rules, long-lived workflows, stable dependency direction, or context ownership boundaries.

Use Vertical Slice when the project needs feature-local code, fast delivery of independent features, and minimal cross-slice coupling.

Do not use both modes in one OnionCry configuration. If a project contains both styles during migration, select the target mode and use rule severities or overrides to manage the transition.
