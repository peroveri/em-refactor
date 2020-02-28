use serde::{Serialize, Deserialize};
use super::{debug, repo_name, clone_project, get_absp, query_candidates,run_refactoring, init, run_unit_tests, read_settings, Project, map_candidates};

fn run_exp_on_project(project: &Project) -> std::io::Result<()> {
    if project.skip.unwrap_or(false) {
        debug(&format!("skipping: {}\n", project.git_repo))?;
        return Ok(());
    }
    let repo_name = repo_name(&project.git_repo).unwrap();

    clone_project(&repo_name, &project.git_repo)?;
    
    let absp = get_absp(&repo_name, &project.subdir)?;

    run_unit_tests(&absp, &repo_name)?;

    eprintln!("Querying candidates for: {}", repo_name);
    
    let s = query_candidates(&absp)?;

    for candidate_output in map_candidates(&s) {
        debug(&format!("{:?}\n", candidate_output))?;
        
        for candidate in candidate_output.candidates {
            eprintln!("Applying: {}", candidate_output.refactoring);
            run_refactoring(&candidate_output.refactoring, candidate.from, candidate.to, &candidate.file, &absp)?;
        }
    }
    Ok(())
}

pub fn run_all_exp() -> std::io::Result<()> {
    init()?;
    debug("settings:\n")?;
    let settings = read_settings()?;
    debug(&format!("settings: {}\n", settings.projects.len()))?;

    for project in settings.projects {
        run_exp_on_project(&project)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateOutput {
    pub candidates: Vec<CandidatePosition>,
    pub crate_name: String,
    pub refactoring: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePosition {
    pub file: String,
    pub from: u32,
    pub to: u32
}