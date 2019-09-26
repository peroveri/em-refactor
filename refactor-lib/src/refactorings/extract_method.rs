use crate::change::Change;
use crate::refactor_args::RefactorArgs;
use crate::refactorings::expr_use_visit::CollectVarsArgs;
use crate::refactorings::stmts_visitor::StmtsVisitor;
use rustc::hir;
use rustc::ty;
use syntax::source_map::{BytePos, Span};

fn get_selection(s: &str) -> (u32, u32) {
    let vs = s.split(':').collect::<Vec<_>>();
    (vs[0].parse().unwrap(), vs[1].parse().unwrap())
}

/**
 * Translate (file name, local offset) to (global offset) in the source map
 * TODO: do this earlier?
 */
fn get_selection_with_global_offset(
    source_map: &syntax::source_map::SourceMap,
    selection: (u32, u32),
    file: &str,
) -> (u32, u32) {
    let filename = syntax::source_map::FileName::Real(std::path::PathBuf::from(file));
    let source_file = source_map.get_source_file(&filename).unwrap();
    (
        selection.0 + source_file.start_pos.0,
        selection.1 + source_file.start_pos.0,
    )
}

fn get_stmts_source(source_map: &syntax::source_map::SourceMap, stmts: &[&hir::Stmt]) -> String {
    stmts
        .iter()
        .map(|stmt| source_map.span_to_snippet(stmt.span).unwrap().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn map_to_span(
    source_map: &syntax::source_map::SourceMap,
    selection: (u32, u32),
    file: &str,
) -> Span {
    let filename = syntax::source_map::FileName::Real(std::path::PathBuf::from(file));
    let source_file = source_map.get_source_file(&filename).unwrap();
    Span::with_root_ctxt(
        BytePos(selection.0 + source_file.start_pos.0),
        BytePos(selection.1 + source_file.start_pos.0),
    )
}

/**
 * WIP
 *
 * Extract method
 *
 * Input:
 * - f - A function
 * - m - The module containing f
 * - s - A selection in f (of consecutive statements?)
 *
 * Assumptions:
 * - f is not a method
 *
 * Steps:
 * g <- new function with fresh name
 * add g to m
 * vs <- all variables in s not declared in s
 * add vs as parameters of g
 * replace s with a call to g with arguments vs
 */

/**
 * Extract method
 * 1.  Collect S0, Si and Sj
 * 2.  Va <- Vars declared in S0, used in Si
 * 3.  Vb <- Vars declared in Si, used in Sj
 * 4.  if |Vb| > 1 fail
 * 5.  g <- new method with fresh name
 * 6.  add g to m
 * 7.  for each v in Va
 * 8.    add v as parameter of g
 * 9.  move body of f to g
 * 10. add method call to g at Si
 * 11. if |Vb| == 1
 * 12.   add as return type of g
 * 13.   declare before Si and asign to g()
 */
pub fn do_refactoring(ty: ty::TyCtxt, args: &RefactorArgs) -> Vec<Change> {
    let selection = get_selection_with_global_offset(
        ty.sess.source_map(),
        get_selection(&args.selection),
        &args.file,
    );
    let spi = map_to_span(
        ty.sess.source_map(),
        get_selection(&args.selection),
        &args.file,
    );

    let stmts_visit = StmtsVisitor::visit(ty, &args.file, selection);

    if let Some(stmts) = stmts_visit.stmts {
        let collect_args = CollectVarsArgs {
            body_id: stmts.fn_body_id,
            spi,
        };
        let x = crate::refactorings::expr_use_visit::collect_vars(ty, collect_args);

        // let Va = get_decl_and_used(ty, &stmts.S0, &stmts.Si);
        // let Vb = get_decl_and_used(ty, &stmts.Si, &stmts.Sj);

        // println!("S0: {:?}, Si: {:?}, Sj: {:?}", stmts.S0, stmts.Si, stmts.Sj);
        // println!("Va: {}, Vb: {}", Va.len(), Vb.len());
        if x.return_values.len() > 1 {
            return vec![]; // should be error
        }

        let params = x
            .arguments
            .iter()
            .map(|(name, ty)| format!("{}: {:?}", name, ty))
            .collect::<Vec<_>>()
            .join(", ");

        // let params = ExtractMethodRefactoring::convert_to_params(&ty, &Va);
        // println!("params: {}", params);
        let new_fn = format!(
            "fn {}({}) {{\n{}\n}}",
            args.new_function,
            params,
            get_stmts_source(ty.sess.source_map(), &stmts.Si)
        );

        vec![Change {
            file_name: args.file.to_string(),
            start: 0,
            end: 0,
            replacement: new_fn,
        }]
    // println!("{}", new_fn);
    } else {
        println!("no statements");
        vec![]
    }
}
