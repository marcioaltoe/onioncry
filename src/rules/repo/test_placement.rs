use crate::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

struct TestPlacementPolicy {
    source_roots: Vec<Vec<String>>,
    unit_test_directories: HashSet<String>,
    integration_roots: Vec<Vec<String>>,
    e2e_roots: Vec<Vec<String>>,
    test_file_suffixes: Vec<String>,
}

pub(crate) fn collect_test_placement_violations(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    for file in files {
        let rule_setting = rule_policy.effective_rule(RULE_TEST_PLACEMENT, project_root, file);
        if rule_setting.severity == Severity::Off {
            continue;
        }
        let policy = TestPlacementPolicy::from_rule_setting(&rule_setting)?;
        if !policy.is_test_file(project_root, file)
            || policy.is_valid_test_placement(project_root, file)
        {
            continue;
        }

        violations.push(Violation::misplaced_test_file(
            file,
            rule_setting.severity,
            policy.suggestion(project_root, file),
        ));
    }

    Ok(violations)
}

impl TestPlacementPolicy {
    fn from_rule_setting(setting: &RuleSetting) -> Result<Self> {
        let source_roots =
            string_vec_option(RULE_TEST_PLACEMENT, setting, "sourceRoots", &["src"])?;
        let unit_test_directories = string_set_option(
            RULE_TEST_PLACEMENT,
            setting,
            "unitTestDirectories",
            &["__tests__"],
        )?;
        let integration_roots = string_vec_option(
            RULE_TEST_PLACEMENT,
            setting,
            "integrationRoots",
            &["tests/integration"],
        )?;
        let e2e_roots =
            string_vec_option(RULE_TEST_PLACEMENT, setting, "e2eRoots", &["tests/e2e"])?;
        let test_file_suffixes = string_vec_option(
            RULE_TEST_PLACEMENT,
            setting,
            "testFileSuffixes",
            DEFAULT_TEST_FILE_SUFFIXES,
        )?
        .into_iter()
        .map(|suffix| suffix.to_ascii_lowercase())
        .collect();

        Ok(Self {
            source_roots: path_roots(source_roots),
            unit_test_directories,
            integration_roots: path_roots(integration_roots),
            e2e_roots: path_roots(e2e_roots),
            test_file_suffixes,
        })
    }

    fn is_test_file(&self, project_root: &Path, file: &Path) -> bool {
        let relative_path = project_relative_display(project_root, file).to_ascii_lowercase();
        self.test_file_suffixes
            .iter()
            .any(|suffix| relative_path.ends_with(suffix))
    }

    fn is_valid_test_placement(&self, project_root: &Path, file: &Path) -> bool {
        let components = project_relative_components(project_root, file);
        self.is_under_context_root(&components, &self.integration_roots)
            || self.is_under_context_root(&components, &self.e2e_roots)
            || (self.is_under_any_root(&components, &self.source_roots)
                && components
                    .iter()
                    .any(|segment| self.unit_test_directories.contains(segment)))
    }

    fn suggestion(&self, project_root: &Path, file: &Path) -> String {
        let components = project_relative_components(project_root, file);
        if self.is_under_any_root(&components, &self.integration_roots) {
            return format!(
                "place integration tests under {}/<context>/",
                display_root(&self.integration_roots)
            );
        }
        if self.is_under_any_root(&components, &self.e2e_roots) {
            return format!(
                "place e2e tests under {}/<context>/",
                display_root(&self.e2e_roots)
            );
        }
        if self.is_under_any_root(&components, &self.source_roots) {
            return format!(
                "move this unit test into a colocated {} directory",
                display_unit_test_directories(&self.unit_test_directories)
            );
        }

        format!(
            "move this test under {}, {}, or a colocated {} directory in {}",
            display_root(&self.integration_roots),
            display_root(&self.e2e_roots),
            display_unit_test_directories(&self.unit_test_directories),
            display_root(&self.source_roots)
        )
    }

    fn is_under_context_root(&self, components: &[String], roots: &[Vec<String>]) -> bool {
        roots.iter().any(|root| {
            path_has_prefix_components(components, root) && components.len() >= root.len() + 2
        })
    }

    fn is_under_any_root(&self, components: &[String], roots: &[Vec<String>]) -> bool {
        roots
            .iter()
            .any(|root| path_has_prefix_components(components, root))
    }
}

fn display_unit_test_directories(unit_test_directories: &HashSet<String>) -> String {
    let mut directories = unit_test_directories.iter().cloned().collect::<Vec<_>>();
    directories.sort();
    directories
        .first()
        .cloned()
        .unwrap_or_else(|| "__tests__".to_string())
}
