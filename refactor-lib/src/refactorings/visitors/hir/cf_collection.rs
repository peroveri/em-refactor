use rustc::ty::TyCtxt;
use rustc_span::Span;
use crate::refactorings::utils::get_source;

#[derive(Clone, Debug, PartialEq)]
pub enum CfType {
    Break = 1,
    Continue = 2,
    Return = 3,
    Nothing = 0
}
impl CfType {
    fn get_keyword(&self) -> &str {
        match self {
            CfType::Break => "break",
            CfType::Continue => "continue",
            CfType::Nothing => "{}",
            CfType::Return => "return",
        }
    }
}

#[derive(Clone)]
pub struct ControlFlowExpr {
    pub cf_type: CfType,
    pub cf_expr_span: Span,
    pub sub_expr_span: Option<Span>
}


pub struct ControlFlowExprCollection {
    pub items: Vec<ControlFlowExpr>
}

impl ControlFlowExprCollection {

    // (, .0, ..) => .2
    // (, .1, ..) ..
    fn get_self_arm<'a>(&self, cf_type: CfType) -> (String, String, String, String) {
        if let Some(c) = self.items.iter().find(|c| c.cf_type == cf_type) {
            if cf_type == CfType::Nothing {
                (", a".to_owned(), ", _".to_owned(), "a.unwrap()".to_owned(), ", None".to_owned())
            } else if let Some(_) = c.sub_expr_span {
                (", a".to_owned(), ", _".to_owned(), format!("{} a.unwrap(),", cf_type.get_keyword()), ", None".to_owned())
            } else {
                ("".to_owned(), "".to_owned(), format!("{},", cf_type.get_keyword()), "".to_owned())
            }
        } else {
            ("".to_owned(), "".to_owned(), "".to_owned(), "".to_owned())
        }
    }

    pub fn get_cf_arms(&self) -> String {
        let mut break_arm = "".to_owned();
        let mut return_arm = "".to_owned();
        let mut expr_arm = "".to_owned();
        let mut cont_arm = "".to_owned();
        let break_part = self.get_self_arm(CfType::Break);
        let return_part = self.get_self_arm(CfType::Return);
        let expr_part = self.get_self_arm(CfType::Nothing);
        
        if let Some(_) = self.get_cf_break() {
            break_arm = format!("\n({}{}{}{}) => {}", CfType::Break as i32, break_part.0, return_part.1, expr_part.1, break_part.2);
        }
        if let Some(_) = self.get_cf_cont() {
            cont_arm = format!("\n({}{}{}{}) => continue,", CfType::Continue as i32, break_part.1, return_part.1, expr_part.1);
        }
        if let Some(_) = self.get_cf_ret() {
            return_arm = format!("\n({}{}{}{}) => {}", CfType::Return as i32, break_part.1, return_part.0, expr_part.1, return_part.2);
        }
        if let Some(_) = self.get_cf_expr() {
            expr_arm = format!("\n(_{}{}{}) => {}", break_part.1, return_part.1, expr_part.0, expr_part.2);
        }

        format!("{}{}{}{}", break_arm, cont_arm, return_arm, expr_arm)
    }

    pub fn has_cfs(&self) -> bool {
        for cf in &self.items {
            if cf.is_cf() {
                return true;
            }
        }
        false
    }

    pub fn replace_cfs(&self, tcx: TyCtxt, mut source: String, block_start: u32) -> String {
        let mut cfs = self.items.to_vec();
        cfs.sort_by_key(|c| c.cf_expr_span);
        cfs.reverse();

        let break_part = self.get_self_arm(CfType::Break);
        let return_part = self.get_self_arm(CfType::Return);
        let expr_part = self.get_self_arm(CfType::Nothing);

        for cf in cfs {
            match cf.cf_type {
                CfType::Break => {
                    // check macros inv!
                    let start = (cf.cf_expr_span.lo().0 - block_start) as usize;
                    let end = (cf.cf_expr_span.hi().0 - block_start) as usize;
                    let expr = 
                    if let Some(e) = cf.sub_expr_span {
                        format!(", Some({})", get_source(tcx, e))
                    } else {
                        "".to_owned()
                    };
                    source.replace_range(start..end, &format!("return ({}{}{}{})", CfType::Break as i32, expr, return_part.3, expr_part.3));
                },
                CfType::Continue => {
                    let start = (cf.cf_expr_span.lo().0 - block_start) as usize;
                    let end = (cf.cf_expr_span.hi().0 - block_start) as usize;
                    source.replace_range(start..end, &format!("return ({}{}{}{})", CfType::Continue as i32, break_part.3, return_part.3, expr_part.3));
                },
                CfType::Nothing => {
                    let start = (cf.cf_expr_span.lo().0 - block_start) as usize;
                    let end = (cf.cf_expr_span.hi().0 - block_start) as usize;

                    let expr = format!(", Some({})", get_source(tcx, cf.cf_expr_span));
                    // if let Some(e) = cf.sub_expr_span {
                    //     format!(", Some({})", get_source(tcx, e))
                    // } else {
                    //     "".to_owned()
                    // };
                    source.replace_range(start..end, &format!("({}{}{}{})", CfType::Nothing as i32, break_part.3, return_part.3, expr));
                },
                CfType::Return => {
                    let start = (cf.cf_expr_span.lo().0 - block_start) as usize;
                    let end = (cf.cf_expr_span.hi().0 - block_start) as usize;
                    let expr = 
                    if let Some(e) = cf.sub_expr_span {
                        format!(", Some({})", get_source(tcx, e))
                    } else {
                        "".to_owned()
                    };
                    source.replace_range(start..end, &format!("return ({}{}{}{})", CfType::Return as i32, break_part.3, expr, expr_part.3));
                },
            }
        }
        source
    }

    // fn get_ret_opt 

    pub fn get_cf_ret(&self) -> Option<&ControlFlowExpr> {
        self.items.iter().find(|c| c.cf_type == CfType::Return)
    }
    pub fn get_cf_cont(&self) -> Option<&ControlFlowExpr> {
        self.items.iter().find(|c| c.cf_type == CfType::Continue)
    }
    pub fn get_cf_expr(&self) -> Option<&ControlFlowExpr> {
        self.items.iter().find(|c| c.cf_type == CfType::Nothing)
    }
    pub fn get_cf_break(&self) -> Option<&ControlFlowExpr> {
        self.items.iter().find(|c| c.cf_type == CfType::Break)
    }


}

impl ControlFlowExpr {
    pub fn is_cf(&self) -> bool {
        match self.cf_type {
            CfType::Nothing => false,
            _ => true
        }
    }
    pub fn new(cf_type: CfType, cf_expr_span: Span, sub_expr_span: Option<Span>) -> Self {
        Self {
            cf_type,
            cf_expr_span,
            sub_expr_span
        }
    }
    pub fn cont(cf_expr_span: Span) -> Self {
        Self::new(
            CfType::Continue,
            cf_expr_span,
            None
        )
    }
    pub fn brk(cf_expr_span: Span, sub_expr_span: Option<Span>) -> Self {
        Self::new(
            CfType::Break,
            cf_expr_span,
            sub_expr_span
        )
    }
    pub fn ret(cf_expr_span: Span, sub_expr_span: Option<Span>) -> Self {
        Self::new(
            CfType::Return,
            cf_expr_span,
            sub_expr_span
        )
    }
    pub fn expr(cf_expr_span: Span) -> Self {
        Self::new(
            CfType::Nothing,
            cf_expr_span,
            None
        )
    }
}
