use rustc_ast::ast::StructField;
use rustc_ast::visit::{Visitor, walk_crate};
use rustc_span::Span;
use crate::refactoring_invocation::{AstContext, QueryResult};

#[derive(PartialEq, Copy, Clone)]
pub enum CollectFieldMode {
    Named,
    Tuple,
    All
}

pub fn collect_box_field_all_candidates(ctx: &AstContext) -> QueryResult<Vec<Span>> {
    collect_box_field_candidates(ctx, CollectFieldMode::All)
}
pub fn collect_box_field_namede_candidates(ctx: &AstContext) -> QueryResult<Vec<Span>> {
    collect_box_field_candidates(ctx, CollectFieldMode::Named)
}
pub fn collect_box_field_tuple_candidates(ctx: &AstContext) -> QueryResult<Vec<Span>> {
    collect_box_field_candidates(ctx, CollectFieldMode::Tuple)
}
pub fn collect_box_field_candidates(ctx: &AstContext, mode: CollectFieldMode) -> QueryResult<Vec<Span>> {
    
    let mut visitor = ExtractBlockCandidateVisitor{candidates: vec![], mode};
    let crate_ = ctx.get_crate();

    walk_crate(&mut visitor, crate_);

    Ok(visitor.candidates)
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
        if b.span.from_expansion() {
            return;
        }
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::assert_ast_success3;
    fn map_all() -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        map(CollectFieldMode::All)
    }
    fn map_named() -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        map(CollectFieldMode::Named)
    }
    fn map_tuple() -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        map(CollectFieldMode::Tuple)
    }

    fn map(mode: CollectFieldMode) -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        Box::new(
            move |ast| 
            Ok(
                collect_box_field_candidates(ast, mode)?
                .iter()
                .map(|span| ast.get_source(*span))
                .collect::<Vec<_>>()
            )
        )
    }
    #[test]
    fn box_named_field_candidate_collector_should_collect_named() {
        assert_ast_success3(
            "struct G {field: i32}",
            map_named,
            vec!["field".to_string()]
        );
    }
    #[test]
    fn box_named_field_candidate_collector_should_collect_tuple() {
        assert_ast_success3(
            "struct H(u32);",
            map_tuple,
            vec!["u32".to_string()]
        );
    }
    #[test]
    fn box_named_field_candidate_collector_should_collect_all() {
        assert_ast_success3(
            r#"struct H(u32);
            struct G {field: i32}"#,
            map_all,
            vec!["u32".to_string(), "field".to_string()]
        );
    }
    #[test]
    fn box_named_field_candidate_collector_should_collect_derive() {
        assert_ast_success3(
            r#"#[derive(Clone)]
            struct G {field: i32}"#,
            map_all,
            vec!["field".to_string()]
        );
    }
}
