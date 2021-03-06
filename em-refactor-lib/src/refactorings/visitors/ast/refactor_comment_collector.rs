use rustc_span::{BytePos, Span};
use em_refactor_lib_types::create_refactor_tool_marker;
use crate::refactoring_invocation::{QueryResult, RefactoringErrorInternal, SourceMapContext};

/**
 * Used for composite refactorings where one micro-refactoring
 * inserts a special tag to mark one or more elements in the AST.
 */
pub fn collect_comments(context: &SourceMapContext) -> QueryResult<Vec<Span>> {
    let tool_id = "refactor-tool";
    let target = format!("/*{}:", tool_id);
    let end_target = "*/";
    let mut result = vec![];

    for file in context.source_map.files().iter() {

        match &file.name {
            rustc_span::FileName::Real(_path) => {

                if let Some(src) = &file.src {
                    let mut offset = 0;

                    while let Some(relative0) = src[offset..].find(&target) {
                        let i0 = relative0 + offset;

                        if let Some(relative1) = src[i0..].find(end_target) {
                            let i1 = i0 + relative1 + end_target.len();
                            result.push(Span::with_root_ctxt(
                                BytePos(i0 as u32 + file.start_pos.0),
                                BytePos(i1 as u32 + file.start_pos.0),
                            ));
                            offset = i1;
                        } else {
                            return Err(RefactoringErrorInternal::int(&format!("Couldn't find string: {}", end_target)));
                        }
                    }
                }
            },
            _ => {}
        };
    }
    Ok(result)
}

/**
 * Used for composite refactorings where one micro-refactoring
 * inserts a special tag to mark one or more elements in the AST.
 */
 pub fn collect_comments_with_id(context: &SourceMapContext, range_id: &str) -> QueryResult<Span> {
    let start_target = create_refactor_tool_marker(range_id, false);
    let end_target = create_refactor_tool_marker(range_id, true);

    let mut indexes = None;

    for file in context.source_map.files().iter() {

        match &file.name {
            rustc_span::FileName::Real(_path) => {
                if let Some(src) = &file.src {
                    if let Some(start0) = src.find(&start_target) {
                        
                        let start1 = start0 + start_target.len();
                        if let Some(end0) = src[start1..].find(&end_target) {
                            indexes = Some((start1 as u32, (end0 + start1) as u32, file.start_pos.0));
                        } else {
                            return Err(RefactoringErrorInternal::int(&format!("Couldn't find string: {}", end_target)));
                        }
                    }
                }
            },
            _ => {}
        };
    }
    if let Some((i0, i1, f0)) = indexes {
        Ok(Span::with_root_ctxt(
            BytePos(i0 + f0),
            BytePos(i1 + f0),
        ))
    } else {
        Err(RefactoringErrorInternal::comment_not_found(&start_target))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{run_ast_query, TestContext};
    use crate::refactoring_invocation::AstContext;

    fn map_with_id(_: TestContext) -> Box<dyn Fn(&AstContext) -> QueryResult<String> + Send> { 
        Box::new(move |ast| {
            let block_span = collect_comments_with_id(&ast.source(), "extract-block.block")?;
            Ok(ast.get_source(block_span))
        })
    }
    fn map_with_id_other(_: TestContext) -> Box<dyn Fn(&AstContext) -> QueryResult<String> + Send> { 
        Box::new(move |ast| {
            let block_span = collect_comments_with_id(&ast.source(), "other")?;
            Ok(ast.get_source(block_span))
        })
    }
    fn map(_: TestContext) -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        Box::new(move |ast| {
            let spans = collect_comments(&ast.source())?;
            Ok(spans.iter().map(|e| ast.get_source(*e)).collect())
        })
    }

    #[test]
    fn should_collect_with_id1() {
        let input = r#"fn foo() {
            /*refactor-tool:extract-block.block:start*/let bar = 0;/*refactor-tool:extract-block.block:end*/
        }"#;
        let expected = Ok("let bar = 0;".to_owned());

        let actual = run_ast_query(input, map_with_id);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_with_id2() {
        let input = r#"fn foo() {
            /*refactor-tool:other:start*/let baz = 0;/*refactor-tool:other:end*/
        }"#;
        let expected = Ok("let baz = 0;".to_owned());

        let actual = run_ast_query(input, map_with_id_other);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_fail_when_start_not_found() {
        let input = r#"fn foo() {
            /*refactor-tool:x:start*/let baz = 0;/*refactor-tool:other:end*/
        }"#;
        let expected = Err(RefactoringErrorInternal::comment_not_found("/*refactor-tool:other:start*/"));

        let actual = run_ast_query(input, map_with_id_other);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_fail_when_end_not_found() {
        let input = r#"fn foo() {
            /*refactor-tool:other:start*/let baz = 0;/*refactor-tool::end*/
        }"#;
        let expected = Err(RefactoringErrorInternal::int("Couldn't find string: /*refactor-tool:other:end*/"));

        let actual = run_ast_query(input, map_with_id_other);

        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_comments() {
        let input = r#"fn foo() {
            /*refactor-tool:extract-block.block:start*/let bar = 0;/*refactor-tool:extract-block.block:end*/
        }"#;
        let expected = Ok(vec![
            "/*refactor-tool:extract-block.block:start*/".to_owned(), 
            "/*refactor-tool:extract-block.block:end*/".to_owned()]);

        let actual = run_ast_query(input, map);

        assert_eq!(actual, expected);
    }
}