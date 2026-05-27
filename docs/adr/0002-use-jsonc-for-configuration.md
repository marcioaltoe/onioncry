# Use JSONC for configuration

OnionCry uses JSONC as the default configuration format because its rule and override model is closer to a linter than to a deployment manifest. JSONC supports comments, JSON Schema validation, and familiar editor tooling while avoiding YAML type ambiguities. YAML can be reconsidered later as an additional input format, but the first public config contract should be JSONC.
