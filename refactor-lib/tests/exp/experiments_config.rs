use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};

pub fn read_settings() -> std::io::Result<Config> {
    let mut file = File::open("./tests/exp/projects.json")?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    let r = serde_json::from_str::<Config>(&file_content)?;

    Ok(r)
}
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