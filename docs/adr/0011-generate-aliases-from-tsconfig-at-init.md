# Generate aliases from tsconfig at init

OnionCry generates the configuration's `aliases` block from a TypeScript config when a team runs `onioncry init --from-tsconfig [path]`. This removes the most tedious onboarding step in projects that already declare `compilerOptions.paths`, without changing where alias truth lives: check-time alias resolution still reads only the committed OnionCry configuration, and the generated block is meant to be reviewed like the rest of the conservative template.

Generation is intentionally narrow. Wildcard prefix mappings such as `"@/*": ["./src/*"]` translate to prefix aliases normalized relative to the project root. Entries that cannot become prefix aliases — non-wildcard keys, catch-all keys, multiple targets, non-wildcard targets, or targets outside the project root — are skipped and listed in a template comment so the team maps them manually instead of trusting a lossy translation. `extends` is not followed; its presence is called out in the same comment.

Runtime tsconfig inference remains out of scope. Reading tsconfig during `check` would make results depend on a file OnionCry does not own, reintroduce the ambiguity the explicit Alias Mapping decision avoids, and require resolving `extends` chains and project references. A one-time, reviewable generation step captures most of the convenience with none of that coupling.
