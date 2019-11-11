use rustc::hir::{
    self,
    intravisit::{walk_crate, NestedVisitorMap, Visitor},
};
use rustc::ty::TyCtxt;
use syntax_pos::Span;

///
/// Collect all places where a given struct occurs in a pattern. Field_ident should also occur in the pattern.
/// 
/// We collect spans of all StructPatternFields where PathInExpression has the same type as `struct_hir_id` and StructPatternField is `field_ident`
/// 
/// # Example
/// 
/// ```
/// match (foo) {
///   /* start */ S {field: 0} /* end */ => {}
/// }
/// ```
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
    struct_hir_id: hir::HirId,
    field_ident: String,
) -> Vec<Span> {
    let mut v = StructPatternCollector {
        tcx,
        struct_hir_id,
        field: vec![],
        field_ident,
        body_id: None,
    };

    walk_crate(&mut v, tcx.hir().krate());

    v.field
}

struct StructPatternCollector<'v> {
    tcx: TyCtxt<'v>,
    struct_hir_id: hir::HirId,
    field: Vec<Span>,
    field_ident: String,
    body_id: Option<hir::BodyId>,
}

impl StructPatternCollector<'_> {
    fn path_resolves_to_struct(&self, qpath: &hir::QPath, h: hir::HirId) -> bool {
        let typecheck_table = self.tcx.typeck_tables_of(h.owner_def_id());

        if let hir::QPath::Resolved(Some(ty), _) = qpath {
            let qp_type = typecheck_table.node_type(ty.hir_id);
            let struct_type = typecheck_table.node_type(self.struct_hir_id);
            rustc::ty::TyS::same_type(struct_type, qp_type)
        } else {
            let res = typecheck_table.qpath_res(qpath, h);
            let res_def_id = res.def_id();
            res_def_id == self.struct_hir_id.owner_def_id()
        }
    }
}

impl<'v> Visitor<'v> for StructPatternCollector<'v> {
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, 'v> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_fn(
        &mut self,
        fk: hir::intravisit::FnKind<'v>,
        fd: &'v hir::FnDecl,
        body_id: hir::BodyId,
        s: Span,
        h: hir::HirId,
    ) {
        self.body_id = Some(body_id);
        hir::intravisit::walk_fn(self, fk, fd, body_id, s, h);
    }
    fn visit_pat(&mut self, p: &'v hir::Pat) {
        if let hir::PatKind::Struct(qpath, fields, _) = &p.kind {
            if self.path_resolves_to_struct(qpath, p.hir_id) {
                for fp in fields {
                    if format!("{}", fp.ident) == self.field_ident {
                        self.field.push(fp.span);
                    }
                }
            }
        }
        hir::intravisit::walk_pat(self, p);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{create_test_span, run_test};
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
    fn get_struct_hir_id(tcx: TyCtxt<'_>) -> hir::HirId {
        let field =
            super::super::struct_field_collector::collect_field(tcx, create_test_span(11, 16))
                .unwrap();
        let struct_def_id = field.hir_id.owner_def_id();
        tcx.hir().as_local_hir_id(struct_def_id).unwrap()
    }

    #[test]
    fn struct_pattern_collector_should_collect_match_pattern() {
        run_test(create_program_match(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, hir_id, "field".to_owned());

            assert_eq!(fields.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_collect_if_let_pattern() {
        run_test(create_program_if_let(), |tcx| {
            let hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, hir_id, "field".to_owned());

            assert_eq!(fields.len(), 1);
        });
    }
    #[test]
    fn struct_pattern_collector_should_not_collect_struct_without_field() {
        run_test(create_program_match_without_field(), |tcx| {
            let struct_hir_id = get_struct_hir_id(tcx);
            let fields = collect_struct_patterns(tcx, struct_hir_id, "field".to_owned());

            assert_eq!(fields.len(), 0);
        });
    }
}
