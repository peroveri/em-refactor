use super::utils::{map_change_from_span, get_source};
use item_declaration_collector::collect_item_declarations;
use crate::refactoring_invocation::{FileStringReplacement, RefactoringErrorInternal};
use rustc::ty::TyCtxt;
use rustc_span::{BytePos, Span};

mod item_declaration_collector;

/// Given a selection within a block, contiguous statements (0..n) and an expression (0|1)
/// It should pull up item declarations occuring at this block level
/// These item declarations can only be found in the selection of statements (if they are item decls.)
pub fn do_refactoring(tcx: TyCtxt, span: Span) -> Result<Vec<FileStringReplacement>, RefactoringErrorInternal> {
    if let Some(item_declarations) = collect_item_declarations(tcx, span) {
        let source_map = tcx.sess.source_map();
        let mut res = vec![];
        res.push(map_change_from_span(
            source_map,
            span.with_lo(span.lo() - BytePos(0)).shrink_to_lo(),
            item_declarations.items.iter().map(|s| get_source(tcx, *s)).collect::<Vec<_>>().join("")
        ));
        for delete in item_declarations.items {
            res.push(map_change_from_span(
                source_map,
                delete,
                "".to_owned(),
            ));
        }
        Ok(res)
    } else {
        Err(RefactoringErrorInternal::invalid_selection_with_code(
            span.lo().0,
            span.hi().0,
            &get_source(tcx, span)
        ))
    }
}

#[cfg(test)]
mod test {
    use super::test_util::{assert_success, assert_err};
    use quote::quote;
    use super::RefactoringErrorInternal;

    #[test]
    fn pull_up_item_declaration_fn_decl() {
        assert_success(quote! {
            fn f ( ) { 0 ; fn g ( ) { } g ( ) ; }
        }, (10, 36),  
        "fn f ( ) {fn g ( ) { } 0 ;  g ( ) ; }");
    }
    #[test]
    fn pull_up_item_declaration_2_fn_decl() {
        assert_success(quote! {
            fn f ( ) { 0 ; fn g ( ) { } fn h ( ) { } g ( ) ; }
        }, (10, 49),  
        "fn f ( ) {fn g ( ) { }fn h ( ) { } 0 ;   g ( ) ; }");
    }
    #[test]
    fn pull_up_item_declaration_no_items() {
        assert_success(quote! {
            fn f ( ) { 0 ; 1 ; }
        }, (10, 19),  
        "fn f ( ) { 0 ; 1 ; }");
    }
    #[test]
    fn pull_up_item_declaration_macro_inv() {
        assert_success(quote! {
            fn f ( ) { print ! ( "{}" , 1 ) ; fn g ( ) { } print ! ( "{}" , 2 ) ; }
        }, (10, 70),  
        r#"fn f ( ) {fn g ( ) { } print ! ( "{}" , 1 ) ;  print ! ( "{}" , 2 ) ; }"#);
    }
    #[test]
    fn pull_up_item_declaration_invalid_selection() {
        assert_err(quote! {
            fn f ( ) { 0 ; 1 ; }
        }, (0, 4),  
        RefactoringErrorInternal::invalid_selection_with_code(0, 4, "fn f"));
    }
}
#[cfg(test)]
mod test_util {
    use super::*;
    use crate::{create_test_span, run_after_analysis};
    use crate::refactoring_invocation::MyRefactorCallbacks;
    pub fn assert_success(prog: quote::__rt::TokenStream, span: (u32, u32), expected: &str) {
        run_after_analysis(prog, | tcx | {
            let actual = do_refactoring(tcx, create_test_span(span.0, span.1)).unwrap();
            let res = MyRefactorCallbacks::get_file_content(&actual, tcx.sess.source_map()).unwrap();

            assert_eq!(res, expected);
        })
    }
    pub fn assert_err(prog: quote::__rt::TokenStream, span: (u32, u32), expected: RefactoringErrorInternal) {
        run_after_analysis(prog, | tcx | {
            let actual = do_refactoring(tcx, create_test_span(span.0, span.1)).unwrap_err();

            assert_eq!(actual, expected);
        })
    }
}