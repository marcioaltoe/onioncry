pub const CONFIG_SCHEMA_URL: &str = "https://raw.githubusercontent.com/marcioaltoe/onioncry/main/docs/schema/onioncryrc.schema.json";

// `init --from-tsconfig` replaces exactly this block with generated aliases.
pub(crate) const TEMPLATE_ALIAS_BLOCK: &str = r#"  // TODO: map import aliases used by your project.
  "aliases": {
    "@app/": "src/"
  },"#;

pub(crate) const INIT_CONFIG_TEMPLATE: &str = r#"{
  "$schema": "https://raw.githubusercontent.com/marcioaltoe/onioncry/main/docs/schema/onioncryrc.schema.json",
  "version": 1,
  "project": {
    "root": ".",
    // TODO: adjust the file universe for your source layout.
    "include": ["src/**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs}"],
    "exclude": ["node_modules/**", "dist/**", "build/**", "coverage/**"]
  },
  "architecture": {
    "mode": "cleanArchitecture",
    "cleanArchitecture": {
      "contextRoot": "contexts",
      "layerPathAliases": {
        "domain": ["domain"],
        "application": ["application"],
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
        "adapter": [
          ".adapter.ts",
          "-adapter.ts",
          ".gateway.ts",
          "-gateway.ts",
          "/client.ts",
          ".client.ts",
          "-client.ts",
          "/handler.ts",
          ".mapper.ts",
          "-mapper.ts",
          "-mappers.ts",
          ".parser.ts",
          "-parser.ts",
          ".provider.ts",
          "-provider.ts",
          ".request.ts",
          "-request.ts",
          "-requests.ts",
          ".schema.ts",
          "-schema.ts",
          "-schemas.ts",
          "-normalization.ts",
          "-resilience.ts",
          "-composition.ts",
          "-scenario.ts",
          "-scenarios.ts",
          "-snapshot.ts",
          "-snapshots.ts"
        ],
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
    },
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
  },
  // TODO: map import aliases used by your project.
  "aliases": {
    "@app/": "src/"
  },
  "layers": {
    "domain": {
      // TODO: point this at your core business model.
      "patterns": ["src/domain/**"],
      "mayImport": ["domain", "shared"]
    },
    "application": {
      // TODO: point this at your use cases and application services.
      "patterns": ["src/application/**"],
      "mayImport": ["application", "domain", "shared"]
    },
    "infra": {
      // TODO: point this at adapters, frameworks, drivers, and runtime details.
      "patterns": ["src/infra/**"],
      "mayImport": ["infra", "application", "domain", "shared"]
    },
    "shared": {
      // TODO: keep shared small and stable.
      "patterns": ["src/shared/**"],
      "mayImport": ["shared"]
    }
  },
  "contexts": {
    // TODO: replace these with your bounded contexts.
    "sales": {
      "patterns": ["src/sales/**"]
    },
    "billing": {
      "patterns": ["src/billing/**"]
    }
  },
  "contextRules": {
    "default": {
      "allowSameContext": true,
      "allowCrossContext": ["contracts", "events", "ports", "shared"]
    }
  },
  "rules": {
    "cleanarch/no-layer-leak": "error",
    "cleanarch/no-cross-context-internal-import": "error",
    "cleanarch/no-forbidden-imports": ["error", {
      "layers": [
        {
          "fromLayer": "domain",
          "severity": "error",
          // TODO: allow only domain-safe packages.
          "allow": ["uuid"]
        },
        {
          "fromLayer": "application",
          "severity": "warn",
          // TODO: allow orchestration packages when they are intentional.
          "allow": []
        },
        {
          "fromLayer": "infra",
          "severity": "off",
          // TODO: infra is open by default; tighten this when useful.
          "allow": []
        }
      ]
    }],
    "cleanarch/no-framework-in-core": "warn",
    "cleanarch/no-outer-data-format-in-core": "warn",
    "cleanarch/no-public-surface-internal-reexport": "warn",
    "cleanarch/no-context-cycle": "warn",
    "cleanarch/no-unowned-schema-import": "warn",
    "cleanarch/artifact-placement": "warn",
    "solid/no-concrete-dependency": "warn",
    "codesmells/feature-envy": "warn",
    "codesmells/shotgun-surgery": "off",
    "cleanarch/unclassified-file": "warn"
    // If you switch architecture.mode to "verticalSlice", remove cleanarch/* rules and enable:
    // "verticalslice/no-cross-slice-internal-import": "warn",
    // "verticalslice/no-global-slice-artifacts": "warn",
    // "verticalslice/slice-entry-point": "warn",
    // "verticalslice/no-shared-layer-artifacts": "warn"
  },
  // TODO: use overrides for temporary policy exceptions, not file selection.
  "overrides": []
}
"#;
