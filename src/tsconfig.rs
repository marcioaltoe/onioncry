use crate::{OnionCryError, Result, normalize_path, resolve_against};
use jsonc_parser::{ParseOptions, parse_to_serde_value};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Tsconfig {
    #[serde(default)]
    extends: Option<serde_json::Value>,
    #[serde(default)]
    compiler_options: Option<TsconfigCompilerOptions>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TsconfigCompilerOptions {
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    paths: Option<BTreeMap<String, Vec<String>>>,
}

#[derive(Debug)]
pub struct TsconfigAliases {
    pub source_display: String,
    pub aliases: BTreeMap<String, String>,
    pub skipped: Vec<SkippedTsconfigPath>,
    pub extends_present: bool,
    pub paths_present: bool,
}

#[derive(Debug)]
pub struct SkippedTsconfigPath {
    pub key: String,
    pub reason: &'static str,
}

pub fn load_tsconfig_aliases(
    cwd: &Path,
    project_root: &Path,
    tsconfig_arg: &Path,
) -> Result<TsconfigAliases> {
    let path = normalize_path(&resolve_against(cwd, tsconfig_arg));
    if !path.is_file() {
        return Err(OnionCryError::MissingTsconfig { path });
    }

    let contents = fs::read_to_string(&path).map_err(|source| OnionCryError::ReadTsconfig {
        path: path.clone(),
        source,
    })?;
    let tsconfig = parse_to_serde_value::<Tsconfig>(&contents, &ParseOptions::default()).map_err(
        |source| OnionCryError::ParseTsconfig {
            path: path.clone(),
            message: source.to_string(),
        },
    )?;

    let tsconfig_dir = path
        .parent()
        .map_or_else(|| cwd.to_path_buf(), Path::to_path_buf);
    let compiler_options = tsconfig
        .compiler_options
        .unwrap_or(TsconfigCompilerOptions {
            base_url: None,
            paths: None,
        });
    let base_dir =
        normalize_path(&tsconfig_dir.join(compiler_options.base_url.as_deref().unwrap_or(".")));
    let project_root = normalize_path(project_root);
    let paths = compiler_options.paths.unwrap_or_default();
    let paths_present = !paths.is_empty();
    let mut aliases = BTreeMap::new();
    let mut skipped = Vec::new();

    for (key, targets) in &paths {
        match alias_for_paths_entry(&base_dir, &project_root, key, targets) {
            Ok((prefix, target)) => {
                aliases.insert(prefix, target);
            }
            Err(reason) => skipped.push(SkippedTsconfigPath {
                key: key.clone(),
                reason,
            }),
        }
    }

    Ok(TsconfigAliases {
        source_display: tsconfig_arg.display().to_string(),
        aliases,
        skipped,
        extends_present: tsconfig.extends.is_some(),
        paths_present,
    })
}

fn alias_for_paths_entry(
    base_dir: &Path,
    project_root: &Path,
    key: &str,
    targets: &[String],
) -> std::result::Result<(String, String), &'static str> {
    let Some(prefix) = key.strip_suffix('*') else {
        return Err("non-wildcard key");
    };
    if prefix.is_empty() {
        return Err("catch-all key");
    }
    let [target] = targets else {
        return Err("multiple targets");
    };
    let Some(target_prefix) = target.strip_suffix('*') else {
        return Err("non-wildcard target");
    };

    let absolute_target = normalize_path(&base_dir.join(target_prefix));
    let Ok(relative_target) = absolute_target.strip_prefix(project_root) else {
        return Err("target outside the project root");
    };
    let mut value = forward_slash_display(relative_target);
    if target_prefix.ends_with('/') && !value.is_empty() {
        value.push('/');
    }

    Ok((prefix.to_string(), value))
}

pub fn render_generated_alias_block(generated: &TsconfigAliases) -> String {
    let mut block = format!(
        "  // Aliases generated from {} for review; runtime resolution reads only this file.\n",
        generated.source_display
    );
    if generated.extends_present {
        block.push_str("  // Note: this tsconfig uses \"extends\", which init does not follow.\n");
    }
    if !generated.paths_present {
        block.push_str("  // No compilerOptions.paths entries were found.\n");
    }
    if !generated.skipped.is_empty() {
        block.push_str("  // Skipped tsconfig paths entries (map these manually):\n");
        for entry in &generated.skipped {
            block.push_str(&format!(
                "  //   {}: {}\n",
                json_string(&entry.key),
                entry.reason
            ));
        }
    }

    block.push_str("  \"aliases\": {");
    let mut entries = generated.aliases.iter().peekable();
    if entries.peek().is_some() {
        block.push('\n');
        while let Some((prefix, target)) = entries.next() {
            block.push_str(&format!(
                "    {}: {}",
                json_string(prefix),
                json_string(target)
            ));
            if entries.peek().is_some() {
                block.push(',');
            }
            block.push('\n');
        }
        block.push_str("  },");
    } else {
        block.push_str("},");
    }
    block
}

fn forward_slash_display(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(segment) => segment.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| format!("{value:?}"))
}
