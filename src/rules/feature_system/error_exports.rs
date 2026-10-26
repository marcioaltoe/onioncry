use crate::{ImportEdge, ImportKind, ImportResolution, OnionCryError, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Class, Declaration, Expression, ImportDeclarationSpecifier, ImportOrExportKind, Statement,
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
    if parsed.panicked || !parsed.errors.is_empty() {
        return Err(OnionCryError::ParseSource {
            path: file.to_path_buf(),
            message: parsed
                .errors
                .first()
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
            Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
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
            Statement::ExportNamedDeclaration(export)
                if export.export_kind == ImportOrExportKind::Value =>
            {
                if let Some(Declaration::ClassDeclaration(class)) = &export.declaration {
                    summary.add_class(class, true);
                }
                for specifier in &export.specifiers {
                    if specifier.export_kind == ImportOrExportKind::Type {
                        continue;
                    }
                    let name = specifier.local.name().to_string();
                    let binding = export.source.as_ref().map_or_else(
                        || ExportBinding::Local(name.clone()),
                        |source| ExportBinding::Remote {
                            module: source.value.to_string(),
                            name: name.clone(),
                        },
                    );
                    summary
                        .exports
                        .insert(specifier.exported.name().to_string(), binding);
                }
            }
            Statement::ImportDeclaration(import)
                if import.import_kind == ImportOrExportKind::Value =>
            {
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
                            module: import.source.value.to_string(),
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
        _ => false,
    }
}

impl ModuleSummary {
    fn add_class(&mut self, class: &Class<'_>, exported: bool) {
        let Some(id) = &class.id else {
            return;
        };
        if !class.declare
            && matches!(&class.super_class, Some(Expression::Identifier(base)) if base.name == "Error")
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
}
