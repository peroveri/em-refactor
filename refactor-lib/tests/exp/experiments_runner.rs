use serde::{Serialize, Deserialize};
use super::{debug, repo_name, clone_project, get_absp, query_candidates,run_refactoring, init, run_unit_tests, read_settings, Project, map_candidates, write_result};
use my_refactor_lib::RefactorOutputs;

struct ExperimentsRunner {
    project: Project,
    repo_name: String,
    absolute_path: std::path::PathBuf,
    refactoring: String
}

impl ExperimentsRunner {
    pub fn new(project: Project, refactoring: &str) -> std::io::Result<Self> {
        let repo_name = repo_name(&project.git_repo).unwrap();
        let absolute_path = get_absp(&repo_name, &project.subdir)?;
        Ok(Self {
            project,
            repo_name,
            absolute_path,
            refactoring: refactoring.to_string()
        })
    }

    fn query_candidates(&self) -> std::io::Result<RefactorOutputs> {
        let candidates = query_candidates(&self.absolute_path, &self.refactoring)?;
        Ok(map_candidates(&candidates))
    }
    fn output_result(&self, candidates: &RefactorOutputs) -> std::io::Result<()> {
        let r = map_candidate_out_to_summary(candidates);
        // write_result(&serde_json::to_string_pretty(candidates).unwrap(), &format!("{}-full", self.repo_name))?;
        write_result(&serde_json::to_string_pretty(&r).unwrap(), &format!("{}-candidates", self.repo_name))
    }
    fn run_exp_on_project(&self) -> std::io::Result<()> {
        clone_project(&self.repo_name, &self.project.git_repo)?;
        run_unit_tests(&self.absolute_path, &self.repo_name)?;
        let candidates = self.query_candidates()?;
        self.output_result(&candidates)?;
        for candidate_output in candidates.candidates {
            debug(&format!("{:?}\n", candidate_output))?;
            
            for candidate in candidate_output.candidates {
                eprintln!("Applying: {}", candidate_output.refactoring);
                run_refactoring(&candidate_output.refactoring, candidate.from, candidate.to, &candidate.file, &self.absolute_path)?;
            }
        }
        Ok(())
    }
}


pub fn run_all_exp(refactoring: &str) -> std::io::Result<()> {
    init()?;
    debug("settings:\n")?;
    let settings = read_settings()?;
    debug(&format!("settings: {}\n", settings.projects.len()))?;

    for project in settings.projects {
        if project.skip.unwrap_or(false) {
            debug(&format!("skipping: {}\n", project.git_repo))?;
            continue;
        }
        let experiments_runner = ExperimentsRunner::new(project, refactoring)?;
        experiments_runner.run_exp_on_project()?;
    }
    Ok(())
}
fn map_candidate_out_to_summary(candidates: &RefactorOutputs) -> CandidateSummary {
    CandidateSummary {
        items: candidates.candidates.iter().map(|c| CandidateSummaryItem {
            count: c.candidates.len(),
            crate_name: c.crate_name.to_string(),
            refactoring: c.refactoring.to_string()
        }).collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSummary {
    pub items: Vec<CandidateSummaryItem>
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSummaryItem {
    pub crate_name: String,
    pub refactoring: String,
    pub count: usize
}