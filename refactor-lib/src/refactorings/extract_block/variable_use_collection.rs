use std::collections::HashMap;
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
    pub fn get_return_values(&self) -> Vec<VariableUse> {
        let mut map: HashMap<String, VariableUse> = HashMap::new();

        let mut ids = vec![]; // HashMap doesnt preserve order

        for rv in self.return_values.iter() {
            if !ids.contains(&rv.ident) {
                ids.push(rv.ident.to_string());
            }
            if let Some(entry) = map.get_mut(&rv.ident) {
                entry.is_mutated = entry.is_mutated || rv.is_mutated;
            } else {
                let e = rv.clone();
                map.insert(rv.ident.clone(), e);
            }
        }

        ids.iter()
            .map(|id| map.get(id).unwrap().clone())
            .collect::<Vec<_>>()
    }
    #[cfg(test)]
    pub fn return_values(&self) -> &Vec<VariableUse> {
        &self.return_values
    }
    pub fn add_return_value(&mut self, ident: String, is_mutated: bool) {
        self.return_values.push(VariableUse {
            ident,
            is_mutated,
        });
    }
}
#[derive(Clone)]
pub struct VariableUse {
    pub is_mutated: bool,
    pub ident: String,
}
