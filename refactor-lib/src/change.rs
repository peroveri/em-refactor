#[derive(Debug)]
pub struct Change {
    pub file_name: String,
    pub start: u32,
    pub end: u32,
    pub replacement: String
}