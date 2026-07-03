use crate::*;
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
