use crate::{
    ArchitectureMode, ContextClassification, ContextClassifier, ContextPolicy, GraphEdge,
    GraphNode, GraphReport, ImportEdge, ImportResolution, LoadedConfig, Result,
    VerticalSlicePolicy,
};
use std::collections::BTreeMap;
use std::path::Path;

const CONTEXTLESS_NODE: &str = "contextless";
const SLICELESS_NODE: &str = "sliceless";

#[derive(Clone)]
struct BoundaryNode {
    id: String,
    label: String,
    kind: String,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct GraphEdgeKey {
    from: String,
    to: String,
    via: Option<String>,
}

pub(crate) fn build_graph_report(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[std::path::PathBuf],
    edges: &[ImportEdge],
) -> Result<GraphReport> {
    match loaded.config.architecture.mode {
        ArchitectureMode::CleanArchitecture => {
            build_context_graph(loaded, project_root, files, edges)
        }
        ArchitectureMode::VerticalSlice => build_slice_graph(loaded, project_root, files, edges),
    }
}

fn build_context_graph(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[std::path::PathBuf],
    edges: &[ImportEdge],
) -> Result<GraphReport> {
    let classifier = ContextClassifier::new(project_root, &loaded.config.contexts)?;
    let policy = ContextPolicy::from(&loaded.config.context_rules);
    let mut nodes = BTreeMap::<String, BoundaryNode>::new();
    let mut edge_counts = BTreeMap::<GraphEdgeKey, usize>::new();

    for file in files {
        insert_node(&mut nodes, context_node(&classifier, file));
    }

    for edge in edges {
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let from = context_node(&classifier, &edge.source);
        let to = context_node(&classifier, target);
        insert_node(&mut nodes, from.clone());
        insert_node(&mut nodes, to.clone());
        if from.id == to.id {
            continue;
        }
        *edge_counts
            .entry(GraphEdgeKey {
                from: from.id,
                to: to.id,
                via: policy.public_surface_segment(target, project_root),
            })
            .or_default() += 1;
    }

    Ok(report_from_maps(nodes, edge_counts))
}

fn build_slice_graph(
    loaded: &LoadedConfig,
    project_root: &Path,
    files: &[std::path::PathBuf],
    edges: &[ImportEdge],
) -> Result<GraphReport> {
    let policy = VerticalSlicePolicy::from_config(&loaded.config.architecture.vertical_slice);
    let mut nodes = BTreeMap::<String, BoundaryNode>::new();
    let mut edge_counts = BTreeMap::<GraphEdgeKey, usize>::new();

    for file in files {
        insert_node(&mut nodes, slice_node(&policy, project_root, file));
    }

    for edge in edges {
        let ImportResolution::Local(target) = &edge.resolution else {
            continue;
        };
        let from = slice_node(&policy, project_root, &edge.source);
        let to = slice_node(&policy, project_root, target);
        insert_node(&mut nodes, from.clone());
        insert_node(&mut nodes, to.clone());
        if from.id == to.id {
            continue;
        }
        let via = policy
            .slice_location(project_root, target)
            .and_then(|location| policy.public_surface_label(&location));
        *edge_counts
            .entry(GraphEdgeKey {
                from: from.id,
                to: to.id,
                via,
            })
            .or_default() += 1;
    }

    Ok(report_from_maps(nodes, edge_counts))
}

fn context_node(classifier: &ContextClassifier, file: &Path) -> BoundaryNode {
    match classifier.classify(file) {
        ContextClassification::Classified(context) => BoundaryNode {
            id: context.to_string(),
            label: context.to_string(),
            kind: "context".to_string(),
        },
        ContextClassification::Contextless => BoundaryNode {
            id: CONTEXTLESS_NODE.to_string(),
            label: "Contextless".to_string(),
            kind: "contextless".to_string(),
        },
        ContextClassification::Ambiguous(contexts) => BoundaryNode {
            id: format!("ambiguous:{}", contexts.join(",")),
            label: format!("Ambiguous: {}", contexts.join(", ")),
            kind: "ambiguous".to_string(),
        },
    }
}

fn slice_node(policy: &VerticalSlicePolicy, project_root: &Path, file: &Path) -> BoundaryNode {
    policy.slice_location(project_root, file).map_or_else(
        || BoundaryNode {
            id: SLICELESS_NODE.to_string(),
            label: "Sliceless".to_string(),
            kind: "sliceless".to_string(),
        },
        |location| BoundaryNode {
            id: location.slice.clone(),
            label: location.slice,
            kind: "slice".to_string(),
        },
    )
}

fn insert_node(nodes: &mut BTreeMap<String, BoundaryNode>, node: BoundaryNode) {
    nodes.entry(node.id.clone()).or_insert(node);
}

fn report_from_maps(
    nodes: BTreeMap<String, BoundaryNode>,
    edge_counts: BTreeMap<GraphEdgeKey, usize>,
) -> GraphReport {
    GraphReport {
        nodes: nodes
            .into_values()
            .map(|node| GraphNode {
                id: node.id,
                label: node.label,
                kind: node.kind,
            })
            .collect(),
        edges: edge_counts
            .into_iter()
            .map(|(edge, import_count)| GraphEdge {
                from: edge.from,
                to: edge.to,
                via: edge.via,
                import_count,
            })
            .collect(),
    }
}
