use crate::{OnionCryError, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectProperty, ObjectPropertyKind};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::HashSet;
use std::path::Path;

#[derive(Default)]
pub(super) struct QuerySyntaxFacts {
    pub(super) declares_query_key: bool,
    pub(super) has_uncontracted_mutation: bool,
}

pub(super) fn summarize_query_syntax(file: &Path, source: &str) -> Result<QuerySyntaxFacts> {
    let source_type = SourceType::from_path(file).map_err(|source| OnionCryError::ParseSource {
        path: file.to_path_buf(),
        message: source.to_string(),
    })?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.fatal_error || parsed.diagnostics.has_errors() {
        return Err(OnionCryError::ParseSource {
            path: file.to_path_buf(),
            message: parsed
                .diagnostics
                .errors()
                .next()
                .map_or_else(|| "parser panicked".to_string(), ToString::to_string),
        });
    }
    let mut visitor = QuerySyntaxVisitor::default();
    visitor.visit_program(&parsed.program);
    Ok(visitor.facts)
}

#[derive(Default)]
struct QuerySyntaxVisitor {
    facts: QuerySyntaxFacts,
    filter_key_starts: HashSet<u32>,
}

impl<'a> Visit<'a> for QuerySyntaxVisitor {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let is_mutation = matches!(
            &call.callee,
            Expression::Identifier(identifier) if identifier.name == "useMutation"
        ) || call.callee.as_member_expression().is_some_and(|member| {
            member
                .static_property_name()
                .is_some_and(|name| name.as_str() == Some("useMutation"))
        });
        if is_mutation && !mutation_has_invalidation_contract(call) {
            self.facts.has_uncontracted_mutation = true;
        }

        if call.callee.as_member_expression().is_some_and(|member| {
            member.static_property_name().is_some_and(|name| {
                matches!(
                    name.as_str(),
                    Some(
                        "invalidateQueries"
                            | "refetchQueries"
                            | "cancelQueries"
                            | "resetQueries"
                            | "removeQueries"
                            | "isFetching"
                            | "getQueriesData"
                            | "setQueriesData"
                    )
                )
            })
        }) && let Some(Expression::ObjectExpression(filter)) = first_argument(call)
        {
            for property in &filter.properties {
                if let ObjectPropertyKind::ObjectProperty(property) = property
                    && property.key.is_specific_static_name("queryKey")
                    && !matches!(
                        property.value.get_inner_expression(),
                        Expression::ArrayExpression(_)
                    )
                {
                    self.filter_key_starts.insert(property.span.start);
                }
            }
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if property.key.is_specific_static_name("queryKey")
            && !self.filter_key_starts.contains(&property.span.start)
        {
            self.facts.declares_query_key = true;
        }
        // Visit nested values even when the filter's own key is a use.
        walk::walk_object_property(self, property);
    }
}

// TypeScript wrappers such as `as const` or `satisfies` never change what the
// wrapped literal declares, so shape checks look through them.
fn first_argument<'b, 'a>(call: &'b CallExpression<'a>) -> Option<&'b Expression<'a>> {
    call.arguments
        .first()
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
}

fn mutation_has_invalidation_contract(call: &CallExpression<'_>) -> bool {
    let Some(Expression::ObjectExpression(options)) = first_argument(call) else {
        return false;
    };
    options.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        if !property.key.is_specific_static_name("meta") {
            return false;
        }
        let Expression::ObjectExpression(meta) = property.value.get_inner_expression() else {
            return false;
        };
        meta.properties.iter().any(|property| {
            matches!(
                property,
                ObjectPropertyKind::ObjectProperty(property)
                    if property.key.is_specific_static_name("invalidates")
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_key_declarations_depend_on_the_direct_filter_position() {
        for source in [
            "useQuery({ queryKey: keys.list() });",
            "const options = { queryKey };",
            "client.invalidateQueries({ queryKey: ['billing'] });",
            "client.refetchQueries({ 'queryKey': ['billing'] });",
            "invalidateQueries({ queryKey: keys.list() });",
            "client.other({ queryKey: keys.list() });",
            "client.invalidateQueries({}, { queryKey: keys.list() });",
            "client.invalidateQueries({ nested: { queryKey: keys.list() } });",
            "client.invalidateQueries({ queryKey: factory({ queryKey: ['billing'] }) });",
            "client.invalidateQueries({ queryKey: keys.list() }); const options = { queryKey };",
            "client.invalidateQueries({ queryKey: ['billing'] as const });",
            "client.refetchQueries({ queryKey: (['billing']) satisfies QueryKey });",
            "client.cancelQueries({ queryKey: <const>['billing'] });",
        ] {
            let facts = summarize_query_syntax(Path::new("hook.ts"), source).unwrap();
            assert!(facts.declares_query_key, "missed declaration: {source}");
        }
        for source in [
            "client.invalidateQueries({ queryKey: keys.list() });",
            "client.refetchQueries({ queryKey });",
            "client['cancelQueries']({ 'queryKey': key });",
            "client.resetQueries({ queryKey: keys.list() }, { nested: true });",
            "// queryKey: ['billing']\nconst label = 'queryKey';",
            "const queryKey = ['billing'];",
            "client.invalidateQueries({ queryKey: keys.list() as QueryKey });",
            "client.invalidateQueries({ queryKey: keys.list() } as const);",
        ] {
            let facts = summarize_query_syntax(Path::new("component.tsx"), source).unwrap();
            assert!(
                !facts.declares_query_key,
                "unexpected declaration: {source}"
            );
        }
    }

    #[test]
    fn mutation_invalidation_contract_belongs_to_each_calls_options() {
        for source in [
            "useMutation({ meta: { invalidates: [] } });",
            "query.useMutation({ meta: { invalidates: [['billing']] } });",
            "query['useMutation']({ 'meta': { 'invalidates': null } });",
            "useMutation({ meta: { invalidates } });",
            "useMutation({ meta: { invalidates: undefined } });",
            "useMutation({ meta: { invalidates: [] } }); useMutation({ meta: { invalidates: keys() } });",
            "// useMutation({})\nconst label = 'useMutation(';",
            "other({ meta: {} });",
            "useMutation({ meta: { invalidates: [] } as const });",
            "useMutation({ meta: { invalidates: [] } satisfies MutationMeta });",
            "useMutation(({ meta: ({ invalidates: [] }) }));",
            "useMutation({ meta: { invalidates: [] } } as const);",
        ] {
            let facts = summarize_query_syntax(Path::new("hook.ts"), source).unwrap();
            assert!(
                !facts.has_uncontracted_mutation,
                "unexpected mutation: {source}"
            );
        }
        for source in [
            "useMutation({});",
            "query.useMutation ({});",
            "query['useMutation']({});",
            "useMutation();",
            "useMutation(options());",
            "useMutation({ meta: { toast: 'saved' } });",
            "useMutation({ meta });",
            "useMutation({ meta: { nested: { invalidates: [] } } });",
            "useMutation({ nested: { meta: { invalidates: [] } } });",
            "useMutation({}, { meta: { invalidates: [] } });",
            "useMutation({}); const options = { meta: { invalidates: [] } };",
            "useMutation({}); useMutation({ meta: { invalidates: [] } });",
            "useMutation({ meta: { invalidates: [] } }); useMutation({});",
            "useMutation({ /* meta: { invalidates: [] } */ });",
            "useMutation({ meta: { toast: 'saved' } as const });",
        ] {
            let facts = summarize_query_syntax(Path::new("hook.ts"), source).unwrap();
            assert!(facts.has_uncontracted_mutation, "missed mutation: {source}");
        }
    }

    #[test]
    fn query_syntax_returns_typed_parse_errors() {
        for (file, source) in [("hook.ts", "const = ;"), ("hook.unknown", "")] {
            let path = Path::new(file);
            assert!(matches!(
                summarize_query_syntax(path, source),
                Err(OnionCryError::ParseSource { path: error_path, .. }) if error_path == path
            ));
        }
    }
}
