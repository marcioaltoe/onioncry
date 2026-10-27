use crate::{ImportEdge, ImportKind, ImportResolution, OnionCryError, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Class, Declaration, ExportSpecifier, Expression, ImportDeclarationSpecifier,
    ImportOrExportKind, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::{HashMap, HashSet, hash_map::Entry};
use std::fs;
use std::path::{Path, PathBuf};

enum ExportBinding {
    Local(String),
    Remote { module: String, name: String },
}

#[derive(Default)]
struct ModuleSummary {
    error_classes: HashSet<String>,
    imports: HashMap<String, ExportBinding>,
    exports: HashMap<String, ExportBinding>,
}

pub(super) struct ErrorExports<'a> {
    files: HashSet<&'a Path>,
    local_modules: HashMap<(&'a Path, &'a str), &'a Path>,
    summaries: HashMap<PathBuf, ModuleSummary>,
}

impl<'a> ErrorExports<'a> {
    pub(super) fn new(files: &'a [PathBuf], edges: &'a [ImportEdge]) -> Self {
        Self {
            files: files.iter().map(PathBuf::as_path).collect(),
            local_modules: edges
                .iter()
                .filter_map(|edge| {
                    if edge.type_only
                        || !matches!(edge.kind, ImportKind::StaticImport | ImportKind::ReExport)
                    {
                        return None;
                    }
                    let ImportResolution::Local(target) = &edge.resolution else {
                        return None;
                    };
                    Some((
                        (edge.source.as_path(), edge.specifier.as_str()),
                        target.as_path(),
                    ))
                })
                .collect(),
            summaries: HashMap::new(),
        }
    }

    pub(super) fn exports_error_class(&mut self, file: &Path, name: &str) -> Result<bool> {
        let mut file = file.to_path_buf();
        let mut name = name.to_string();
        let mut visited = HashSet::new();
        loop {
            // Guard before reading: a resolved path can still be excluded from analysis.
            if name == "default"
                || !self.files.contains(file.as_path())
                || !visited.insert((file.clone(), name.clone()))
            {
                return Ok(false);
            }
            let summary = match self.summaries.entry(file.clone()) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(summarize_module(&file)?),
            };
            let binding = match summary.exports.get(&name) {
                Some(ExportBinding::Local(local)) => {
                    if summary.error_classes.contains(local) {
                        return Ok(true);
                    }
                    summary.imports.get(local)
                }
                remote => remote,
            };
            let Some(ExportBinding::Remote {
                module,
                name: imported,
            }) = binding
            else {
                return Ok(false);
            };
            let Some(target) = self.local_modules.get(&(file.as_path(), module.as_str())) else {
                return Ok(false);
            };
            name = imported.clone();
            file = target.to_path_buf();
        }
    }
}

fn summarize_module(file: &Path) -> Result<ModuleSummary> {
    let source = fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
        path: file.to_path_buf(),
        source,
    })?;
    let source_type = SourceType::from_path(file).map_err(|source| OnionCryError::ParseSource {
        path: file.to_path_buf(),
        message: source.to_string(),
    })?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    if parsed.fatal_error || parsed.diagnostics.has_errors() {
        return Err(OnionCryError::ParseSource {
            path: file.to_path_buf(),
            message: parsed
                .diagnostics
                .errors()
                .next()
                .map_or_else(|| "parser panicked".to_string(), ToString::to_string),
        });
    }
    let mut summary = ModuleSummary::default();
    if source_type.is_typescript_definition() {
        return Ok(summary);
    }
    let mut shadows_error = false;
    for statement in &parsed.program.body {
        let declaration = match statement {
            Statement::ExportDeclaration(export) => Some(&export.declaration),
            _ => statement.as_declaration(),
        };
        shadows_error |= declaration.is_some_and(declares_error_binding);
        if let Statement::ImportDeclaration(import) = statement {
            shadows_error |= import
                .specifiers
                .iter()
                .flatten()
                .any(|specifier| specifier.local().name == "Error");
        }
        match statement {
            Statement::ClassDeclaration(class) => summary.add_class(class, false),
            Statement::ExportDeclaration(export) => {
                if let Declaration::ClassDeclaration(class) = &export.declaration {
                    summary.add_class(class, true);
                }
            }
            Statement::ExportNamedDeclaration(export)
                if export.export_kind == ImportOrExportKind::Value =>
            {
                summary.add_exports(&export.specifiers, ExportBinding::Local);
            }
            Statement::ExportFromDeclaration(export)
                if export.export_kind == ImportOrExportKind::Value =>
            {
                // Like oxc's module record, skip specifiers that are not valid UTF-8.
                if let Some(module) = export.source.value.as_str() {
                    summary.add_exports(&export.specifiers, |name| ExportBinding::Remote {
                        module: module.to_string(),
                        name,
                    });
                }
            }
            Statement::ImportDeclaration(import)
                if import.import_kind == ImportOrExportKind::Value =>
            {
                let Some(module) = import.source.value.as_str() else {
                    continue;
                };
                for specifier in import.specifiers.iter().flatten() {
                    let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                        continue;
                    };
                    if specifier.import_kind == ImportOrExportKind::Type {
                        continue;
                    }
                    summary.imports.insert(
                        specifier.local.name.to_string(),
                        ExportBinding::Remote {
                            module: module.to_string(),
                            name: specifier.imported.name().to_string(),
                        },
                    );
                }
            }
            _ => {}
        }
    }
    if shadows_error {
        summary.error_classes.clear();
    }
    Ok(summary)
}

fn declares_error_binding(declaration: &Declaration<'_>) -> bool {
    match declaration {
        Declaration::ClassDeclaration(class) => {
            class.id.as_ref().is_some_and(|id| id.name == "Error")
        }
        Declaration::FunctionDeclaration(function) => {
            function.id.as_ref().is_some_and(|id| id.name == "Error")
        }
        Declaration::VariableDeclaration(variables) => {
            variables.declarations.iter().any(|declaration| {
                declaration
                    .id
                    .get_binding_identifiers()
                    .iter()
                    .any(|id| id.name == "Error")
            })
        }
        Declaration::TSEnumDeclaration(enumeration) => enumeration.id.name == "Error",
        Declaration::TSImportEqualsDeclaration(import) => import.id.name == "Error",
        Declaration::TSNamespaceDeclaration(namespace) => {
            // Only the identifier root binds here, including dotted namespaces.
            // Keep ambient/type-only namespace bodies conservative without emit inference.
            namespace.id.name == "Error"
        }
        _ => false,
    }
}

impl ModuleSummary {
    fn add_class(&mut self, class: &Class<'_>, exported: bool) {
        let Some(id) = &class.id else {
            return;
        };
        if !class.declare
            && matches!(
                class.heritage.as_ref().map(|heritage| &heritage.expression),
                Some(Expression::Identifier(base)) if base.name == "Error"
            )
        {
            self.error_classes.insert(id.name.to_string());
        }
        if exported {
            self.exports.insert(
                id.name.to_string(),
                ExportBinding::Local(id.name.to_string()),
            );
        }
    }

    fn add_exports(
        &mut self,
        specifiers: &[ExportSpecifier<'_>],
        binding: impl Fn(String) -> ExportBinding,
    ) {
        for specifier in specifiers {
            if specifier.export_kind == ImportOrExportKind::Type {
                continue;
            }
            self.exports.insert(
                specifier.exported.name().to_string(),
                binding(specifier.local.name().to_string()),
            );
        }
    }
}
