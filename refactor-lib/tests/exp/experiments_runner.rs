use serde::{Serialize, Deserialize};
use super::{debug, repo_name, clone_project, get_absp, query_candidates,run_refactoring, init, run_unit_tests, read_settings, Project, map_candidates};

struct ExperimentsRunner {
    project: Project,
    repo_name: String,
    absolute_path: std::path::PathBuf
}

impl ExperimentsRunner {
    pub fn new(project: Project) -> std::io::Result<Self> {
        let repo_name = repo_name(&project.git_repo).unwrap();
        let absolute_path = get_absp(&repo_name, &project.subdir)?;
        Ok(Self {
            project,
            repo_name,
            absolute_path
        })
    }

    fn query_candidates(&self) -> std::io::Result<Vec<CandidateOutput>> {
        let candidates = query_candidates(&self.absolute_path)?;
        Ok(map_candidates(&candidates))
    }

    fn run_exp_on_project(&self) -> std::io::Result<()> {
        clone_project(&self.repo_name, &self.project.git_repo)?;
        run_unit_tests(&self.absolute_path, &self.repo_name)?;
        eprintln!("Querying candidates for: {}", self.repo_name);
        for candidate_output in self.query_candidates()? {
            debug(&format!("{:?}\n", candidate_output))?;
            
            for candidate in candidate_output.candidates {
                eprintln!("Applying: {}", candidate_output.refactoring);
                run_refactoring(&candidate_output.refactoring, candidate.from, candidate.to, &candidate.file, &self.absolute_path)?;
            }
        }
        Ok(())
    }
}


pub fn run_all_exp() -> std::io::Result<()> {
    init()?;
    debug("settings:\n")?;
    let settings = read_settings()?;
    debug(&format!("settings: {}\n", settings.projects.len()))?;

    for project in settings.projects {
        if project.skip.unwrap_or(false) {
            debug(&format!("skipping: {}\n", project.git_repo))?;
            continue;
        }
        let experiments_runner = ExperimentsRunner::new(project)?;
        experiments_runner.run_exp_on_project()?;
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