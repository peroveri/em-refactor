use rustc::ty::TyCtxt;
use rustc_interface::interface;
use rustc_span::Span;
use rustc_hir::print;
use rustc_hir::intravisit::{walk_crate, NestedVisitorMap, Visitor};
use rustc_hir::{Expr, FunctionRetTy, Item, ItemKind, Stmt};
use rustc_hir::intravisit::{walk_expr, walk_item, walk_stmt};
use rustc::hir::map::Map;
use syntax::ast::{Crate};
use crate::refactorings::utils::map_range_to_span;
use crate::refactorings::visitors::collect_inline_macro;
use crate::refactor_definition::SourceCodeRange;

struct RustcAfterAnalysisCallbacks<F>(F, SourceCodeRange);

impl<F> rustc_driver::Callbacks for RustcAfterAnalysisCallbacks<F>
where
    F: Fn(TyCtxt<'_>) -> (),
    F: Send,
{
    fn after_parsing<'tcx>(
        &mut self,
        _compiler: &interface::Compiler,
        _queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        rustc_driver::Compilation::Continue
    }
    
    fn after_expansion<'tcx>(
        &mut self,
        compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {

        let span = map_range_to_span(compiler.session().source_map(), &self.1);
        if span.is_err() {
            return rustc_driver::Compilation::Stop;
        }
        let span = span.unwrap();

        let (crate_, _, _) = 
        &*queries
            .expansion()
            .unwrap()
            .peek_mut();

        if get_macro_body(span, crate_) {
            rustc_driver::Compilation::Stop
        } else {
            rustc_driver::Compilation::Continue
        }
    }
    fn after_analysis<'tcx>(
        &mut self, 
        _compiler: &interface::Compiler,
        queries: &'tcx rustc_interface::Queries<'tcx>
    ) -> rustc_driver::Compilation {
        // compiler.session().abort_if_errors();
        queries
            .global_ctxt()
            .unwrap()
            .peek_mut()
            .enter(|tcx| self.0(tcx));
        rustc_driver::Compilation::Stop
    }
}

struct IdentCollector<'v> {
    tcx: TyCtxt<'v>,
    span: Span,
    res_type: Option<String>
}

impl<'v> Visitor<'v> for IdentCollector<'v> {
    type Map = Map<'v>;
    fn nested_visit_map<'this>(&'this mut self) -> NestedVisitorMap<'this, Self::Map> {
        NestedVisitorMap::All(&self.tcx.hir())
    }
    fn visit_item(&mut self, i: &'v Item) {
        if !i.span.contains(self.span) {
            return;
        }

        if let ItemKind::Fn(sig, _, _) = &i.kind {
            let decl = &sig.decl;
            let a = decl.inputs.iter().map(|t| 
                print::to_string(print::NO_ANN, |s| s.print_type(t))).collect::<Vec<_>>();
            let inputs = a.join(",");
            let output = if let FunctionRetTy::Return(t) = &decl.output {
                print::to_string(print::NO_ANN, |s| s.print_type(t))
            } else {
                "".to_owned()
            };
            let ident = i.ident.to_string();


            self.res_type = Some(format!("fn {}({}) -> ({})", ident, inputs, output));
        }
        
        walk_item(self, i);
    }
    fn visit_expr(&mut self, expr: &'v Expr) {
        if !expr.span.contains(self.span) {
            return;
        }

        self.res_type = Some(print::to_string(print::NO_ANN, |s| s.print_expr(expr)));

        walk_expr(self, expr);
    }
    fn visit_stmt(&mut self, stmt: &'v Stmt) {
        if !stmt.span.contains(self.span) {
            return;
        }

        self.res_type = Some(print::to_string(print::NO_ANN, |s| s.print_stmt(stmt)));

        walk_stmt(self, stmt);
    }
}


pub fn provide_type(rustc_args: &[String], file_name: &str, selection: &str) -> Result<(), String> {

    let mut s = selection.split(':');
    let (a, b) = (s.nth(0).unwrap().parse().unwrap(), s.nth(0).unwrap().parse().unwrap());

    let range = SourceCodeRange {
        file_name: file_name.to_string(),
        from: a,
        to: b
    };
    
    let mut callbacks = RustcAfterAnalysisCallbacks(|tcx: TyCtxt<'_>| {
        let range = map_range_to_span(&tcx.sess.source_map(), &range);

        if range.is_err() {
            return;
        }
        let mut v = IdentCollector {
            tcx,
            span: range.unwrap(),
            res_type: None
        };
    
        walk_crate(&mut v, tcx.hir().krate());

        if let Some(res_type) = v.res_type {
            print!("{}", serde_json::json!([{
                "type": res_type
            }]));
        } else {
            print!("[]");
        }
    }, range.clone());

    let emitter = Box::new(Vec::new());
    std::env::set_var("RUST_BACKTRACE", "1");
    let err = rustc_driver::run_compiler(&rustc_args, &mut callbacks, None, Some(emitter));
    err.unwrap();
    Ok(())
}

fn get_macro_body(span: Span, crate_: &Crate) -> bool {
    if let Some((content, _)) = collect_inline_macro(span, crate_) {
        print!("{}", serde_json::json!([{
            "type": content
        }]));
        true
    } else {
        false
    }
}
