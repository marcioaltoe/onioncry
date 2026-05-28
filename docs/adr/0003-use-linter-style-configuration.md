# Use linter-style configuration

OnionCry uses a linter-style configuration model in `.onioncryrc.jsonc`: rules are named with intent-specific namespaces such as `cleanarch/...` and `codesmells/...`, severities are `off`, `warn`, or `error`, and rule options use the familiar `[severity, options]` shape. `overrides` apply rule severity or option changes to matching file globs, with later overrides winning for the same rule. Overrides intentionally do not change the file universe, aliases, layers, or contexts, keeping boundary classification separate from policy exceptions.
