use crate::{ArchitectureMode, Severity};
use serde::Serialize;

pub(crate) const RULE_NO_LAYER_LEAK: &str = "cleanarch/no-layer-leak";
pub(crate) const RULE_NO_FORBIDDEN_IMPORTS: &str = "cleanarch/no-forbidden-imports";
pub(crate) const RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT: &str =
    "cleanarch/no-cross-context-internal-import";
pub(crate) const RULE_NO_FRAMEWORK_IN_CORE: &str = "cleanarch/no-framework-in-core";
pub(crate) const RULE_NO_OUTER_DATA_FORMAT_IN_CORE: &str = "cleanarch/no-outer-data-format-in-core";
pub(crate) const RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT: &str =
    "cleanarch/no-public-surface-internal-reexport";
pub(crate) const RULE_NO_CONTEXT_CYCLE: &str = "cleanarch/no-context-cycle";
pub(crate) const RULE_NO_UNOWNED_SCHEMA_IMPORT: &str = "cleanarch/no-unowned-schema-import";
pub(crate) const RULE_CLEAN_ARTIFACT_PLACEMENT: &str = "cleanarch/artifact-placement";
pub(crate) const RULE_UNCLASSIFIED_FILE: &str = "cleanarch/unclassified-file";
pub(crate) const RULE_AMBIGUOUS_LAYER: &str = "cleanarch/ambiguous-layer";
pub(crate) const RULE_AMBIGUOUS_CONTEXT: &str = "cleanarch/ambiguous-context";
pub(crate) const RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT: &str =
    "verticalslice/no-cross-slice-internal-import";
pub(crate) const RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS: &str =
    "verticalslice/no-global-slice-artifacts";
pub(crate) const RULE_VERTICAL_SLICE_ENTRY_POINT: &str = "verticalslice/slice-entry-point";
pub(crate) const RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS: &str =
    "verticalslice/no-shared-layer-artifacts";
pub(crate) const RULE_NO_CONCRETE_DEPENDENCY: &str = "solid/no-concrete-dependency";
pub(crate) const RULE_FEATURE_ENVY: &str = "codesmells/feature-envy";
pub(crate) const RULE_SHOTGUN_SURGERY: &str = "codesmells/shotgun-surgery";
pub(crate) const RULE_TEST_PLACEMENT: &str = "repo/test-placement";
pub(crate) const RULE_PATH_NAMING: &str = "repo/path-naming";
pub(crate) const RULE_INVALID_SUPPRESSION: &str = "repo/invalid-suppression";
pub(crate) const RULE_UNUSED_SUPPRESSION: &str = "repo/unused-suppression";
pub(crate) const RULE_FEATURE_SYSTEM_LAYOUT: &str = "frontend/feature-system-layout";
pub(crate) const RULE_FEATURE_SYSTEM_PUBLIC_API: &str = "frontend/feature-system-public-api";
pub(crate) const RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW: &str =
    "frontend/feature-system-dependency-flow";
pub(crate) const RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT: &str =
    "frontend/feature-system-adapter-contract";
pub(crate) const RULE_FEATURE_SYSTEM_QUERY_CONTRACT: &str =
    "frontend/feature-system-query-contract";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArchitectureRuleFamily {
    CleanArchitecture,
    VerticalSlice,
}

impl ArchitectureRuleFamily {
    pub(crate) fn expected_for_mode(mode: ArchitectureMode) -> Self {
        match mode {
            ArchitectureMode::CleanArchitecture => Self::CleanArchitecture,
            ArchitectureMode::VerticalSlice => Self::VerticalSlice,
        }
    }

    pub(crate) fn display(self) -> &'static str {
        match self {
            Self::CleanArchitecture => "cleanarch/*",
            Self::VerticalSlice => "verticalslice/*",
        }
    }
}

pub(crate) struct RuleDescriptor {
    pub(crate) id: &'static str,
    pub(crate) legacy_aliases: &'static [&'static str],
    pub(crate) default_severity: Severity,
    pub(crate) architecture_family: Option<ArchitectureRuleFamily>,
    pub(crate) explanation: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleCatalogEntry {
    pub name: &'static str,
    pub legacy_aliases: &'static [&'static str],
    pub default_severity: &'static str,
    pub architecture_family: &'static str,
    pub explanation: &'static str,
}

pub(crate) const RULES: &[RuleDescriptor] = &[
    RuleDescriptor {
        id: RULE_NO_LAYER_LEAK,
        legacy_aliases: &["onion/no-layer-leak"],
        default_severity: Severity::Error,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Layer rules only allow imports declared in the importing layer's mayImport policy.",
    },
    RuleDescriptor {
        id: RULE_NO_FORBIDDEN_IMPORTS,
        legacy_aliases: &["onion/no-forbidden-imports"],
        default_severity: Severity::Error,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "External packages are closed by default in sensitive layers unless explicitly allowed.",
    },
    RuleDescriptor {
        id: RULE_NO_CROSS_CONTEXT_INTERNAL_IMPORT,
        legacy_aliases: &["onion/no-cross-context-internal-import"],
        default_severity: Severity::Error,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Cross-context imports must target the imported context's configured public surface.",
    },
    RuleDescriptor {
        id: RULE_NO_FRAMEWORK_IN_CORE,
        legacy_aliases: &["onion/no-framework-in-core"],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Core layers should depend on ports, not framework packages.",
    },
    RuleDescriptor {
        id: RULE_NO_OUTER_DATA_FORMAT_IN_CORE,
        legacy_aliases: &["onion/no-outer-data-format-in-core"],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Core layers should not mention data formats owned by outer details.",
    },
    RuleDescriptor {
        id: RULE_NO_PUBLIC_SURFACE_INTERNAL_REEXPORT,
        legacy_aliases: &["onion/no-public-surface-internal-reexport"],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "A public surface should expose intentional contracts, not internal implementation files.",
    },
    RuleDescriptor {
        id: RULE_NO_CONTEXT_CYCLE,
        legacy_aliases: &["onion/no-context-cycle"],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Context dependencies should form a directed acyclic ownership graph.",
    },
    RuleDescriptor {
        id: RULE_NO_UNOWNED_SCHEMA_IMPORT,
        legacy_aliases: &["onion/no-unowned-schema-import"],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "A context should not depend directly on another context's storage schema.",
    },
    RuleDescriptor {
        id: RULE_CLEAN_ARTIFACT_PLACEMENT,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Clean Architecture artifacts should live under a context-first layer boundary or a contextless base layer.",
    },
    RuleDescriptor {
        id: RULE_UNCLASSIFIED_FILE,
        legacy_aliases: &["onion/unclassified-file"],
        default_severity: Severity::Warn,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Layer checks need each analyzed file to match exactly one configured architectural layer.",
    },
    RuleDescriptor {
        id: RULE_AMBIGUOUS_LAYER,
        legacy_aliases: &["onion/ambiguous-layer"],
        default_severity: Severity::Error,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Overlapping layer patterns make it unclear which dependency policy applies to this file.",
    },
    RuleDescriptor {
        id: RULE_AMBIGUOUS_CONTEXT,
        legacy_aliases: &["onion/ambiguous-context"],
        default_severity: Severity::Error,
        architecture_family: Some(ArchitectureRuleFamily::CleanArchitecture),
        explanation: "Overlapping context patterns make it unclear which ownership boundary applies to this file.",
    },
    RuleDescriptor {
        id: RULE_VERTICAL_NO_CROSS_SLICE_INTERNAL_IMPORT,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::VerticalSlice),
        explanation: "Cross-slice imports should target the imported slice's configured public surface.",
    },
    RuleDescriptor {
        id: RULE_VERTICAL_NO_GLOBAL_SLICE_ARTIFACTS,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::VerticalSlice),
        explanation: "Vertical Slice artifacts should live under the configured slice root unless their global folder is explicitly allowed.",
    },
    RuleDescriptor {
        id: RULE_VERTICAL_SLICE_ENTRY_POINT,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::VerticalSlice),
        explanation: "Each Vertical Slice should expose a small configured entry point so routes, jobs, or composition code depend on the slice boundary.",
    },
    RuleDescriptor {
        id: RULE_VERTICAL_NO_SHARED_LAYER_ARTIFACTS,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: Some(ArchitectureRuleFamily::VerticalSlice),
        explanation: "Vertical Slice projects should not rebuild shared technical layers such as global services, repositories, handlers, or use cases.",
    },
    RuleDescriptor {
        id: RULE_NO_CONCRETE_DEPENDENCY,
        legacy_aliases: &["onion/no-concrete-dependency"],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Core layers should depend on abstractions rather than concrete details.",
    },
    RuleDescriptor {
        id: RULE_FEATURE_ENVY,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "A file that mostly imports another context may contain behavior owned by that context.",
    },
    RuleDescriptor {
        id: RULE_SHOTGUN_SURGERY,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Files that repeatedly change with many companions may hide scattered responsibilities.",
    },
    RuleDescriptor {
        id: RULE_TEST_PLACEMENT,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Source-level unit tests should live in colocated test directories, while integration and e2e tests should live under their dedicated workspace roots.",
    },
    RuleDescriptor {
        id: RULE_PATH_NAMING,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Path naming checks observable file and directory names, not code symbols.",
    },
    RuleDescriptor {
        id: RULE_INVALID_SUPPRESSION,
        legacy_aliases: &[],
        default_severity: Severity::Error,
        architecture_family: None,
        explanation: "Inline suppression comments must name known rules and include a non-empty reason.",
    },
    RuleDescriptor {
        id: RULE_UNUSED_SUPPRESSION,
        legacy_aliases: &[],
        default_severity: Severity::Warn,
        architecture_family: None,
        explanation: "Inline suppression comments should match at least one current violation on the next line.",
    },
    RuleDescriptor {
        id: RULE_FEATURE_SYSTEM_LAYOUT,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Feature system layout checks observable systems/<domain> folders, shared UI roots, and surface CSS placement.",
    },
    RuleDescriptor {
        id: RULE_FEATURE_SYSTEM_PUBLIC_API,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Feature system public APIs should be explicit barrels, and callers outside a system should depend on those barrels instead of internals.",
    },
    RuleDescriptor {
        id: RULE_FEATURE_SYSTEM_DEPENDENCY_FLOW,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Feature system dependency flow keeps upper UI layers from shortcutting into adapters and keeps routes on public barrels.",
    },
    RuleDescriptor {
        id: RULE_FEATURE_SYSTEM_ADAPTER_CONTRACT,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Feature system adapter contracts check domain-named API adapters, typed API errors, cancellable reads, and adapter import boundaries.",
    },
    RuleDescriptor {
        id: RULE_FEATURE_SYSTEM_QUERY_CONTRACT,
        legacy_aliases: &[],
        default_severity: Severity::Off,
        architecture_family: None,
        explanation: "Feature system query contracts keep TanStack Query keys, options, hooks, and cache mutations owned by the system lib layer.",
    },
];

pub(crate) fn canonical_rule_name(rule: &str) -> Option<&'static str> {
    rule_descriptor_for(rule).map(|descriptor| descriptor.id)
}

pub(crate) fn default_rule_severity(rule: &str) -> Severity {
    rule_descriptor_for(rule).map_or(Severity::Warn, |descriptor| descriptor.default_severity)
}

pub(crate) fn known_rule_names_display() -> String {
    RULES
        .iter()
        .map(|descriptor| descriptor.id)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn closest_rule_names(rule: &str, limit: usize) -> Vec<&'static str> {
    let mut candidates = RULES
        .iter()
        .map(|descriptor| {
            (
                levenshtein(rule, descriptor.id),
                descriptor.id.len().abs_diff(rule.len()),
                descriptor.id,
            )
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(right.2))
    });

    candidates
        .into_iter()
        .take(limit)
        .map(|(_, _, rule)| rule)
        .collect()
}

pub(crate) fn rule_descriptor_for(rule: &str) -> Option<&'static RuleDescriptor> {
    RULES
        .iter()
        .find(|descriptor| descriptor.id == rule || descriptor.legacy_aliases.contains(&rule))
}

pub(crate) fn rule_explanation(rule: &str) -> &'static str {
    rule_descriptor_for(rule).map_or(
        "This finding violates the configured OnionCry architecture policy.",
        |descriptor| descriptor.explanation,
    )
}

pub fn rule_catalog() -> Vec<RuleCatalogEntry> {
    RULES
        .iter()
        .map(|descriptor| RuleCatalogEntry {
            name: descriptor.id,
            legacy_aliases: descriptor.legacy_aliases,
            default_severity: descriptor.default_severity.as_str(),
            architecture_family: descriptor
                .architecture_family
                .map_or("neutral", ArchitectureRuleFamily::display),
            explanation: descriptor.explanation,
        })
        .collect()
}

fn levenshtein(left: &str, right: &str) -> usize {
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current = vec![0; right_chars.len() + 1];

    for (left_index, left_char) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_char) in right_chars.iter().enumerate() {
            let deletion = previous[right_index + 1] + 1;
            let insertion = current[right_index] + 1;
            let substitution = previous[right_index] + usize::from(left_char != *right_char);
            current[right_index + 1] = deletion.min(insertion).min(substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}
