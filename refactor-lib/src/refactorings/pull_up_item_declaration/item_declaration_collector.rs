use crate::refactorings::visitors::collect_block;
use rustc::ty::TyCtxt;
use rustc_span::{Span};
use rustc_hir::{StmtKind};

pub struct ItemDeclarations {
    pub items: Vec<Span>,
    pub selection_start: Span
}

pub fn collect_item_declarations<'v>(tcx: TyCtxt<'v>, span: Span) -> Option<ItemDeclarations> {
    if let Some(selection) = collect_block(tcx, span) {
        let mut items = vec![];

        for stmt in selection.get_stmts().iter() {
            match stmt.kind {
                StmtKind::Item(item_id) => {
                    let item = tcx.hir().item(item_id.id);
                    items.push(item.span);
                },
                _ => {}
            };
        }
        Some(ItemDeclarations {
            items,
            selection_start: selection.get_stmts().first().unwrap().span
        })
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quote::quote;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactorings::utils::get_source;

    #[test]
    fn item_delcaration_collector() {
        run_after_analysis(quote! {
            fn f ( ) { 0 ; fn foo ( ) { } foo ( ) ; }
        }, |tcx| {
            assert_eq!(get_source(tcx, create_test_span(11, 29)), "0 ; fn foo ( ) { }");

            let selection = collect_item_declarations(tcx, create_test_span(10, 32)).unwrap();

            assert_eq!(1, selection.items.len());
            assert_eq!(get_source(tcx, selection.items[0]), "fn foo ( ) { }");
        });
    }
}
