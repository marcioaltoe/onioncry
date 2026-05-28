# Delegate generic JavaScript smells to JavaScript linters

OnionCry checks architectural boundaries that require project-specific layer, context, public-surface, and ownership configuration. Generic JavaScript and TypeScript code-smell rules such as unresolved imports, file-level import cycles, maximum lines, maximum parameters, warning comments, and restricted imports belong to Oxlint, Biome, TypeScript, or equivalent JavaScript linters.

OnionCry may still build an import graph and expose unresolved local imports in `explain` output because architecture rules need resolution metadata. It should not report generic unresolved-import or file-cycle diagnostics as first-class rules. Architecture-specific variants remain in scope, including context cycles, public-surface leaks, framework dependencies from core layers, outer data formats in core layers, unowned schema imports, concrete dependencies from core layers, feature envy across contexts, and shotgun-surgery history analysis.
