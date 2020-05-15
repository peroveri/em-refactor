use rustc_ast::ast::{Item, ItemKind, StructField};
use rustc_ast::visit::{Visitor, walk_crate, walk_item};
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
    fn visit_item(&mut self, i: &'ast Item) {
        match i.kind {
            ItemKind::Union(..)
            | ItemKind::Enum(..) => {},
            _ => walk_item(self, i)
        }
    }
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
    use crate::test_utils::run_ast_query;
    
    fn map(mode: CollectFieldMode) -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> {
        Box::new(move |ast| {
            Ok(
                collect_box_field_candidates(ast, mode)?
                .iter()
                .map(|span| ast.get_source(*span))
                .collect::<Vec<_>>()
            )
        })
    }
    fn map_all() -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        map(CollectFieldMode::All)
    }
    fn map_named() -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        map(CollectFieldMode::Named)
    }
    fn map_tuple() -> Box<dyn Fn(&AstContext) -> QueryResult<Vec<String>> + Send> { 
        map(CollectFieldMode::Tuple)
    }

    #[test]
    fn should_collect_named() {
        let input = "struct G {field: i32}";
        let expected = Ok(vec!["field".to_string()]);

        let actual = run_ast_query(input, map_named);
        
        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_tuple() {
        let input = "struct H(u32);";
        let expected = Ok(vec!["u32".to_string()]);

        let actual = run_ast_query(input, map_tuple);
        
        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_all() {
        let input = r#"struct H(u32);
struct G {field: i32}"#;
        let expected = Ok(vec!["u32".to_string(), "field".to_string()]);

        let actual = run_ast_query(input, map_all);
        
        assert_eq!(actual, expected);
    }
    #[test]
    fn should_collect_derive() {
        let input = r#"#[derive(Clone)]
struct G {field: i32}"#;
        let expected = Ok(vec!["field".to_string()]);

        let actual = run_ast_query(input, map_all);
        
        assert_eq!(actual, expected);
    }
    #[test]
    fn shouldnt_collect_nonstructs() {
        let input = r#"
enum Enum {
    EnumItem1,
    EnumTup1(u32),
    EnumStruct1{struct1: u32},
}
union Union {
    union_field1: u32,
}
"#;
        let expected = Ok(vec![]);

        let actual = run_ast_query(input, map_all);
        
        assert_eq!(actual, expected);
    }
}
