use std::collections::HashMap;
use rustc_span::Span;
use super::ExpressionUseKind;

// // Should keep track of the use of a variable that is declared outside the closure, but used inside.
// pub struct SingleVariableUse {
//     ident: String,
//     uses: Vec<Uses>
// }

// impl SingleVariableUse {
//     pub fn add_use(&mut self, span: Span, is_borrow: bool, is_mutated: bool) {
//         self.uses.push(Uses {
//             span,
//             is_borrow,
//             is_mutated
//         });
//     }

//     pub fn is_used_later(&self) -> bool {
//         false
//     }
// }
// pub struct Uses {
//     span: Span,
//     is_borrow: bool,
//     is_mutated: bool
// }

#[derive(PartialEq, Debug, Clone)]
pub struct Param {
    pub ident: String,
    pub is_borrow: bool,
    pub is_mut: bool,
    pub is_move: bool,
    pub ty: String
}
impl Param {
    pub fn new(ident: &str, is_mut: bool, is_borrow: bool, is_move: bool, ty: String) -> Self {
        Self {
            ident: ident.to_string(),
            is_borrow,
            is_mut,
            is_move,
            ty
        }
    }
}
// change to keep a dictionary?
// find a name
pub struct VariableUseCollection {
    /**
     * Variables declared in 'span', used after 'span'
     */
    return_values: Vec<VariableUse>,
}
impl VariableUseCollection {
    pub fn new() -> Self {
        Self {
            return_values: vec![],
        }
    }
    #[cfg(test)]
    pub fn to_cmp(&self) -> Vec<(ExpressionUseKind, String, (u32, u32), String)> {
        self.return_values.iter()
            .map(|r| (r.bk, r.ident.to_string(), (r.span.lo().0, r.span.hi().0), r.ty.to_string()))
            .collect::<Vec<_>>()
    }
    // pub fn get_use_by_ident(&self) -> Vec<SingleVariableUse> {
    //     let mut ret: Vec<SingleVariableUse> = vec![];
    //     for v in &self.return_values {

    //         let mut found = false;
    //         for usage in ret.iter_mut() {
    //             if usage.ident == v.ident {
    //                 usage.add_use(v.span, v.is_borrow, v.is_mutated);
    //                 found = true;
    //             }
    //         }
    //         if !found {
    //             let mut use1 = SingleVariableUse {
    //                 ident: v.ident.to_string(),
    //                 uses: vec![]
    //             };
    //             use1.add_use(v.span, v.is_borrow, v.is_mutated);
    //         }
    //     }
    //     ret
    // }
    pub fn add_return_value(&mut self, ident: String, bk: ExpressionUseKind, span: Span, ty: String) {
        self.return_values.push(VariableUse {
            ident,
            bk,
            span,
            ty
        });
    }

    pub fn get_params(&self) -> Vec<Param> {
        let mut map: HashMap<String, Param> = HashMap::new();

        let mut ids = vec![]; // HashMap doesnt preserve order

        for rv in self.return_values.iter() {
            if !ids.contains(&rv.ident) {
                ids.push(rv.ident.to_string());
            }
            if let Some(entry) = map.get_mut(&rv.ident) {
                entry.is_mut = entry.is_mut || rv.is_mutated();
                entry.is_borrow = entry.is_borrow || rv.is_borrow();
                entry.is_move = entry.is_move || rv.is_move();
            } else {
                let e = Param::new(&rv.ident, rv.is_mutated(), rv.is_borrow(), rv.is_move(), rv.ty.to_string());
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| {
                map.get(id).unwrap().clone()
                // if v.is_borrow && !v.is_mut {
                //     v.is_borrow = false;
                // }
                
            })
            .collect::<Vec<_>>()
    }
    pub fn get_params_formatted(&self) -> String {
        self.get_params().iter().map(|p| p.as_param()).collect::<Vec<_>>().join(", ")
    }
    pub fn get_args(&self) -> Vec<Param> {
        let mut map: HashMap<String, Param> = HashMap::new();

        let mut ids = vec![]; // HashMap doesnt preserve order

        for rv in self.return_values.iter() {
            if !ids.contains(&rv.ident) {
                ids.push(rv.ident.to_string());
            }
            if let Some(entry) = map.get_mut(&rv.ident) {
                entry.is_mut = entry.is_mut || rv.is_mutated();
                entry.is_borrow = entry.is_borrow || rv.is_borrow();
                entry.is_move = entry.is_move || rv.is_move();
            } else {
                let e = Param::new(&rv.ident, rv.is_mutated(), rv.is_borrow(), rv.is_move(), rv.ty.to_string());
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| map.get(id).unwrap().clone())
            .collect::<Vec<_>>()
    }
    pub fn get_args_formatted(&self) -> String {
        self.get_args().iter().map(|p| p.as_arg()).collect::<Vec<_>>().join(", ")
    }
    pub fn get_borrows(&self) -> Vec<Span> {
        if self.return_values.iter().any(|rv| rv.is_move()) {
            vec![]
        } else {
            self.return_values.iter().filter(|rv| rv.is_borrow() || rv.is_mutated()).map(|rv| rv.span).collect::<Vec<_>>()
        }
    }
}
#[derive(Clone)]
pub struct VariableUse {
    pub bk: ExpressionUseKind,
    pub ident: String,
    pub span: Span,
    pub ty: String
}
impl VariableUse {
    pub fn is_borrow(&self) -> bool {
        self.bk.is_borrow()
    }
    pub fn is_mutated(&self) -> bool {
        self.bk.is_mutated()
    }
    pub fn is_move(&self) -> bool {
        self.bk.is_moved()
    }
}
impl Param {
    pub fn as_arg(&self) -> String {
        format!("{}{}", 
        if self.is_move {
            ""
        } else if self.is_mut {
            "&mut "
        } else if self.is_borrow {
            "&"
        } else {
            ""
        }, self.ident)
    }
    pub fn as_param(&self) -> String {
        format!("{}: {}{}", self.ident,
        if self.is_move {
            ""
        } else if self.is_mut {
            "&mut "
        } else if self.is_borrow {
            "&"
        } else {
            ""
        }, self.ty)
    }
}