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
    pub is_mut: bool
}
impl Param {
    pub fn new(ident: &str, is_mut: bool, is_borrow: bool) -> Self {
        Self {
            ident: ident.to_string(),
            is_borrow,
            is_mut
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
    pub fn to_cmp(&self) -> Vec<(ExpressionUseKind, String, (u32, u32))> {
        self.return_values.iter()
            .map(|r| (r.bk, r.ident.to_string(), (r.span.lo().0, r.span.hi().0)))
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
    pub fn add_return_value(&mut self, ident: String, bk: ExpressionUseKind, span: Span) {
        self.return_values.push(VariableUse {
            ident,
            bk,
            span
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
            } else {
                let e = Param::new(&rv.ident, rv.is_mutated(), rv.is_borrow());
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| {
                let mut v = map.get(id).unwrap().clone();
                if v.is_borrow && !v.is_mut {
                    v.is_borrow = false;
                }
                v
            })
            .collect::<Vec<_>>()
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
            } else {
                let e = Param::new(&rv.ident, rv.is_mutated(), rv.is_borrow());
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| map.get(id).unwrap().clone())
            .collect::<Vec<_>>()
    }
    pub fn get_borrows(&self) -> Vec<Span> {
        self.return_values.iter().filter(|rv| rv.is_borrow()).map(|rv| rv.span).collect::<Vec<_>>()
    }
}
#[derive(Clone)]
pub struct VariableUse {
    pub bk: ExpressionUseKind,
    pub ident: String,
    pub span: Span
}
impl VariableUse {
    pub fn is_borrow(&self) -> bool {
        self.bk.is_borrow()
    }
    pub fn is_mutated(&self) -> bool {
        self.bk.is_mutated()
    }
}
impl Param {
    pub fn as_arg(&self) -> String {
        format!("{}{}", if self.is_borrow {
            "&"
        } else {
            ""
        }, self.ident)
    }
    pub fn as_param(&self) -> String {
        format!("{}: {}_", self.ident,
        if self.is_borrow {
            "&mut "
        } else {
            ""
        })
    }
}