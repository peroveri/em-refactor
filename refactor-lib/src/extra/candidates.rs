use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateOutput {
    pub candidates: Vec<CandidatePosition>,
    pub refactoring: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePosition {
    pub file: String,
    pub from: u32,
    pub to: u32
}