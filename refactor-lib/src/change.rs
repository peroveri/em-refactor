#[derive(Debug, Clone)]
pub struct Change {
    pub file_name: String,
    pub file_start_pos: u32,
    pub start: u32,
    pub end: u32,
    pub replacement: String
}