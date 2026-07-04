use crate::{DEFAULT_TEST_FILE_SUFFIXES, OnionCryError, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::path::Component;
use std::path::{Path, PathBuf};

pub(crate) fn path_has_any_segment(
    path: &Path,
    project_root: &Path,
    segments: &HashSet<String>,
) -> bool {
    let relative_path = path.strip_prefix(project_root).unwrap_or(path);
    relative_path.components().any(|component| {
        let Component::Normal(segment) = component else {
            return false;
        };
        segment.to_str().is_some_and(|segment| {
            segments.contains(segment) || segments.contains(&segment.to_ascii_lowercase())
        })
    })
}

pub(crate) fn path_ends_with_any(path: &Path, project_root: &Path, suffixes: &[String]) -> bool {
    let relative_path = path
        .strip_prefix(project_root)
        .unwrap_or(path)
        .display()
        .to_string()
        .to_ascii_lowercase();
    suffixes
        .iter()
        .any(|suffix| relative_path.ends_with(&suffix.to_ascii_lowercase()))
}

pub(crate) fn normalized_package_name(specifier: &str) -> String {
    if specifier.starts_with('@') {
        let mut segments = specifier.split('/');
        let Some(scope) = segments.next() else {
            return specifier.to_string();
        };
        let Some(name) = segments.next() else {
            return specifier.to_string();
        };
        return format!("{scope}/{name}");
    }

    specifier
        .split('/')
        .next()
        .map_or_else(|| specifier.to_string(), str::to_string)
}

pub(crate) fn project_relative_display(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub(crate) fn project_relative_components(project_root: &Path, path: &Path) -> Vec<String> {
    path_components(path.strip_prefix(project_root).unwrap_or(path))
}

pub(crate) fn path_roots(roots: Vec<String>) -> Vec<Vec<String>> {
    roots
        .iter()
        .map(|root| path_components(Path::new(root)))
        .collect()
}

pub(crate) fn path_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| {
            let Component::Normal(segment) = component else {
                return None;
            };
            segment.to_str().map(str::to_string)
        })
        .collect()
}

pub(crate) fn path_from_components(components: &[String]) -> PathBuf {
    components.iter().collect()
}

pub(crate) fn path_has_prefix_components(components: &[String], root: &[String]) -> bool {
    components.len() >= root.len()
        && components
            .iter()
            .zip(root.iter())
            .all(|(component, root_component)| component == root_component)
}

pub(crate) fn path_under_any_root(components: &[String], roots: &[Vec<String>]) -> bool {
    roots
        .iter()
        .any(|root| path_has_prefix_components(components, root))
}

pub(crate) fn display_path_components(components: &[String]) -> String {
    if components.is_empty() {
        ".".to_string()
    } else {
        components.join("/")
    }
}

pub(crate) fn is_index_file_name(file_name: &str) -> bool {
    matches!(
        file_name,
        "index.ts" | "index.tsx" | "index.js" | "index.jsx" | "index.mts" | "index.cts"
    )
}

pub(crate) fn is_test_file_name(file_name: &str) -> bool {
    DEFAULT_TEST_FILE_SUFFIXES
        .iter()
        .any(|suffix| file_name.ends_with(suffix))
}

pub(crate) fn sorted_strings(values: &HashSet<String>) -> Vec<String> {
    let mut values = values.iter().cloned().collect::<Vec<_>>();
    values.sort();
    values
}

pub(crate) fn display_root(roots: &[Vec<String>]) -> String {
    roots.first().map_or_else(
        || ".".to_string(),
        |root| {
            if root.is_empty() {
                ".".to_string()
            } else {
                root.join("/")
            }
        },
    )
}

pub(crate) fn artifact_role_folder(role: &str) -> String {
    match role {
        "repository" => "repositories",
        "service" => "services",
        "useCase" => "use-cases",
        "entity" => "entities",
        "valueObject" => "value-objects",
        "adapter" => "adapters",
        "handler" => "handlers",
        "port" => "ports",
        other => other,
    }
    .to_string()
}

pub(crate) fn is_core_clean_artifact_role(role: &str) -> bool {
    matches!(role, "use-cases" | "entities" | "value-objects" | "ports")
}

pub(crate) fn is_kebab_case_file_name(file_name: &str) -> bool {
    source_file_stem(file_name)
        .split('.')
        .all(is_kebab_case_name)
}

pub(crate) fn is_kebab_case_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return false;
    }
    name.bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub(crate) fn source_file_stem(file_name: &str) -> &str {
    SOURCE_EXTENSIONS
        .iter()
        .find_map(|extension| file_name.strip_suffix(&format!(".{extension}")))
        .unwrap_or(file_name)
}

pub(crate) fn stem_matches_collection_suffix(stem: &str, suffixes: &[String]) -> bool {
    let stem = stem
        .strip_suffix(".test")
        .or_else(|| stem.strip_suffix(".spec"))
        .unwrap_or(stem);

    suffixes.iter().any(|suffix| stem.ends_with(suffix))
}

pub(crate) fn singular_directory_name(directory: &str) -> String {
    if let Some(stem) = directory.strip_suffix("ies") {
        return format!("{stem}y");
    }
    directory
        .strip_suffix('s')
        .map_or_else(|| directory.to_string(), str::to_string)
}

pub(crate) fn plural_like(segment: &str) -> bool {
    segment.ends_with('s') && segment != "shared"
}

pub(crate) fn is_source_file(file: &Path) -> bool {
    file.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| SOURCE_EXTENSIONS.contains(&extension))
}

pub(crate) fn has_wildcard_reexport(source: &str) -> bool {
    source.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("export * from")
            || line.starts_with("export * as ")
            || line.starts_with("export type * from")
            || line.starts_with("export type * as ")
    })
}

pub(crate) fn is_component_source_file(file: &Path) -> bool {
    file.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "tsx" | "jsx"))
}

const SOURCE_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

pub(crate) fn resolve_against(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

pub(crate) fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(
            Glob::new(pattern).map_err(|source| OnionCryError::InvalidGlob {
                pattern: pattern.clone(),
                source,
            })?,
        );
    }
    builder
        .build()
        .map_err(|source| OnionCryError::InvalidGlob {
            pattern: patterns.join(", "),
            source,
        })
}
