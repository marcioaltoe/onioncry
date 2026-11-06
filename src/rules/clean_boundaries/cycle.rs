use crate::rules::catalog::RULE_NO_CONTEXT_CYCLE;
use crate::{
    ContextClassification, ContextClassifier, ImportEdge, ImportResolution, LoadedConfig, Result,
    RulePolicy, Severity, Violation,
};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

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

fn find_canonical_cycles(graph: &BTreeMap<PathBuf, Vec<PathBuf>>) -> Vec<Vec<PathBuf>> {
    let components = TarjanState::new(graph).strongly_connected_components();
    let mut cycles = components
        .into_iter()
        .filter_map(|component| representative_cycle(graph, component))
        .collect::<Vec<_>>();
    cycles.sort();
    cycles
}

// Node state lives in vectors indexed by node id, so a visited node always has
// an index and a lowlink and the algorithm has no unreachable lookup branches.
struct TarjanState<'a> {
    nodes: Vec<&'a PathBuf>,
    successors: Vec<Vec<usize>>,
    next_index: usize,
    indexes: Vec<Option<usize>>,
    lowlinks: Vec<usize>,
    stack: Vec<usize>,
    on_stack: Vec<bool>,
    components: Vec<Vec<PathBuf>>,
}

impl<'a> TarjanState<'a> {
    fn new(graph: &'a BTreeMap<PathBuf, Vec<PathBuf>>) -> Self {
        let nodes = graph.keys().collect::<Vec<_>>();
        let ids = nodes
            .iter()
            .enumerate()
            .map(|(id, node)| (*node, id))
            .collect::<BTreeMap<_, _>>();
        // Targets that are not graph keys cannot be visited and are dropped here.
        let successors = graph
            .values()
            .map(|targets| {
                targets
                    .iter()
                    .filter_map(|target| ids.get(target).copied())
                    .collect()
            })
            .collect();
        let node_count = nodes.len();
        Self {
            nodes,
            successors,
            next_index: 0,
            indexes: vec![None; node_count],
            lowlinks: vec![0; node_count],
            stack: Vec::new(),
            on_stack: vec![false; node_count],
            components: Vec::new(),
        }
    }

    fn strongly_connected_components(mut self) -> Vec<Vec<PathBuf>> {
        for node in 0..self.nodes.len() {
            if self.indexes[node].is_none() {
                self.visit(node);
            }
        }
        self.components
    }

    fn visit(&mut self, node: usize) {
        let index = self.next_index;
        self.next_index += 1;
        self.indexes[node] = Some(index);
        self.lowlinks[node] = index;
        self.stack.push(node);
        self.on_stack[node] = true;

        for target in self.successors[node].clone() {
            match self.indexes[target] {
                None => {
                    self.visit(target);
                    self.lowlinks[node] = self.lowlinks[node].min(self.lowlinks[target]);
                }
                Some(target_index) if self.on_stack[target] => {
                    self.lowlinks[node] = self.lowlinks[node].min(target_index);
                }
                Some(_) => {}
            }
        }

        if self.lowlinks[node] != index {
            return;
        }

        let mut component = Vec::new();
        while let Some(member) = self.stack.pop() {
            self.on_stack[member] = false;
            component.push(self.nodes[member].clone());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(edges: &[(&str, &[&str])]) -> BTreeMap<PathBuf, Vec<PathBuf>> {
        edges
            .iter()
            .map(|(source, targets)| {
                (
                    PathBuf::from(source),
                    targets.iter().map(PathBuf::from).collect(),
                )
            })
            .collect()
    }

    fn cycles(edges: &[(&str, &[&str])]) -> Vec<Vec<String>> {
        find_canonical_cycles(&graph(edges))
            .into_iter()
            .map(|cycle| {
                cycle
                    .iter()
                    .map(|node| node.display().to_string())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn acyclic_graphs_have_no_cycles() {
        assert!(cycles(&[]).is_empty());
        assert!(cycles(&[("a", &["b"]), ("b", &["c"]), ("c", &[])]).is_empty());
        assert!(cycles(&[("a", &["b", "c"]), ("b", &["c"]), ("c", &[])]).is_empty());
    }

    #[test]
    fn self_references_are_single_node_cycles() {
        assert_eq!(cycles(&[("a", &["a"])]), vec![vec!["a", "a"]]);
    }

    #[test]
    fn each_strongly_connected_component_yields_its_smallest_closed_cycle() {
        assert_eq!(
            cycles(&[("a", &["b"]), ("b", &["a"])]),
            vec![vec!["a", "b", "a"]]
        );
        assert_eq!(
            cycles(&[("c", &["a"]), ("a", &["b"]), ("b", &["c"])]),
            vec![vec!["a", "b", "c", "a"]]
        );
        assert_eq!(
            cycles(&[
                ("a", &["b"]),
                ("b", &["a", "c"]),
                ("c", &["d"]),
                ("d", &["c", "e"]),
                ("e", &[]),
            ]),
            vec![vec!["a", "b", "a"], vec!["c", "d", "c"]]
        );
    }

    #[test]
    fn targets_outside_the_graph_are_ignored() {
        assert!(cycles(&[("a", &["missing"])]).is_empty());
        assert_eq!(
            cycles(&[("a", &["b", "missing"]), ("b", &["a"])]),
            vec![vec!["a", "b", "a"]]
        );
    }

    #[test]
    fn deep_chains_closing_on_the_first_node_form_one_cycle() {
        let names = (0..50)
            .map(|index| format!("n{index:02}"))
            .collect::<Vec<_>>();
        let graph = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let next = &names[(index + 1) % names.len()];
                (PathBuf::from(name), vec![PathBuf::from(next)])
            })
            .collect::<BTreeMap<_, _>>();
        let found = find_canonical_cycles(&graph);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].len(), 51);
        assert_eq!(found[0].first(), found[0].last());
    }
}
