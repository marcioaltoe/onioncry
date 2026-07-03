use crate::*;
use globset::GlobSet;
use std::collections::{BTreeMap, HashSet};
use std::path::{Component, Path, PathBuf};

pub(crate) struct LayerClassifier {
    project_root: PathBuf,
    layers: Vec<CompiledLayer>,
}

struct CompiledLayer {
    name: String,
    patterns: GlobSet,
    may_import: HashSet<String>,
}

pub(crate) enum LayerClassification<'a> {
    Classified(&'a str),
    Unclassified,
    Ambiguous(Vec<String>),
}

pub(crate) struct ContextClassifier {
    project_root: PathBuf,
    contexts: Vec<CompiledContext>,
}

struct CompiledContext {
    name: String,
    patterns: GlobSet,
}

pub(crate) enum ContextClassification<'a> {
    Classified(&'a str),
    Contextless,
    Ambiguous(Vec<String>),
}

pub(crate) struct ContextPolicy {
    pub(crate) allow_same_context: bool,
    pub(crate) allow_cross_context: HashSet<String>,
}

impl ContextClassifier {
    pub(crate) fn new(
        project_root: &Path,
        context_configs: &BTreeMap<String, ContextConfig>,
    ) -> Result<Self> {
        let mut contexts = Vec::new();
        for (name, config) in context_configs {
            contexts.push(CompiledContext {
                name: name.clone(),
                patterns: build_glob_set(&config.patterns)?,
            });
        }
        Ok(Self {
            project_root: project_root.to_path_buf(),
            contexts,
        })
    }

    pub(crate) fn classify(&self, file: &Path) -> ContextClassification<'_> {
        let relative_path = file.strip_prefix(&self.project_root).unwrap_or(file);
        let matched = self
            .contexts
            .iter()
            .filter(|context| context.patterns.is_match(relative_path))
            .collect::<Vec<_>>();

        match matched.as_slice() {
            [] => ContextClassification::Contextless,
            [context] => ContextClassification::Classified(&context.name),
            contexts => ContextClassification::Ambiguous(
                contexts
                    .iter()
                    .map(|context| context.name.clone())
                    .collect(),
            ),
        }
    }
}

impl ContextPolicy {
    pub(crate) fn from(config: &ContextRulesConfig) -> Self {
        Self {
            allow_same_context: config.default.allow_same_context,
            allow_cross_context: config.default.allow_cross_context.iter().cloned().collect(),
        }
    }

    pub(crate) fn is_public_surface(&self, target: &Path, project_root: &Path) -> bool {
        let relative_path = target.strip_prefix(project_root).unwrap_or(target);
        relative_path.components().any(|component| {
            let Component::Normal(segment) = component else {
                return false;
            };
            segment
                .to_str()
                .is_some_and(|segment| self.allow_cross_context.contains(segment))
        })
    }
}

impl LayerClassifier {
    pub(crate) fn new(
        project_root: &Path,
        layer_configs: &BTreeMap<String, LayerConfig>,
    ) -> Result<Self> {
        let mut layers = Vec::new();
        for (name, config) in layer_configs {
            let patterns = build_glob_set(&config.patterns)?;
            layers.push(CompiledLayer {
                name: name.clone(),
                patterns,
                may_import: config.may_import.iter().cloned().collect(),
            });
        }
        Ok(Self {
            project_root: project_root.to_path_buf(),
            layers,
        })
    }

    pub(crate) fn classify(&self, file: &Path) -> LayerClassification<'_> {
        let relative_path = file.strip_prefix(&self.project_root).unwrap_or(file);
        let matched = self
            .layers
            .iter()
            .filter(|layer| layer.patterns.is_match(relative_path))
            .collect::<Vec<_>>();

        match matched.as_slice() {
            [] => LayerClassification::Unclassified,
            [layer] => LayerClassification::Classified(&layer.name),
            layers => LayerClassification::Ambiguous(
                layers.iter().map(|layer| layer.name.clone()).collect(),
            ),
        }
    }

    pub(crate) fn may_import(&self, from_layer: &str, to_layer: &str) -> bool {
        self.layers
            .iter()
            .find(|layer| layer.name == from_layer)
            .is_some_and(|layer| layer.may_import.contains(to_layer))
    }
}
