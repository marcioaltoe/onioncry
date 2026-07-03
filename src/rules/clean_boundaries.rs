use crate::*;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

pub(crate) fn collect_layer_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.layers.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = LayerClassifier::new(project_root, &loaded.config.layers)?;
    let mut violations = Vec::new();

    for file in files {
        match classifier.classify(file) {
            LayerClassification::Classified(_) => {}
            LayerClassification::Unclassified => {
                let unclassified_severity =
                    rule_policy.effective_severity(RULE_UNCLASSIFIED_FILE, project_root, file);
                if unclassified_severity == Severity::Off {
                    continue;
                }
                violations.push(Violation::unclassified_file(file, unclassified_severity));
            }
            LayerClassification::Ambiguous(layers) => {
                let ambiguous_severity =
                    rule_policy.effective_severity(RULE_AMBIGUOUS_LAYER, project_root, file);
                if ambiguous_severity == Severity::Off {
                    continue;
                }
                violations.push(Violation::ambiguous_layer(file, layers, ambiguous_severity));
            }
        }
    }

    for edge in edges {
        let layer_leak_severity =
            rule_policy.effective_severity(RULE_NO_LAYER_LEAK, project_root, &edge.source);
        if layer_leak_severity == Severity::Off {
            continue;
        }
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        let LayerClassification::Classified(to_layer) = classifier.classify(target) else {
            continue;
        };
        if classifier.may_import(from_layer, to_layer) {
            continue;
        }
        violations.push(Violation::layer_leak(
            edge,
            target,
            from_layer,
            to_layer,
            layer_leak_severity,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_external_package_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.layers.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = LayerClassifier::new(project_root, &loaded.config.layers)?;
    let mut violations = Vec::new();

    for edge in edges {
        if !matches!(edge.resolution, ImportResolution::External) {
            continue;
        }
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        let rule_setting =
            rule_policy.effective_rule(RULE_NO_FORBIDDEN_IMPORTS, project_root, &edge.source);
        let package_policy = ExternalPackagePolicy::from_rule_setting(&rule_setting)?;
        let layer_policy = package_policy.for_layer(from_layer);
        if layer_policy.severity == Severity::Off {
            continue;
        }

        let package_name = normalized_package_name(&edge.specifier);
        if layer_policy.allow.is_allowed(&package_name) {
            continue;
        }

        violations.push(Violation::forbidden_external_package(
            edge,
            from_layer,
            &package_name,
            layer_policy.severity,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_context_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[PathBuf],
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let context_policy = ContextPolicy::from(&loaded.config.context_rules);
    let mut violations = Vec::new();

    for file in files {
        if let ContextClassification::Ambiguous(contexts) = classifier.classify(file) {
            let severity =
                rule_policy.effective_severity(RULE_AMBIGUOUS_CONTEXT, project_root, file);
            if severity != Severity::Off {
                violations.push(Violation::ambiguous_context(file, contexts, severity));
            }
        }
    }

    for edge in edges {
        let severity = rule_policy.effective_severity(
            RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let ContextClassification::Classified(from_context) = classifier.classify(&edge.source)
        else {
            continue;
        };
        let ContextClassification::Classified(to_context) = classifier.classify(target) else {
            continue;
        };
        if from_context == to_context && context_policy.allow_same_context {
            continue;
        }
        if from_context != to_context && context_policy.is_public_surface(target, project_root) {
            continue;
        }

        violations.push(Violation::cross_context_internal_import(
            edge,
            target,
            from_context,
            to_context,
            severity,
            &context_policy.allow_cross_context,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_framework_in_core_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.layers.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = LayerClassifier::new(project_root, &loaded.config.layers)?;
    let mut violations = Vec::new();

    for edge in edges {
        if !matches!(edge.resolution, ImportResolution::External) {
            continue;
        }
        let severity =
            rule_policy.effective_severity(RULE_NO_FRAMEWORK_IN_CORE, project_root, &edge.source);
        if severity == Severity::Off {
            continue;
        };
        let rule_setting =
            rule_policy.effective_rule(RULE_NO_FRAMEWORK_IN_CORE, project_root, &edge.source);
        let core_layers = string_set_option(
            RULE_NO_FRAMEWORK_IN_CORE,
            &rule_setting,
            "coreLayers",
            &["domain", "application"],
        )?;
        let framework_packages = package_pattern_option(
            RULE_NO_FRAMEWORK_IN_CORE,
            &rule_setting,
            "packages",
            &[
                "express",
                "fastify",
                "hono",
                "koa",
                "next",
                "react",
                "vue",
                "angular",
                "@nestjs/*",
                "drizzle-orm",
                "prisma",
                "@prisma/*",
                "typeorm",
                "sequelize",
                "mongoose",
                "pg",
                "mysql2",
                "redis",
                "ioredis",
            ],
        )?;
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        if !core_layers.contains(from_layer) {
            continue;
        }

        let package_name = normalized_package_name(&edge.specifier);
        if !framework_packages.is_allowed(&package_name) {
            continue;
        }

        violations.push(Violation::framework_in_core(
            edge,
            from_layer,
            &package_name,
            severity,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_outer_data_format_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.layers.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = LayerClassifier::new(project_root, &loaded.config.layers)?;
    let mut violations = Vec::new();

    for edge in edges {
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let severity = rule_policy.effective_severity(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        let rule_setting = rule_policy.effective_rule(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            project_root,
            &edge.source,
        );
        let core_layers = string_set_option(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            &rule_setting,
            "coreLayers",
            &["domain", "application"],
        )?;
        let outer_layers = string_set_option(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            &rule_setting,
            "outerLayers",
            &["infra"],
        )?;
        let format_segments = string_set_option(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            &rule_setting,
            "formatSegments",
            &[
                "schema", "schemas", "dto", "dtos", "record", "records", "row", "rows",
            ],
        )?;
        let format_suffixes = string_vec_option(
            RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
            &rule_setting,
            "formatSuffixes",
            &[
                ".schema.ts",
                ".schema.tsx",
                ".schema.js",
                ".dto.ts",
                ".dto.tsx",
                ".record.ts",
                ".row.ts",
                "config-types.ts",
            ],
        )?;
        let LayerClassification::Classified(from_layer) = classifier.classify(&edge.source) else {
            continue;
        };
        let LayerClassification::Classified(to_layer) = classifier.classify(target) else {
            continue;
        };
        if !core_layers.contains(from_layer) || !outer_layers.contains(to_layer) {
            continue;
        }
        if !path_has_any_segment(target, project_root, &format_segments)
            && !path_ends_with_any(target, project_root, &format_suffixes)
        {
            continue;
        }

        violations.push(Violation::outer_data_format_in_core(
            edge, target, from_layer, to_layer, severity,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_public_surface_reexport_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let context_policy = ContextPolicy::from(&loaded.config.context_rules);
    let mut violations = Vec::new();

    for edge in edges {
        if edge.kind != ImportKind::ReExport {
            continue;
        }
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let severity = rule_policy.effective_severity(
            RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        if !context_policy.is_public_surface(&edge.source, project_root)
            || context_policy.is_public_surface(target, project_root)
        {
            continue;
        }
        let ContextClassification::Classified(from_context) = classifier.classify(&edge.source)
        else {
            continue;
        };
        let ContextClassification::Classified(to_context) = classifier.classify(target) else {
            continue;
        };
        if from_context != to_context {
            continue;
        }

        violations.push(Violation::public_surface_internal_reexport(
            edge,
            target,
            from_context,
            severity,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_context_cycle_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let mut graph = BTreeMap::<PathBuf, BTreeSet<PathBuf>>::new();
    let mut representatives = BTreeMap::<(String, String), (&ImportEdge, PathBuf)>::new();

    for edge in edges {
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let ContextClassification::Classified(from_context) = classifier.classify(&edge.source)
        else {
            continue;
        };
        let ContextClassification::Classified(to_context) = classifier.classify(target) else {
            continue;
        };
        if from_context == to_context {
            continue;
        }

        graph
            .entry(PathBuf::from(from_context))
            .or_default()
            .insert(PathBuf::from(to_context));
        graph.entry(PathBuf::from(to_context)).or_default();
        representatives
            .entry((from_context.to_string(), to_context.to_string()))
            .or_insert((edge, target.clone()));
    }

    let graph = graph
        .into_iter()
        .map(|(source, targets)| (source, targets.into_iter().collect::<Vec<_>>()))
        .collect::<BTreeMap<_, _>>();
    let cycles = find_canonical_cycles(&graph);
    let mut violations = Vec::new();

    for cycle in cycles {
        if cycle.len() < 3 {
            continue;
        }
        let context_path = cycle
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        let Some(from_context) = context_path.first() else {
            continue;
        };
        let Some(to_context) = context_path.get(1) else {
            continue;
        };
        let Some((edge, target)) = representatives.get(&(from_context.clone(), to_context.clone()))
        else {
            continue;
        };
        let severity =
            rule_policy.effective_severity(RULE_NO_CONTEXT_CYCLE, project_root, &edge.source);
        if severity == Severity::Off {
            continue;
        }
        violations.push(Violation::context_cycle(
            edge,
            target,
            &context_path,
            severity,
        ));
    }

    Ok(violations)
}

pub(crate) fn collect_unowned_schema_import_violations(
    loaded: &LoadedConfig,
    project_root: &Path,
    edges: &[ImportEdge],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    if loaded.config.contexts.is_empty() {
        return Ok(Vec::new());
    }

    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let mut violations = Vec::new();

    for edge in edges {
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let severity = rule_policy.effective_severity(
            RULE_NO_UNOWNED_SCHEMA_IMPORT,
            project_root,
            &edge.source,
        );
        if severity == Severity::Off {
            continue;
        }
        let rule_setting =
            rule_policy.effective_rule(RULE_NO_UNOWNED_SCHEMA_IMPORT, project_root, &edge.source);
        let schema_segments = string_set_option(
            RULE_NO_UNOWNED_SCHEMA_IMPORT,
            &rule_setting,
            "schemaSegments",
            &["schema", "schemas"],
        )?;
        let schema_suffixes = string_vec_option(
            RULE_NO_UNOWNED_SCHEMA_IMPORT,
            &rule_setting,
            "schemaSuffixes",
            &[
                ".schema.ts",
                ".schema.tsx",
                ".schema.js",
                ".model.ts",
                ".model.tsx",
            ],
        )?;
        if !path_has_any_segment(target, project_root, &schema_segments)
            && !path_ends_with_any(target, project_root, &schema_suffixes)
        {
            continue;
        }
        let ContextClassification::Classified(from_context) = classifier.classify(&edge.source)
        else {
            continue;
        };
        let ContextClassification::Classified(to_context) = classifier.classify(target) else {
            continue;
        };
        if from_context == to_context {
            continue;
        }

        violations.push(Violation::unowned_schema_import(
            edge,
            target,
            from_context,
            to_context,
            severity,
        ));
    }

    Ok(violations)
}

fn find_canonical_cycles(graph: &BTreeMap<PathBuf, Vec<PathBuf>>) -> Vec<Vec<PathBuf>> {
    let components = TarjanState::new(graph).strongly_connected_components();
    let mut cycles = components
        .into_iter()
        .filter_map(|component| representative_cycle(graph, component))
        .collect::<Vec<_>>();
    cycles.sort();
    cycles
}

struct TarjanState<'a> {
    graph: &'a BTreeMap<PathBuf, Vec<PathBuf>>,
    next_index: usize,
    indexes: BTreeMap<PathBuf, usize>,
    lowlinks: BTreeMap<PathBuf, usize>,
    stack: Vec<PathBuf>,
    on_stack: HashSet<PathBuf>,
    components: Vec<Vec<PathBuf>>,
}

impl<'a> TarjanState<'a> {
    fn new(graph: &'a BTreeMap<PathBuf, Vec<PathBuf>>) -> Self {
        Self {
            graph,
            next_index: 0,
            indexes: BTreeMap::new(),
            lowlinks: BTreeMap::new(),
            stack: Vec::new(),
            on_stack: HashSet::new(),
            components: Vec::new(),
        }
    }

    fn strongly_connected_components(mut self) -> Vec<Vec<PathBuf>> {
        for node in self.graph.keys() {
            if !self.indexes.contains_key(node) {
                self.visit(node.clone());
            }
        }
        self.components
    }

    fn visit(&mut self, node: PathBuf) {
        let index = self.next_index;
        self.next_index += 1;
        self.indexes.insert(node.clone(), index);
        self.lowlinks.insert(node.clone(), index);
        self.stack.push(node.clone());
        self.on_stack.insert(node.clone());

        if let Some(targets) = self.graph.get(&node) {
            for target in targets {
                if !self.graph.contains_key(target) {
                    continue;
                }
                if !self.indexes.contains_key(target) {
                    self.visit(target.clone());
                    let target_lowlink = *self
                        .lowlinks
                        .get(target)
                        .expect("visited target should have a lowlink");
                    let node_lowlink = *self
                        .lowlinks
                        .get(&node)
                        .expect("visited node should have a lowlink");
                    self.lowlinks
                        .insert(node.clone(), node_lowlink.min(target_lowlink));
                } else if self.on_stack.contains(target) {
                    let target_index = *self
                        .indexes
                        .get(target)
                        .expect("indexed target should have an index");
                    let node_lowlink = *self
                        .lowlinks
                        .get(&node)
                        .expect("visited node should have a lowlink");
                    self.lowlinks
                        .insert(node.clone(), node_lowlink.min(target_index));
                }
            }
        }

        let node_lowlink = *self
            .lowlinks
            .get(&node)
            .expect("visited node should have a lowlink");
        if node_lowlink != index {
            return;
        }

        let mut component = Vec::new();
        loop {
            let member = self
                .stack
                .pop()
                .expect("root component should have stack members");
            self.on_stack.remove(&member);
            component.push(member.clone());
            if member == node {
                break;
            }
        }
        component.sort();
        self.components.push(component);
    }
}

fn representative_cycle(
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    component: Vec<PathBuf>,
) -> Option<Vec<PathBuf>> {
    if component.len() == 1 {
        let node = component.first()?;
        return graph.get(node).and_then(|targets| {
            targets
                .contains(node)
                .then(|| vec![node.clone(), node.clone()])
        });
    }

    let component_set = component.iter().cloned().collect::<HashSet<_>>();
    component
        .iter()
        .find_map(|start| representative_cycle_from(graph, start, &component_set))
}

fn representative_cycle_from(
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    start: &PathBuf,
    component: &HashSet<PathBuf>,
) -> Option<Vec<PathBuf>> {
    let mut targets = graph.get(start)?.clone();
    targets.retain(|target| component.contains(target) && target != start);
    targets.sort();

    for target in targets {
        let mut cycle = vec![start.clone(), target.clone()];
        let mut visited = HashSet::from([start.clone(), target]);
        if close_cycle(graph, start, component, &mut cycle, &mut visited) {
            return Some(cycle);
        }
    }

    None
}

fn close_cycle(
    graph: &BTreeMap<PathBuf, Vec<PathBuf>>,
    start: &PathBuf,
    component: &HashSet<PathBuf>,
    cycle: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) -> bool {
    let Some(current) = cycle.last().cloned() else {
        return false;
    };
    let Some(targets) = graph.get(&current) else {
        return false;
    };
    let mut targets = targets
        .iter()
        .filter(|target| component.contains(*target))
        .cloned()
        .collect::<Vec<_>>();
    targets.sort();

    for target in targets {
        if &target == start {
            cycle.push(start.clone());
            return true;
        }
        if !visited.insert(target.clone()) {
            continue;
        }
        cycle.push(target.clone());
        if close_cycle(graph, start, component, cycle, visited) {
            return true;
        }
        cycle.pop();
        visited.remove(&target);
    }

    false
}
