use rustc_span::Span;

#[derive(Clone, Debug, PartialEq)]
pub enum CfType {
    Break = 1,
    Continue = 2,
    Return = 3,
    Nothing = 0
}
#[derive(Clone)]
pub struct ControlFlowExpr {
    pub cf_type: CfType,
    pub cf_key_span: Span,
    pub cf_expr_span: Span,
    pub sub_expr_span: Option<Span>,
    pub sub_expr_type: Option<String>
}

pub struct ControlFlowExprCollection {
    pub items: Vec<ControlFlowExpr>
}
fn get_enum_name() -> String {
    "ReturnFoo".to_owned() // TODO: add a random number to the name?
}

impl ControlFlowExprCollection {

    pub fn get_cf_arms(&self) -> String {
        let enum_name = get_enum_name();
        let mut arms = vec![];
        
        if let Some(e) = self.get_cf_break() {
            let (sub1, sub2) = 
                if e.sub_expr_span.is_some() {("e".to_owned(), " e".to_owned())}
                else {("".to_owned(), "".to_owned())};
            arms.push(format!("\n{}::Break({}) => break{}", enum_name, sub1, sub2));
        }
        if let Some(_) = self.get_cf_cont() {
            arms.push(format!("\n{}::Continue() => continue", enum_name));
        }
        if let Some(_) = self.get_cf_expr() {
            arms.push(format!("\n{}::Expr(e) => e", enum_name));
        }
        if let Some(_) = self.get_cf_ret() {
            arms.push(format!("\n{}::Return(e) => return e", enum_name));
        }

        arms.join(",")
    }

    pub fn has_cfs(&self) -> bool {
        for cf in &self.items {
            if cf.is_cf() {
                return true;
            }
        }
        false
    }

    pub fn replace_cfs(&self) -> Vec<(Span, String)> {
        let mut replacements = vec![];
        let cfs = self.items.to_vec();

        let enum_name = get_enum_name();
        for cf in cfs {
            match cf.cf_type {
                CfType::Break => {
                    // check macros inv!

                    replacements.push((cf.cf_key_span, format!("return {}::Break(", enum_name)));
                    replacements.push((cf.cf_expr_span.shrink_to_hi(), ")".to_owned()));
                },
                CfType::Continue => {
                    replacements.push((cf.cf_key_span, format!("return {}::Continue()", enum_name)));
                },
                CfType::Nothing => {

                    if cf.cf_expr_span.lo() == cf.cf_expr_span.hi() {
                        replacements.push((cf.cf_expr_span, format!("{}::Expr(())", enum_name)));
                    } else {
                        replacements.push((cf.cf_expr_span.shrink_to_lo(), format!("{}::Expr(", enum_name)));
                        replacements.push((cf.cf_expr_span.shrink_to_hi(), ")".to_owned()));
                    }
                },
                CfType::Return => {
                    replacements.push((cf.cf_key_span, format!("return {}::Return(", enum_name)));
                    replacements.push((cf.cf_expr_span.shrink_to_hi(), ")".to_owned()));
                },
            }
        }
        replacements
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

    pub fn get_enum_decl(&self) -> String {
        let enum_name = get_enum_name();

        let mut parts = vec![];
        if let Some(e) = self.get_cf_break() {
            parts.push(format!("Break({})", e.sub_expr_type.clone().unwrap_or_default()));
        }
        if let Some(_) = self.get_cf_cont() {
            parts.push(format!("Continue()"));
        }
        if let Some(e) = self.get_cf_expr() {
            parts.push(format!("Expr({})", e.sub_expr_type.clone().unwrap_or_default()));
        }
        if let Some(e) = self.get_cf_ret() {
            parts.push(format!("Return({})", e.sub_expr_type.clone().unwrap_or_default()));
        }

        format!(r#"
enum {} {{
{}
}}"#, enum_name, parts.join(",\n"))
    }

}

impl ControlFlowExpr {
    pub fn is_cf(&self) -> bool {
        match self.cf_type {
            CfType::Nothing => false,
            _ => true
        }
    }
    pub fn new(cf_type: CfType, cf_expr_span: Span, cf_key_span: Span, sub_expr_span: Option<Span>, sub_expr_type: Option<String>) -> Self {
        Self {
            cf_type,
            cf_expr_span,
            cf_key_span,
            sub_expr_span,
            sub_expr_type
        }
    }
    pub fn cont(cf_expr_span: Span) -> Self {
        Self::new(
            CfType::Continue,
            cf_expr_span,
            cf_expr_span,
            None,
            None
        )
    }
    pub fn brk(cf_expr_span: Span, cf_key_span: Span, sub_expr_span: Option<Span>, sub_expr_type: Option<String>) -> Self {
        Self::new(
            CfType::Break,
            cf_expr_span,
            cf_key_span,
            sub_expr_span,
            sub_expr_type
        )
    }
    pub fn ret(cf_expr_span: Span, cf_key_span: Span, sub_expr_span: Option<Span>, sub_expr_type: Option<String>) -> Self {
        Self::new(
            CfType::Return,
            cf_expr_span,
            cf_key_span,
            sub_expr_span,
            sub_expr_type
        )
    }
    pub fn expr(cf_expr_span: Span, sub_expr_type: String) -> Self {
        Self::new(
            CfType::Nothing,
            cf_expr_span,
            cf_expr_span,
            None,
            Some(sub_expr_type)
        )
    }
}
