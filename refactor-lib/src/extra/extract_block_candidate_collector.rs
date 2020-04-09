use rustc_ast::ast::Block;
use rustc_ast::visit::{Visitor, walk_block, walk_crate};
use rustc_span::Span;
use crate::refactoring_invocation::{AstContext, QueryResult};
/// 
/// Modules that are not inlined (from files) are not visited in the pre macro exp. AST,
/// so we should use the post macro exp. AST
/// 
pub fn collect_extract_block_candidates(ctx: &AstContext) -> QueryResult<Vec<Span>> {
    let mut visitor = ExtractBlockCandidateVisitor{candidates: vec![]};

    let crate_ = ctx.get_crate();

    walk_crate(&mut visitor, crate_);

    Ok(visitor.candidates)
}

struct ExtractBlockCandidateVisitor {
    candidates: Vec<Span>
}

impl<'ast> Visitor<'ast> for ExtractBlockCandidateVisitor {
    fn visit_block(&mut self, b: &'ast Block) {
        if b.span.from_expansion() {
            return;
        }
        
        let spans = b.stmts.iter().map(|s| s.span.source_callsite()).collect::<Vec<_>>();
        let l = spans.len();

        for i in 0..l {
            let si = spans[i];
            for j in i..l {
                let sj = spans[j];
                if i == j || si != sj {
                    self.candidates.push(si.with_hi(sj.hi()));
                }
            }
        }
        walk_block(self, b);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::assert_ast_success3;

    fn map() -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        Box::new(|ast| 
            Ok(
                collect_extract_block_candidates(ast)?
                .iter()
                .map(|span| ast.get_source(*span))
                .collect::<Vec<_>>()
            )
        )
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_1() {
        assert_ast_success3(
            r#"fn foo() { 
                1;
            }"#,
            map,
            vec!["1;".to_string()]
        );
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_2() {
        assert_ast_success3(
            r#"fn foo() -> u32 { 
                1
            }"#,
            map,
            vec!["1".to_string()]
        );
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_3() {
        assert_ast_success3(
            r#"fn foo() -> u32 { 
                1; 2
            }"#,
            map,
            vec!["1;".to_string(), "1; 2".to_string(), "2".to_string()]
        );
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_4() {
        assert_ast_success3(
r#"mod a {
    struct S;
    impl S {
        fn foo() {
            loop {
                let _ = 1;
            }
        }
    }
}"#,
            map,
            vec![
r#"loop {
                let _ = 1;
            }"#.to_string(), 
"let _ = 1;".to_string()]
        );
    }
}
