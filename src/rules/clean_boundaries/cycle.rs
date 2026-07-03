use crate::*;
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
