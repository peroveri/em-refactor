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
    use crate::test_utils::{run_ast_query, TestContext};

    fn map(_: TestContext) -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
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
        let input = r#"
        fn foo() { 
            1;
        }"#;
        let expected = Ok(vec!["1;".to_owned()]);

        let actual = run_ast_query(input, map);
        
        assert_eq!(actual, expected);
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_2() {
        let input = r#"
        fn foo() -> u32 { 
            1
        }"#;
        let expected = Ok(vec!["1".to_owned()]);

        let actual = run_ast_query(input, map);
        
        assert_eq!(actual, expected);
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_3() {
        let input = r#"
        fn foo() -> u32 { 
            1; 2
        }"#;
        let expected = Ok(vec![
            "1;".to_owned(), 
            "1; 2".to_owned(), 
            "2".to_owned()]);

        let actual = run_ast_query(input, map);
        
        assert_eq!(actual, expected);
    }
    #[test]
    fn local_variable_use_collector_should_collect_uses_4() {
        let input = r#"
        mod a {
            struct S;
            impl S {
                fn foo() {
                    loop {
                        let _ = 1;
                    }
                }
            }
        }"#;
        let expected = Ok(vec![
            r#"loop {
                        let _ = 1;
                    }"#.to_owned(),
            "let _ = 1;".to_owned()]);

        let actual = run_ast_query(input, map);
        
        assert_eq!(actual, expected);
    }
}
