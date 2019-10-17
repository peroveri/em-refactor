use crate::change::Change;

pub fn serialize_changes(changes: Vec<Change>) -> Result<String, i32> {
    if let Ok(serialized) = serde_json::to_string(&changes) {
        Ok(serialized)
    } else {
        Err(4)
    }
}
