use std::fs::File;
use std::io::prelude::*;

pub use experiments_struct::*;
pub use test_result::*;

mod experiments_struct;
mod test_result;

pub fn read_settings() -> std::io::Result<Config> {
    let mut file = File::open("./tests/exp/projects.json")?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    let r = serde_json::from_str::<Config>(&file_content)?;

    Ok(r)
}