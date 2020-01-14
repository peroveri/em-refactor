use rustc_hir::{HirId, Pat, PatKind};
use rustc_hir::intravisit::{walk_pat, walk_crate, NestedVisitorMap, Visitor};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;
use super::StructFieldType;
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
pub fn collect_struct_patterns(
    tcx: TyCtxt,
    struct_hir_id: HirId,
    field_ident: StructFieldType,
) -> StructPatternCollection {
    let mut v = StructPatternCollector {
        tcx,
        struct_hir_id,
        patterns: StructPatternCollection {
            new_bindings: vec![],
            other: vec![]
        },
        field_ident,
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.patterns
}

pub struct StructPatternCollection {
    pub new_bindings: Vec<HirId>,
    pub other: Vec<Span>
}

struct StructPatternCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: HirId,
    patterns: StructPatternCollection,
    field_ident: StructFieldType,
}

impl StructPatternCollector<'_> {
    fn path_resolves_to_struct(&self, pat: &Pat) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(pat.hir_id.owner_def_id());
        if_chain! {
            if let Some(pat_type) = typecheck_table.pat_ty_opt(pat);
            if let Some(adt_def) = pat_type.ty_adt_def();
            if adt_def.did == self.struct_hir_id.owner_def_id();
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
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_pat(&mut self, p: &'v Pat) {
        if self.path_resolves_to_struct(p) {
            if_chain! {
                if let PatKind::Struct(_, fields, _) = &p.kind;
                if let StructFieldType::Named(name) = &self.field_ident;
                if let Some(fp) = fields.iter().find(|e| {&format!("{}", e.ident) == name});
                then {
                    self.struct_pattern_used(&fp.pat.kind, fp.span);
                }
            }
            if_chain! {
                if let PatKind::TupleStruct(_, fields, _) = &p.kind;
                if let StructFieldType::Tuple(index) = &self.field_ident;
                if let Some(field) = fields.get(*index);
                then {
                    self.struct_pattern_used(&field.kind, field.span);
                }
            }
        }
        walk_pat(self, p);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::struct_def_field_collector::collect_field;
    use crate::{create_test_span, run_after_analysis};
    use quote::quote;

    fn tup(i: usize) -> StructFieldType {
        StructFieldType::Tuple(i)
    }
    fn field(s: &str) -> StructFieldType {
        StructFieldType::Named(s.to_string())
    }
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
    fn create_program_self_tuple_wildcard() -> quote::__rt::TokenStream {
        quote! {
            struct S ( i32 );
            impl S {
                fn foo(s: Self) {
                    match s {
                        Self (field) => {},
                        _ => {}
                    }
                }
            }
        }
    }
    fn create_program_self_tuple_pattern() -> quote::__rt::TokenStream {
        quote! {
            struct S ( i32 );
            impl S {
                fn foo(s: Self) {
                    match s {
                        Self (field @ 0) => {},
                        _ => {}
                    }
                }
            }
        }
    }
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(11, 16)).unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }
    fn get_tuple_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let (field, _) = collect_field(tcx, create_test_span(11, 14)).unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_pattern_collector_should_collect_match_pattern() {
        run_after_analysis(create_program_match(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, hir_id, field("field"));

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_if_let_pattern() {
        run_after_analysis(create_program_if_let(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, hir_id, field("field"));

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_not_collect_struct_without_field() {
        run_after_analysis(create_program_match_without_field(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, field("field"));

            assert_eq!(fields.other.len(), 0);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_struct_self_type() {
        run_after_analysis(create_program_self_type(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, field("field"));

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_struct_self_type_wildcard() {
        run_after_analysis(create_program_self_type_wildcard(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, field("field"));

            assert_eq!(fields.new_bindings.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_struct_self_tuple_wildcard() {
        run_after_analysis(create_program_self_tuple_wildcard(), |tcx| {
            let struct_hir_id = get_tuple_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, tup(0));

            assert_eq!(fields.new_bindings.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_struct_self_tuple_pattern() {
        run_after_analysis(create_program_self_tuple_pattern(), |tcx| {
            let struct_hir_id = get_tuple_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, tup(0));

            assert_eq!(fields.new_bindings.len(), 0);
            assert_eq!(fields.other.len(), 1);
        });
    }
}
