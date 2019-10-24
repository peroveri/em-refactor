use serde::{Serialize, Deserialize};

/// 
/// Represents a file change applied by the refactorings
/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub file_name: String,
    pub file_start_pos: u32,
    /// Indexed relative to this file
    pub start: u32,
    /// Indexed relative to this file
    pub end: u32,
    pub replacement: String
}