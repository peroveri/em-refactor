use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub projects: Vec<Project>
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub git_repo: String,
    pub git_hash: Option<String>,
    pub subdir: Option<String>,
    pub skip: Option<bool>
}