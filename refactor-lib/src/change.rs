use serde::{Serialize, Deserialize};

/// 
/// Represents a file change applied by the refactorings
/// 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub end: u32,
    pub file_name: String,
    pub file_start_pos: u32,
    /// Indexed relative to this file
    /// Indexed relative to this file
    pub replacement: String,
    pub start: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReplaceContent {
    pub byte_end: u32,
    pub byte_start: u32,
    pub char_end: usize,
    pub char_start: usize,
    pub file_name: String,
    pub line_end: usize,
    pub line_start: usize,
    pub replacement: String
}