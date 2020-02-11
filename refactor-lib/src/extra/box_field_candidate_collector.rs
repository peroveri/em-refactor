use syntax::visit::{Visitor, walk_crate};
use syntax::ast::{StructField, Mac};
use rustc_span::Span;

#[derive(PartialEq)]
pub enum CollectFieldMode {
    Named,
    Tuple,
    All
}

pub fn collect_box_field_candidates<'tcx>(queries: &'tcx rustc_interface::Queries<'_>, mode: CollectFieldMode) -> Vec<Span> {
    let mut v = ExtractBlockCandidateVisitor{candidates: vec![], mode};

    let crate_ = &*queries.parse().unwrap().peek_mut();

    walk_crate(&mut v, crate_);

    v.candidates
}

struct ExtractBlockCandidateVisitor {
    candidates: Vec<Span>,
    mode: CollectFieldMode
}
impl ExtractBlockCandidateVisitor {
    fn collect_named(&self) -> bool {
        self.mode == CollectFieldMode::Named || self.mode == CollectFieldMode::All
    }
    fn collect_tuple(&self) -> bool {
        self.mode == CollectFieldMode::Tuple || self.mode == CollectFieldMode::All
    }
}

impl<'ast> Visitor<'ast> for ExtractBlockCandidateVisitor {
    fn visit_struct_field(&mut self, b: &'ast StructField) {
        if let Some(id) = b.ident {
            if self.collect_named() {
                self.candidates.push(id.span);
            }
        } else {
            if self.collect_tuple() {
                self.candidates.push(b.span);
            }
        }
    }

    fn visit_mac(&mut self, _mac: &'ast Mac) {
        // Override to prevent `visit_mac disabled by default`
    }
}

#[cfg(test)]
mod test {
    use crate::refactorings::utils::get_source_from_compiler;
    use super::*;
    use crate::run_after_parsing;
    use quote::quote;
    use quote::__rt::TokenStream;

    fn create_program_1() -> TokenStream {
        quote! {
            struct F;
            struct G {field: i32}
            struct H(u32);
            fn foo() {
                let g = G {field: 1};
                let h = H(0);
                match g {
                    G{field: _} => {}
                }
                match h {
                    H(_) => {}
                }
            }
        }
    }
    fn create_program_2() -> TokenStream {
        quote! {
            #[derive(Clone)]
            struct G {field: i32}
        }
    }
    #[test]
    fn box_named_field_candidate_collector_should_collect_named() {
        run_after_parsing(create_program_1(), |queries, compiler| {
            let expected = "field";
            let actual = collect_box_field_candidates(&queries, CollectFieldMode::Named);
            assert_eq!(actual.len(), 1);
            assert_eq!(expected, get_source_from_compiler(compiler, actual[0]));
        });
    }
    #[test]
    fn box_named_field_candidate_collector_should_collect_tuple() {
        run_after_parsing(create_program_1(), |queries, compiler| {
            let expected = "u32";
            let actual = collect_box_field_candidates(&queries, CollectFieldMode::Tuple);
            assert_eq!(expected, get_source_from_compiler(compiler, actual[0]));
            assert_eq!(actual.len(), 1);
        });
    }
    #[test]
    fn box_named_field_candidate_collector_should_collect_all() {
        run_after_parsing(create_program_1(), |queries, compiler| {
            let expected0 = "field";
            let expected1 = "u32";
            let actual = collect_box_field_candidates(&queries, CollectFieldMode::All);
            assert_eq!(actual.len(), 2);
            assert_eq!(expected0, get_source_from_compiler(compiler, actual[0]));
            assert_eq!(expected1, get_source_from_compiler(compiler, actual[1]));
        });
    }
    #[test]
    fn box_named_field_candidate_collector_should_collect_derive() {
        run_after_parsing(create_program_2(), |queries, compiler| {
            let expected = "field";
            let actual = collect_box_field_candidates(&queries, CollectFieldMode::All);
            assert_eq!(actual.len(), 1);
            assert_eq!(expected, get_source_from_compiler(compiler, actual[0]));
        });
    }
}
