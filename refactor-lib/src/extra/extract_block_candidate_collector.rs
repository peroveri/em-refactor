use rustc_ast::ast::{Block, Mac};
use rustc_ast::visit::{Visitor, walk_block, walk_crate};
use rustc_span::Span;

pub fn collect_extract_block_candidates<'tcx>(queries: &'tcx rustc_interface::Queries<'_>) -> Vec<Span> {
    let mut v = ExtractBlockCandidateVisitor{candidates: vec![]};

    let crate_ = &*queries.parse().unwrap().peek_mut();

    walk_crate(&mut v, crate_);

    v.candidates
}

struct ExtractBlockCandidateVisitor {
    candidates: Vec<Span>
}

impl<'ast> Visitor<'ast> for ExtractBlockCandidateVisitor {
    fn visit_block(&mut self, b: &'ast Block) {
        let l = b.stmts.len();
        for i in 0..l {
            for j in i..l {
                let si = b.stmts[i].span;
                let sj = b.stmts[j].span;
                self.candidates.push(si.with_hi(sj.hi()));
            }
        }
        walk_block(self, b);
    }

    fn visit_mac(&mut self, _mac: &'ast Mac) {
        // Override to prevent `visit_mac disabled by default`
    }
}

#[cfg(test)]
mod test {
    use crate::refactorings::utils::get_source_from_compiler;
    use super::*;
    use crate::{create_test_span, run_after_parsing};
    use quote::quote;
    use quote::__rt::TokenStream;

    fn create_program_1() -> TokenStream {
        quote! {
            fn foo ( ) { 1 ; }
        }
    }
    fn create_program_2() -> TokenStream {
        quote! {
            fn foo ( ) -> i32 { 1 }
        }
    }
    fn create_program_3() -> TokenStream {
        quote! {
            fn foo ( ) -> i32 { 1 ; 2 }
        }
    }
    fn create_program_4() -> TokenStream {
        quote! {
            mod a {
                struct S;
                impl S {
                    fn foo() {
                        loop {
                            let _ = 1 ;
                        }
                    }
                }
            }
        }
    }
    fn create_span(v: Vec<(u32, u32)>) -> Vec<Span> {
        v.iter().map(|a| create_test_span(a.0, a.1)).collect::<Vec<_>>()
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_1() {
        run_after_parsing(create_program_1(), |queries, _| {
            let expected = create_span(vec![(13, 16)]);
            let actual = collect_extract_block_candidates(&queries);
            assert_eq!(expected, actual);
        });
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_2() {
        run_after_parsing(create_program_2(), |queries, _| {
            let expected = create_span(vec![(20, 21)]);
            let actual = collect_extract_block_candidates(&queries);
            assert_eq!(expected, actual);
        });
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_3() {
        run_after_parsing(create_program_3(), |queries, _| {
            let expected = create_span(vec![(20, 23), (20, 25), (24, 25)]);
            let actual = collect_extract_block_candidates(&queries);
            assert_eq!(expected, actual);
        });
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_4() {
        run_after_parsing(create_program_4(), |queries, compiler| {
            let expected0 = "loop { let _ = 1 ; }";
            let expected1 = "let _ = 1 ;";
            let actual = collect_extract_block_candidates(&queries);

            assert_eq!(actual.len(), 2);

            assert_eq!(get_source_from_compiler(compiler, actual[0]), expected0);
            assert_eq!(get_source_from_compiler(compiler, actual[1]), expected1);
        });
    }
}
