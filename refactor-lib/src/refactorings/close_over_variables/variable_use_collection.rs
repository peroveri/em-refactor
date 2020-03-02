use std::collections::HashMap;
use rustc_span::Span;
// find a name
pub struct VariableUseCollection {
    /**
     * Variables declared in 'span', used after 'span'
     */
    return_values: Vec<VariableUse>,
}
impl VariableUseCollection {
    pub fn new() -> Self {
        VariableUseCollection {
            return_values: vec![],
        }
    }
    pub fn add_return_value(&mut self, ident: String, is_borrow: bool, is_mutated: bool, span: Span) {
        self.return_values.push(VariableUse {
            ident,
            is_borrow,
            is_mutated,
            span
        });
    }

    pub fn get_params(&self) -> Vec<VariableUse> {
        let mut map: HashMap<String, VariableUse> = HashMap::new();

        let mut ids = vec![]; // HashMap doesnt preserve order

        for rv in self.return_values.iter() {
            if !ids.contains(&rv.ident) {
                ids.push(rv.ident.to_string());
            }
            if let Some(entry) = map.get_mut(&rv.ident) {
                entry.is_mutated = entry.is_mutated || rv.is_mutated;
                entry.is_borrow = entry.is_borrow || rv.is_borrow;
            } else {
                let e = rv.clone();
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| {
                let mut v = map.get(id).unwrap().clone();
                if v.is_borrow && !v.is_mutated {
                    v.is_borrow = false;
                }
                v
            })
            .collect::<Vec<_>>()
    }
    pub fn get_args(&self) -> Vec<VariableUse> {
        let mut map: HashMap<String, VariableUse> = HashMap::new();

        let mut ids = vec![]; // HashMap doesnt preserve order

        for rv in self.return_values.iter() {
            if !ids.contains(&rv.ident) {
                ids.push(rv.ident.to_string());
            }
            if let Some(entry) = map.get_mut(&rv.ident) {
                entry.is_mutated = entry.is_mutated || rv.is_mutated;
                entry.is_borrow = entry.is_borrow || rv.is_borrow;
            } else {
                let e = rv.clone();
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| map.get(id).unwrap().clone())
            .collect::<Vec<_>>()
    }
    pub fn get_borrows(&self) -> Vec<Span> {
        self.return_values.iter().filter(|rv| rv.is_borrow).map(|rv| rv.span).collect::<Vec<_>>()
    }
}
#[derive(Clone)]
pub struct VariableUse {
    pub is_mutated: bool,
    pub is_borrow: bool,
    pub ident: String,
    pub span: Span
}
impl VariableUse {
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