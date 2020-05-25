use rustc_hir::{HirId, Item, Pat, PatKind};
use rustc_hir::intravisit::{walk_item, walk_pat, walk_crate, NestedVisitorMap, Visitor};
use rustc_middle::hir::map::Map;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use super::super::box_field::StructPatternCollection;
use if_chain::if_chain;
use crate::refactoring_invocation::TyContext;

///
/// Collect all places where a given struct occurs in a pattern. Field_ident should also occur in the pattern.
/// 
/// We collect spans of all StructPatternFields where PathInExpression has the same type as `struct_hir_id` and StructPatternField is `field_ident`
/// 
/// # Example
/// given:
/// ```example
/// match (foo) {
///   S {field: 0} => {}
///   |          |
///   x          y
/// }
/// ```
/// then `collect_struct_patterns(S, "ident")` would return a single byte range `(x, y)`
/// 
/// # Grammar
/// ```example
/// StructPattern :
///   PathInExpression {
///      StructPatternElements ?
///   }
/// StructPatternElements :
///      StructPatternFields (, | , StructPatternEtCetera)?
///   | StructPatternEtCetera
/// StructPatternFields :
///   StructPatternField (, StructPatternField) \*
/// ```
/// [Struct pattern grammar](https://doc.rust-lang.org/stable/reference/patterns.html#struct-patterns)
pub fn collect_struct_named_patterns(
    tcx: &TyContext,
    struct_hir_id: HirId,
    field_ident: &str,
) -> StructPatternCollection {
    let mut v = StructPatternCollector {
        tcx: tcx.0,
        struct_hir_id,
        patterns: StructPatternCollection {
            new_bindings: vec![],
            other: vec![]
        },
        field_ident: field_ident.to_string(),
    };

    walk_crate(&mut v, tcx.0.hir().krate());

    v.patterns
}

struct StructPatternCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: HirId,
    patterns: StructPatternCollection,
    field_ident: String,
}

impl StructPatternCollector<'_> {
    fn path_resolves_to_struct(&self, pat: &Pat) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(pat.hir_id.owner.to_def_id());
        if_chain! {
            if let Some(pat_type) = typecheck_table.pat_ty_opt(pat);
            if let Some(adt_def) = pat_type.ty_adt_def();
            if adt_def.did == self.struct_hir_id.owner.to_def_id();
            then {
                true
            } else {
                false
            }
        }
    }
    fn struct_pattern_used(&mut self, pattern_kind: &PatKind, span: Span) {
        match pattern_kind {
            // Wildcard patterns match anything, so no changes are needed
            PatKind::Wild => {},
            PatKind::Binding(_, hir_id, _, None) => { 
                self.patterns.new_bindings.push(*hir_id);
            },
            PatKind::Binding(_, hir_id, _, Some(at_pattern)) => {
                if let PatKind::Wild = at_pattern.kind {
                    self.patterns.new_bindings.push(*hir_id);
                } else {
                    self.patterns.other.push(span);
                }
            },
            _ =>  {
                self.patterns.other.push(span);
            }
        }
    } 
}

impl<'v> Visitor<'v> for StructPatternCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map(&mut self) -> NestedVisitorMap<Self::Map> {
        NestedVisitorMap::All(self.tcx.hir())
    }
    fn visit_pat(&mut self, p: &'v Pat) {
        if self.path_resolves_to_struct(p) {
            if_chain! {
                if let PatKind::Struct(_, fields, _) = &p.kind;
                if let Some(fp) = fields.iter().find(|e| {format!("{}", e.ident) == self.field_ident});
                then {
                    self.struct_pattern_used(&fp.pat.kind, fp.span);
                }
            }
        }
        walk_pat(self, p);
    }
    fn visit_item(&mut self, i: &'v Item<'v>) {
        if !super::is_impl_from_std_derive_expansion(&i) {
            walk_item(self, i);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::run_ty_query;
    use crate::refactoring_invocation::QueryResult;
    use crate::refactorings::utils::get_struct_hir_id;
    use crate::refactorings::visitors::collect_field;

    fn map(file_name: String, from: u32, to: u32) -> Box<dyn Fn(&TyContext) -> QueryResult<(usize, Vec<String>)> + Send> {
        Box::new(move |ty| {
            let span = ty.source().map_span(&file_name, from, to)?;

            let (field, _) = collect_field(ty.0, span).unwrap();
            let hir_id = get_struct_hir_id(ty.0, field);
            let fields = collect_struct_named_patterns(ty, hir_id, &field.ident.as_str().to_string());

            Ok((
                fields.new_bindings.len(),
                fields.other.iter().map(|s| ty.get_source(*s)).collect::<Vec<_>>()))
        })
    }
    #[test]
    fn should_collect_match_pattern() {
        let input = r#"
            struct S { /*START*/field/*END*/: u32 }
            fn foo() {
                match (S {field: 0}) {
                    S {field: 0} => {},
                    _ => {}
                }
            }"#;

        let expected = Ok((
            0,
            vec!["field: 0".to_owned()],
        ));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_if_let_pattern() {
        let input = r#"
            struct S { /*START*/field/*END*/: u32 }
            fn foo() {
                if let (S{field: 0}) = (S{field: 1}) {
                }
            }"#;

        let expected = Ok((
            0,
            vec!["field: 0".to_owned()],
        ));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_not_collect_struct_without_field() {
        let input = r#"
            struct S { /*START*/field/*END*/: u32 }
            fn foo() {
                match (S {field: 0}) {
                    S {..} => {},
                    _ => {}
                }
            }"#;

        let expected = Ok((
            0,
            vec![],
        ));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_struct_self_type() {
        let input = r#"
            struct S { /*START*/field/*END*/: u32 }
            impl S {
                fn foo(s: Self) {
                    match s {
                        Self {field: 0} => {},
                        _ => {}
                    }
                }
            }"#;

        let expected = Ok((
            0,
            vec!["field: 0".to_owned()],
        ));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_struct_self_type_wildcard() {
        let input = r#"
            struct S { /*START*/field/*END*/: u32 }
            impl S {
                fn foo(s: Self) {
                    match s {
                        Self {field} => {},
                        _ => {}
                    }
                }
            }"#;

        let expected = Ok((
            1,
            vec![],
        ));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_not_collect_in_std_derives() {
        let input = r#"
            #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Hash, Default, Debug)]
            struct S { /*START*/field/*END*/: u32 }"#;

        let expected = Ok((
            0,
            vec![],
        ));

        let actual = run_ty_query(input, map);

        assert_eq!(actual, expected);
    }
}
