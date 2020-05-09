use rustc_hir::BodyId;
use rustc_span::Span;
use crate::refactoring_invocation::{QueryResult, TyContext};
use super::{expr_use_visit::{collect_vars, TypeKind}, local_use_collector::collect_local_uses};
use itertools::Itertools;

pub struct NewClosure {
    pub params: String,
    pub args: String,
    /// Used to introduce deref when we introduce & or &mut
    pub uses: Vec<Span>,
    /// Used to rewrite occurences of 'self' with self_
    /// Assuming that there isnt already a variable called self_ in scope
    pub selfs: Vec<Span>
}

pub fn collect_vars3<'v>(tcx: &'v TyContext, body_id: BodyId) -> QueryResult<NewClosure> {

    let vars = collect_vars(tcx.0, body_id)?;

    let mut params = vec![];
    let mut args = vec![];

    let mut hir_ids = vec![];
    let mut self_hir_ids = vec![];

    for (k, val) in &vars.iter()
        .sorted_by_key(|(_, id, ..)| id.to_string())
        .group_by(|(_, id, ..)| id.to_string()) {

        let val = val.collect::<Vec<_>>();
        let ty = &val[0].3;
        let bks = val.iter().map(|(_, _, bk, ..)| *bk).collect::<Vec<_>>();

        let is_moved = bks.iter().any(|bk| bk.is_moved());
        let is_borrowded = bks.iter().any(|bk| bk.is_borrow());
        let is_mutated = bks.iter().any(|bk| bk.is_mutated());
        let is_type_normal = val.iter().any(|(.., a)| *a == TypeKind::None);

        let modif_arg = if is_moved || !is_type_normal {
            ""
        } else if is_mutated {
            "&mut "
        } else if is_borrowded {
            "&"
        } else {
            ""
        };
        let modif_param = if is_moved || !is_type_normal {
            ""
        } else if is_mutated {
            "&mut "
        } else if is_borrowded {
            "&"
        } else {
            ""
        };

        if (is_borrowded || is_mutated) && is_type_normal {
            hir_ids.push(val[0].0);
        }
        if k == "self" {
            self_hir_ids.push(val[0].0);
        }

        let param_ident = if k == "self" {
            "self_".to_string()
        } else {
            k.to_string()
        };

        params.push(format!("{}: {}{}", param_ident, modif_param, ty));
        args.push(format!("{}{}", modif_arg, k));
    }

    let uses = collect_local_uses(tcx, hir_ids, body_id)?;
    let selfs = collect_local_uses(tcx, self_hir_ids, body_id)?;

    Ok(NewClosure{
        params: params.join(", "),
        args: args.join(", "),
        uses,
        selfs
    })
}