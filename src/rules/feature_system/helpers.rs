use crate::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub(super) fn render_feature_template(template: &str, domain: &str) -> String {
    template
        .replace("{domain}", domain)
        .replace("{domainCamel}", &domain_camel_case(domain))
        .replace("{DomainPascal}", &domain_pascal_case(domain))
}

fn domain_camel_case(domain: &str) -> String {
    let pascal = domain_pascal_case(domain);
    let mut characters = pascal.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    format!(
        "{}{}",
        first.to_ascii_lowercase(),
        characters.collect::<String>()
    )
}

fn domain_pascal_case(domain: &str) -> String {
    domain
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            let Some(first) = characters.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), characters.as_str())
        })
        .collect()
}

pub(super) fn source_exports_value(source: &str, name: &str) -> bool {
    source.contains(&format!("export const {name}"))
        || source.contains(&format!("export let {name}"))
        || source.contains(&format!("export var {name}"))
        || (source.contains(&format!("const {name}"))
            && source.contains(&format!("export {{ {name} }}")))
}

pub(super) fn source_exports_error_class(source: &str, name: &str) -> bool {
    (source.contains(&format!("export class {name}")) && source.contains("extends Error"))
        || (source.contains(&format!("class {name}"))
            && source.contains("extends Error")
            && source.contains(&format!("export {{ {name} }}")))
}

pub(super) fn source_has_configured_read_call(source: &str, http_client_names: &[String]) -> bool {
    http_client_names.iter().any(|client| {
        if client == "fetch" {
            source.contains("fetch(")
        } else {
            source.contains(&format!("{client}("))
        }
    })
}

pub(super) fn source_accepts_and_passes_abort_signal(source: &str) -> bool {
    source.contains("AbortSignal") && source.matches("signal").count() >= 2
}

pub(super) fn read_source_files(files: &[PathBuf]) -> Result<BTreeMap<PathBuf, String>> {
    let mut sources = BTreeMap::new();
    for file in files {
        if !is_source_file(file) {
            continue;
        }
        let source = fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
            path: file.clone(),
            source,
        })?;
        sources.insert(file.clone(), source);
    }
    Ok(sources)
}

pub(super) fn source_has_query_ownership(source: &str) -> bool {
    source_has_query_hook_read(source)
        || source.contains("prefetchQuery(")
        || source.contains("fetchQuery(")
        || source.contains("ensureQueryData(")
        || source.contains("getQueryData(")
}

pub(super) fn source_uses_query_options_surface(source: &str) -> bool {
    source_has_query_hook_read(source)
        || source.contains("prefetchQuery(")
        || source.contains("fetchQuery(")
        || source.contains("ensureQueryData(")
        || source.contains("getQueryData(")
}

pub(super) fn source_has_query_hook_read(source: &str) -> bool {
    source.contains("useQuery(")
        || source.contains("useSuspenseQuery(")
        || source.contains("useInfiniteQuery(")
}

pub(super) fn source_imports_and_uses_query_options(source: &str) -> bool {
    source.contains("@tanstack/react-query")
        && source.contains("queryOptions")
        && source.contains("queryOptions(")
}

pub(super) fn source_declares_query_key(source: &str) -> bool {
    source.contains("queryKey")
}

pub(super) fn source_passes_query_signal(source: &str) -> bool {
    source.contains("queryFn") && source.matches("signal").count() >= 2
}

pub(super) fn source_has_mutation_invalidation(source: &str) -> bool {
    source.contains("useMutation(")
        && (source.contains("onSuccess") || source.contains("onSettled"))
        && source.contains("invalidateQueries")
}

pub(super) fn source_has_optimistic_update_contract(source: &str) -> bool {
    source.contains("cancelQueries")
        && (source.contains("previous")
            || source.contains("snapshot")
            || source.contains("rollback"))
        && (source.contains("onSettled") || source.contains("invalidateQueries"))
}
