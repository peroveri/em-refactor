use std::collections::HashMap;
use crate::refactorings::visitors::hir::ExpressionUseKind;
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
    pub fn get_return_values(&self) -> Vec<ReturnValue> {
        let mut map: HashMap<String, ReturnValue> = HashMap::new();

        let mut ids = vec![]; // HashMap doesnt preserve order

        for rv in self.return_values.iter() {
            if !ids.contains(&rv.ident) {
                ids.push(rv.ident.to_string());
            }
            if let Some(entry) = map.get_mut(&rv.ident) {
                entry.is_mutated = entry.is_mutated || rv.use_kind.is_mutated();
            } else {
                let e = ReturnValue {
                    ident: rv.ident.clone(),
                    is_mutated: rv.use_kind.is_mutated()
                };
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| map.get(id).unwrap().clone())
            .collect::<Vec<_>>()
    }
    pub fn add_return_value(&mut self, ident: String, use_kind: ExpressionUseKind) {
        self.return_values.push(VariableUse {
            ident,
            use_kind,
        });
    }
}
#[derive(Clone)]
pub struct VariableUse {
    pub use_kind: ExpressionUseKind,
    pub ident: String,
}

#[derive(Clone)]
pub struct ReturnValue {
    pub ident: String,
    pub is_mutated: bool
}