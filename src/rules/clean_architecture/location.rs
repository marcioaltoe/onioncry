use crate::path_has_prefix_components;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub(super) struct CleanArtifactLocationPolicy {
    source_prefix: Vec<String>,
    context_root: Vec<String>,
    layer_aliases: BTreeMap<String, HashSet<String>>,
    folder_layers: BTreeMap<String, BTreeSet<String>>,
}

impl CleanArtifactLocationPolicy {
    pub(super) fn new(
        source_prefix: Vec<String>,
        context_root: Vec<String>,
        layer_aliases: BTreeMap<String, HashSet<String>>,
        folder_layers: BTreeMap<String, BTreeSet<String>>,
    ) -> Self {
        Self {
            source_prefix,
            context_root,
            layer_aliases,
            folder_layers,
        }
    }

    pub(super) fn context_first_location(&self, components: &[String]) -> Option<(String, String)> {
        let context_index = self.context_root.len();
        let layer_index = context_index + 1;
        if !path_has_prefix_components(components, &self.context_root)
            || components.len() <= layer_index
        {
            return None;
        }
        let layer = self.layer_for_segment(&components[layer_index])?;
        Some((components[context_index].clone(), layer))
    }

    pub(super) fn context_missing_layer(&self, components: &[String]) -> Option<String> {
        let context_index = self.context_root.len();
        let next_index = context_index + 1;
        if !path_has_prefix_components(components, &self.context_root)
            || components.len() <= next_index
        {
            return None;
        }
        let next = &components[next_index];
        (self.layer_for_segment(next).is_none() && self.folder_layers.contains_key(next))
            .then(|| components[context_index].clone())
    }

    pub(super) fn first_layer(&self, components: &[String]) -> Option<String> {
        self.first_layer_with_index(components)
            .map(|(layer, _)| layer)
    }

    pub(super) fn first_layer_with_index(&self, components: &[String]) -> Option<(String, usize)> {
        components
            .iter()
            .enumerate()
            .find_map(|(index, component)| {
                self.layer_for_segment(component)
                    .map(|layer| (layer, index))
            })
    }

    pub(super) fn layer_for_segment(&self, segment: &str) -> Option<String> {
        self.layer_aliases
            .iter()
            .find_map(|(layer, aliases)| aliases.contains(segment).then(|| layer.clone()))
    }

    pub(super) fn is_contextless_base_layer(
        &self,
        components: &[String],
        layer_index: usize,
    ) -> bool {
        layer_index == self.source_prefix_len(components)
    }

    pub(super) fn context_before_layer<'a>(
        &self,
        components: &'a [String],
        layer_index: usize,
    ) -> Option<&'a str> {
        let context_index = self.source_prefix_len(components);
        (layer_index > context_index)
            .then(|| components.get(context_index).map(String::as_str))
            .flatten()
    }

    pub(super) fn context_after_source_prefix<'a>(
        &self,
        components: &'a [String],
    ) -> Option<&'a str> {
        components
            .get(self.source_prefix_len(components))
            .map(String::as_str)
    }

    pub(super) fn layer_first_context_candidate<'a>(
        &self,
        components: &'a [String],
        layer_index: usize,
        folder_index: Option<usize>,
    ) -> Option<&'a str> {
        if folder_index == Some(layer_index + 1) {
            return None;
        }
        let context_index = layer_index + 1;
        if components.len() <= context_index + 1 {
            return None;
        }
        let candidate = components[context_index].as_str();
        if self.layer_for_segment(candidate).is_some() || self.folder_layers.contains_key(candidate)
        {
            return None;
        }
        Some(candidate)
    }

    pub(super) fn is_layer_or_artifact_segment(&self, segment: &str) -> bool {
        self.layer_for_segment(segment).is_some() || self.folder_layers.contains_key(segment)
    }

    fn source_prefix_len(&self, components: &[String]) -> usize {
        if path_has_prefix_components(components, &self.source_prefix) {
            self.source_prefix.len()
        } else {
            0
        }
    }
}
