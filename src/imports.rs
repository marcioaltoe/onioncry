use crate::{
    ImportEdge, ImportKind, ImportResolution, LoadedConfig, OnionCryError, Result, is_source_file,
    normalize_path,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, ImportExpression};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct RawImport {
    specifier: String,
    kind: ImportKind,
    type_only: bool,
    span: Span,
}

#[derive(Clone, Debug)]
struct AliasMapping {
    prefix: String,
    target: PathBuf,
}

impl LoadedConfig {
    fn alias_mappings(&self) -> Vec<AliasMapping> {
        let mut aliases = self
            .config
            .aliases
            .iter()
            .filter_map(|(prefix, target)| {
                target.as_str().map(|target| AliasMapping {
                    prefix: prefix.to_string(),
                    target: PathBuf::from(target),
                })
            })
            .collect::<Vec<_>>();
        aliases.sort_by_key(|alias| std::cmp::Reverse(alias.prefix.len()));
        aliases
    }
}

pub fn collect_import_edges(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
) -> Result<Vec<ImportEdge>> {
    let aliases = loaded.alias_mappings();
    let mut edges = Vec::new();

    for file in files {
        if !is_source_file(file) {
            continue;
        }
        let source = fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
            path: file.clone(),
            source,
        })?;
        let raw_imports = scan_imports(file, &source)?;
        for raw_import in raw_imports {
            let resolution = resolve_import(&raw_import.specifier, file, project_root, &aliases);
            let (line, column) = line_column(&source, raw_import.span.start as usize);
            edges.push(ImportEdge {
                source: file.clone(),
                specifier: raw_import.specifier,
                kind: raw_import.kind,
                type_only: raw_import.type_only,
                line,
                column,
                resolution,
            });
        }
    }

    Ok(edges)
}

fn scan_imports(path: &Path, source: &str) -> Result<Vec<RawImport>> {
    let source_type = SourceType::from_path(path).map_err(|source| OnionCryError::ParseSource {
        path: path.to_path_buf(),
        message: source.to_string(),
    })?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        let message = parsed
            .errors
            .first()
            .map_or_else(|| "parser panicked".to_string(), ToString::to_string);
        return Err(OnionCryError::ParseSource {
            path: path.to_path_buf(),
            message,
        });
    }

    let mut imports = Vec::new();
    for (specifier, requested_modules) in &parsed.module_record.requested_modules {
        for requested_module in requested_modules {
            imports.push(RawImport {
                specifier: specifier.to_string(),
                kind: if requested_module.is_import {
                    ImportKind::StaticImport
                } else {
                    ImportKind::ReExport
                },
                type_only: requested_module.is_type,
                span: requested_module.span,
            });
        }
    }

    let mut visitor = RuntimeImportVisitor::default();
    visitor.visit_program(&parsed.program);
    imports.extend(visitor.imports);
    imports.sort_by_key(|import| import.span.start);
    Ok(imports)
}

#[derive(Default)]
struct RuntimeImportVisitor {
    imports: Vec<RawImport>,
}

impl<'a> Visit<'a> for RuntimeImportVisitor {
    fn visit_import_expression(&mut self, import: &ImportExpression<'a>) {
        if let Expression::StringLiteral(source) = &import.source {
            self.imports.push(RawImport {
                specifier: source.value.to_string(),
                kind: ImportKind::DynamicImport,
                type_only: false,
                span: source.span,
            });
        }
        walk::walk_import_expression(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(specifier) = string_literal_require_specifier(call) {
            self.imports.push(RawImport {
                specifier: specifier.value.to_string(),
                kind: ImportKind::Require,
                type_only: false,
                span: specifier.span,
            });
        }
        walk::walk_call_expression(self, call);
    }
}

fn string_literal_require_specifier<'a>(
    call: &'a CallExpression<'a>,
) -> Option<&'a oxc_ast::ast::StringLiteral<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name != "require" || call.arguments.len() != 1 {
        return None;
    }
    let Argument::StringLiteral(specifier) = &call.arguments[0] else {
        return None;
    };
    Some(specifier)
}

fn resolve_import(
    specifier: &str,
    source_file: &Path,
    project_root: &Path,
    aliases: &[AliasMapping],
) -> ImportResolution {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        let Some(source_dir) = source_file.parent() else {
            return ImportResolution::UnresolvedLocal;
        };
        return resolve_local_candidate(&source_dir.join(specifier))
            .map_or(ImportResolution::UnresolvedLocal, ImportResolution::Local);
    }

    for alias in aliases {
        if let Some(remainder) = specifier.strip_prefix(&alias.prefix) {
            let remainder = remainder.strip_prefix('/').unwrap_or(remainder);
            let candidate = project_root.join(&alias.target).join(remainder);
            return resolve_local_candidate(&candidate)
                .map_or(ImportResolution::UnresolvedLocal, ImportResolution::Local);
        }
    }

    ImportResolution::External
}

fn resolve_local_candidate(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_file() {
        return Some(normalize_path(candidate));
    }

    for extension in SOURCE_EXTENSIONS {
        let with_extension = append_extension(candidate, extension);
        if with_extension.is_file() {
            return Some(normalize_path(&with_extension));
        }
    }

    for extension in SOURCE_EXTENSIONS {
        let index_path = candidate.join(format!("index.{extension}"));
        if index_path.is_file() {
            return Some(normalize_path(&index_path));
        }
    }

    None
}

fn append_extension(path: &Path, extension: &str) -> PathBuf {
    let mut path_with_extension = path.as_os_str().to_os_string();
    path_with_extension.push(".");
    path_with_extension.push(extension);
    PathBuf::from(path_with_extension)
}

pub(crate) fn line_column(source: &str, byte_index: usize) -> (usize, usize) {
    let safe_index = byte_index.min(source.len());
    let prefix = &source[..safe_index];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, current_line)| current_line)
        .chars()
        .count()
        + 1;
    (line, column)
}

const SOURCE_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];
