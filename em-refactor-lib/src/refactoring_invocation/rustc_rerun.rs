use em_refactor_lib_types::FileStringReplacement;
use crate::refactoring_invocation::{RefactoringErrorInternal, InMemoryFileLoader};
use std::sync::{Arc, Mutex};
use std::io::Write;
use rustc_session::DiagnosticOutput;
use serde_json::Value;

#[derive(Clone)]
pub(crate) struct StorageDiagnosticOutput {
    storage: Arc<Mutex<Vec<u8>>>,
}

impl Write for StorageDiagnosticOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut storage = self.storage.lock().unwrap();
        storage.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl StorageDiagnosticOutput {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(Vec::new()))
        }
    }
    pub fn errors(&self) -> String {
        let storage = self.storage.lock().unwrap();
        if let Ok(output) = std::str::from_utf8(&storage) {
            output.to_string()
        } else {
            "Compiler emitted invalid UTF8".to_owned()
        }
    }
}

struct CollectRustcErrorsCallbacks(StorageDiagnosticOutput);
impl rustc_driver::Callbacks for CollectRustcErrorsCallbacks {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        config.diagnostic_output =
            DiagnosticOutput::Raw(Box::new(self.0.clone()));
    }
}
pub fn rustc_rerun(changes: Vec<Vec<FileStringReplacement>>, rustc_args: &[String]) -> Result<(), RefactoringErrorInternal> {
    let mut default = CollectRustcErrorsCallbacks(StorageDiagnosticOutput::new());

    let mut file_loader = Box::new(InMemoryFileLoader::new(
        rustc_span::source_map::RealFileLoader,
    ));
    file_loader.add_changes(changes);

    let mut rustc_args = rustc_args
        .into_iter()
        .filter(|s| !s.starts_with(&"--error-format".to_owned()))
        .filter(|s| !s.starts_with(&"--color".to_owned()))
        .filter(|s| !s.starts_with(&"--json".to_owned()))
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    rustc_args.push("--error-format=json".to_owned());
    rustc_args.push("--color=never".to_owned());

    let err =
        rustc_driver::run_compiler(&rustc_args, &mut default, Some(file_loader), None);

    if err.is_err() {
        return Err(map_rustc_error_to_internal(default.0.errors()));
    }
    return Ok(());
}
fn map_rustc_error_to_internal(s: String) -> RefactoringErrorInternal {

    let vs = s
        .lines()
        .filter_map(|line| {
            let json: Value = serde_json::from_str(line).ok()?;

            Some(json["rendered"].as_str()?.to_string())
        })
        .collect::<Vec<_>>();
        
    let codes = s
        .lines()
        .filter_map(|line| {
            let json: Value = serde_json::from_str(line).ok()?;

            Some(json["code"].as_object()?["code"].as_str()?.to_string())
        })
        .collect::<Vec<_>>();


    RefactoringErrorInternal::recompile_err(&vs.join("\n"), codes)
}