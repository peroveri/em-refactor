use rustc_hir::{HirId, Item, Pat, PatKind};
use rustc_hir::intravisit::{walk_item, walk_pat, walk_crate, NestedVisitorMap, Visitor};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;
use super::super::box_field::StructPatternCollection;
use if_chain::if_chain;

///
/// Collect all places where a given struct occurs in a pattern. Field_ident should also occur in the pattern.
/// 
/// We collect spans of all StructPatternFields where PathInExpression has the same type as `struct_hir_id` and StructPatternField is `field_ident`
/// 
/// # Example
/// given:
/// ```
/// match (foo) {
///   S {field: 0} => {}
///   |          |
///   x          y
/// }
/// ```
/// then `collect_struct_patterns(S, "ident")` would return a single byte range `(x, y)`
/// 
/// # Grammar
/// ```
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
    tcx: TyCtxt,
    struct_hir_id: HirId,
    field_ident: &str,
) -> StructPatternCollection {
    let mut v = StructPatternCollector {
        tcx,
        struct_hir_id,
        patterns: StructPatternCollection {
            new_bindings: vec![],
            other: vec![]
        },
        field_ident: field_ident.to_string(),
    };

    walk_crate(&mut v, tcx.hir().krate());

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
    use crate::refactorings::visitors::collect_field;
    use crate::{create_test_span, run_after_analysis};
    use quote::quote;

    fn create_program_match() -> quote::__rt::TokenStream {
        quote! {
            struct S { field: u32 }
            fn foo() {
                match (S {field: 0}) {
                    S {field: 0} => {},
                    _ => {}
                }
            }
        }
    }
    fn create_program_match_without_field() -> quote::__rt::TokenStream {
        quote! {
            struct S { field: u32 }
            fn foo() {
                match (S {field: 0}) {
                    S {..} => {},
                    _ => {}
                }
            }
        }
    }

    fn create_program_if_let() -> quote::__rt::TokenStream {
        quote! {
            struct S { field: u32 }
            fn foo() {
                if let (S{field: 0}) = (S{field: 1}) {
                }
            }
        }
    }
    fn create_program_self_type() -> quote::__rt::TokenStream {
        quote! {
            struct S { field: u32 }
            impl S {
                fn foo(s: Self) {
                    match s {
                        Self {field: 0} => {},
                        _ => {}
                    }
                }
            }
        }
    }
    fn create_program_self_type_wildcard() -> quote::__rt::TokenStream {
        quote! {
            struct S { field: u32 }
            impl S {
                fn foo(s: Self) {
                    match s {
                        Self {field} => {},
                        _ => {}
                    }
                }
            }
        }
    }
    fn create_program_match_6() -> quote::__rt::TokenStream {
        quote! {
            # [ derive ( Eq , PartialEq , Ord , PartialOrd , Clone , Hash , Default , Debug ) ] struct S { foo : u32 }
        }
    }
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(11, 16)).unwrap();
        let struct_def_id = field.hir_id.owner.to_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    fn get_struct_hir_id6(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(95, 98)).unwrap();
        let struct_def_id = field.hir_id.owner.to_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }
    #[test]
    fn struct_pattern_collector_should_collect_match_pattern() {
        run_after_analysis(create_program_match(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_named_patterns(tcx, hir_id, "field");

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_if_let_pattern() {
        run_after_analysis(create_program_if_let(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_named_patterns(tcx, hir_id, "field");

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_not_collect_struct_without_field() {
        run_after_analysis(create_program_match_without_field(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_named_patterns(tcx, struct_hir_id, "field");

            assert_eq!(fields.other.len(), 0);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_struct_self_type() {
        run_after_analysis(create_program_self_type(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_named_patterns(tcx, struct_hir_id, "field");

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_struct_self_type_wildcard() {
        run_after_analysis(create_program_self_type_wildcard(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_named_patterns(tcx, struct_hir_id, "field");

            assert_eq!(fields.new_bindings.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_not_collect_in_std_derives() {
        run_after_analysis(create_program_match_6(), |tcx| {
            let struct_hir_id = get_struct_hir_id6(tcx);
            let fields = collect_struct_named_patterns(tcx, struct_hir_id, "foo");

            assert_eq!(fields.new_bindings.len(), 0);
            assert_eq!(fields.other.len(), 0);
        });
    }
}
