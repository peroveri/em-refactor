use rustc_hir::{BodyId, FnDecl, HirId, Pat, PatKind};
use rustc_hir::intravisit::{FnKind, walk_fn, walk_pat, walk_crate, NestedVisitorMap, Visitor};
use rustc::hir::map::Map;
use rustc::ty::TyCtxt;
use rustc_span::Span;

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
    field_ident: String,
) -> StructPatternCollection {
    let mut v = StructPatternCollector {
        tcx,
        struct_hir_id,
        patterns: StructPatternCollection {
            new_bindings: vec![],
            other: vec![]
        },
        field_ident,
        body_id: None,
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
    field_ident: String,
    body_id: Option<BodyId>,
}

impl StructPatternCollector<'_> {
    fn path_resolves_to_struct(&self, pat: &Pat) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(pat.hir_id.owner_def_id());

        if let Some(pat_type) = typecheck_table.pat_ty_opt(pat) {
            if let Some(adt_def) = pat_type.ty_adt_def() {
                if adt_def.did == self.struct_hir_id.owner_def_id() {
                    return true;
                }
            }
        }

        false
    }
}

impl<'v> Visitor<'v> for StructPatternCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: FnKind<'v>,
        fd: &'v FnDecl,
        body_id: BodyId,
        s: Span,
        h: HirId,
    ) {
        self.body_id = Some(body_id);
        walk_fn(self, fk, fd, body_id, s, h);
    }
    fn visit_pat(&mut self, p: &'v Pat) {
        if let PatKind::Struct(_, fields, _) = &p.kind {
            if self.path_resolves_to_struct(p) {
                for fp in fields.iter() {
                    if format!("{}", fp.ident) == self.field_ident {
                        if let PatKind::Wild = fp.pat.kind {
                            // Wildcard patterns match anything, so no changes are needed
                        } else if let PatKind::Binding(_, hir_id, ..) = fp.pat.kind {
                            self.patterns.new_bindings.push(hir_id);
                        } else {
                            self.patterns.other.push(fp.span);
                        }
                    }
                }
            }
        }
        walk_pat(self, p);
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> HirId {
        let field =
            super::super::struct_def_field_collector::collect_field(tcx, create_test_span(11, 16))
                .unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_pattern_collector_should_collect_match_pattern() {
        run_after_analysis(create_program_match(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, hir_id, "field".to_owned());

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_if_let_pattern() {
        run_after_analysis(create_program_if_let(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, hir_id, "field".to_owned());

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_not_collect_struct_without_field() {
        run_after_analysis(create_program_match_without_field(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, "field".to_owned());

            assert_eq!(fields.other.len(), 0);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_struct_self_type() {
        run_after_analysis(create_program_self_type(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, "field".to_owned());

            assert_eq!(fields.other.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_struct_self_type_wildcard() {
        run_after_analysis(create_program_self_type_wildcard(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, "field".to_owned());

            assert_eq!(fields.new_bindings.len(), 1);
        });
    }
}
