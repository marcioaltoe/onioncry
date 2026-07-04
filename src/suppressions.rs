use crate::imports::line_column;
use crate::rules::catalog::{
    RULE_INVALID_SUPPRESSION, RULE_UNUSED_SUPPRESSION, canonical_rule_name, closest_rule_names,
};
use crate::{OnionCryError, Result, RulePolicy, Severity, Violation, is_source_file};
use oxc_allocator::Allocator;
use oxc_ast::ast::Comment;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};

const DIRECTIVE: &str = "onioncry-disable-next-line";

#[derive(Debug)]
struct InlineSuppression {
    file: PathBuf,
    file_display: String,
    line: usize,
    column: usize,
    reason: String,
    rules: Vec<SuppressedRule>,
}

#[derive(Debug)]
struct SuppressedRule {
    canonical_name: String,
    used: bool,
}

pub(crate) fn apply_inline_suppressions(
    project_root: &Path,
    files: &[PathBuf],
    rule_policy: &RulePolicy,
    mut violations: Vec<Violation>,
) -> Result<Vec<Violation>> {
    let mut suppressions = Vec::new();
    let mut suppression_violations = Vec::new();

    for file in files {
        if !is_source_file(file) {
            continue;
        }
        let collection = collect_file_suppressions(file, project_root, rule_policy)?;
        suppressions.extend(collection.suppressions);
        suppression_violations.extend(collection.violations);
    }

    for suppression in &mut suppressions {
        for rule in &mut suppression.rules {
            let target_line = suppression.line + 1;
            for violation in &mut violations {
                if violation.file != suppression.file_display {
                    continue;
                }
                if violation.rule != rule.canonical_name {
                    continue;
                }
                if violation.line != Some(target_line) {
                    continue;
                }
                violation.suppressed = true;
                violation.suppression_reason = Some(suppression.reason.clone());
                rule.used = true;
            }
        }
    }

    for suppression in &suppression_violations {
        violations.push(suppression.clone());
    }

    for suppression in &suppressions {
        for rule in &suppression.rules {
            if rule.used {
                continue;
            }
            let severity = rule_policy.effective_severity(
                RULE_UNUSED_SUPPRESSION,
                project_root,
                &suppression.file,
            );
            if severity == Severity::Off {
                continue;
            }
            violations.push(Violation::unused_suppression(
                &suppression.file,
                severity,
                suppression.line,
                suppression.column,
                &rule.canonical_name,
            ));
        }
    }

    Ok(violations)
}

struct FileSuppressions {
    suppressions: Vec<InlineSuppression>,
    violations: Vec<Violation>,
}

fn collect_file_suppressions(
    file: &Path,
    project_root: &Path,
    rule_policy: &RulePolicy,
) -> Result<FileSuppressions> {
    let source = fs::read_to_string(file).map_err(|source| OnionCryError::ReadSource {
        path: file.to_path_buf(),
        source,
    })?;
    let source_type = SourceType::from_path(file).map_err(|source| OnionCryError::ParseSource {
        path: file.to_path_buf(),
        message: source.to_string(),
    })?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        let message = parsed
            .errors
            .first()
            .map_or_else(|| "parser panicked".to_string(), ToString::to_string);
        return Err(OnionCryError::ParseSource {
            path: file.to_path_buf(),
            message,
        });
    }

    let mut suppressions = Vec::new();
    let mut violations = Vec::new();
    for comment in &parsed.program.comments {
        let Some(parsed_directive) = parse_directive(file, &source, comment) else {
            continue;
        };

        match parsed_directive {
            ParsedDirective::Valid(suppression) => suppressions.push(suppression),
            ParsedDirective::Invalid {
                line,
                column,
                messages,
            } => {
                let severity =
                    rule_policy.effective_severity(RULE_INVALID_SUPPRESSION, project_root, file);
                if severity == Severity::Off {
                    continue;
                }
                violations.extend(messages.into_iter().map(|message| {
                    Violation::invalid_suppression(file, severity, line, column, message)
                }));
            }
        }
    }

    Ok(FileSuppressions {
        suppressions,
        violations,
    })
}

enum ParsedDirective {
    Valid(InlineSuppression),
    Invalid {
        line: usize,
        column: usize,
        messages: Vec<String>,
    },
}

fn parse_directive(file: &Path, source: &str, comment: &Comment) -> Option<ParsedDirective> {
    if !comment.is_line() {
        return None;
    }

    let content_span = comment.content_span();
    let content = &source[content_span.start as usize..content_span.end as usize];
    let content = content.trim_start();
    let body = content.strip_prefix(DIRECTIVE)?;
    if !body.is_empty() && !body.starts_with(char::is_whitespace) {
        return None;
    }

    let (line, column) = line_column(source, comment.span.start as usize);
    let Some((rules_part, reason_part)) = body.split_once("--") else {
        return Some(invalid_directive(
            line,
            column,
            "inline suppression requires `-- <reason>`",
        ));
    };

    let reason = reason_part.trim();
    if reason.is_empty() {
        return Some(invalid_directive(
            line,
            column,
            "inline suppression requires `-- <reason>`",
        ));
    }

    let rule_names = rules_part.split(',').map(str::trim).collect::<Vec<_>>();
    if rule_names.is_empty() || rule_names.iter().all(|rule| rule.is_empty()) {
        return Some(invalid_directive(
            line,
            column,
            "inline suppression requires at least one rule before `--`",
        ));
    }

    let mut errors = Vec::new();
    let mut rules = Vec::new();
    for rule in rule_names {
        if rule.is_empty() {
            errors.push("inline suppression contains an empty rule name".to_string());
            continue;
        }

        match canonical_rule_name(rule) {
            Some(canonical_name) => rules.push(SuppressedRule {
                canonical_name: canonical_name.to_string(),
                used: false,
            }),
            None => errors.push(format!(
                "unknown rule {rule:?} in inline suppression; closest known rules: {}",
                closest_rule_names(rule, 3).join(", ")
            )),
        }
    }

    if !errors.is_empty() {
        return Some(ParsedDirective::Invalid {
            line,
            column,
            messages: errors,
        });
    }

    Some(ParsedDirective::Valid(InlineSuppression {
        file: file.to_path_buf(),
        file_display: file.display().to_string(),
        line,
        column,
        reason: reason.to_string(),
        rules,
    }))
}

fn invalid_directive(line: usize, column: usize, message: &str) -> ParsedDirective {
    ParsedDirective::Invalid {
        line,
        column,
        messages: vec![message.to_string()],
    }
}
