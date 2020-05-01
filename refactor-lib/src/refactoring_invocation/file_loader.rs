use std::io;
use std::path::{Path, PathBuf};
use rustc_span::source_map::{FileLoader};
use refactor_lib_types::FileStringReplacement;

///
/// Used for running the compiler with modified files without having to write the modifications to the filesystem.
/// 
#[derive(Clone)]
pub struct InMemoryFileLoader<T: FileLoader + Send + Sync> {
    inner_file_loader: T,
    changes: Vec<Vec<FileStringReplacement>>
}
impl<T: FileLoader + Send + Sync> InMemoryFileLoader<T> {
    pub fn new(inner: T) -> InMemoryFileLoader<T> {
        InMemoryFileLoader {
            inner_file_loader: inner,
            changes: vec![]
        }
    }

    pub fn add_changes(&mut self, changes: Vec<Vec<FileStringReplacement>>) {
        self.changes.extend(changes);
    }
}

impl<T: FileLoader + Send + Sync> FileLoader for InMemoryFileLoader<T> {
    fn file_exists(&self, path: &Path) -> bool {
        self.inner_file_loader.file_exists(path)
    }

    fn abs_path(&self, _: &Path) -> Option<PathBuf> {
        None
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        let mut content = self.inner_file_loader.read_file(path)?;

        for changes in &self.changes {
            let mut changes =  changes.clone();
            changes.sort_by_key(|c| c.byte_start);
            changes.reverse();
            for change in changes {
                if change.file_name == path.file_name().unwrap().to_str().unwrap() || change.file_name == path.to_str().unwrap() {
                    let s1 = &content[..(change.byte_start) as usize];
                    let s2 = &content[(change.byte_end) as usize..];
                    content = format!("{}{}{}", s1, change.replacement, s2);
                }
            }
        }

        Ok(content)
    }
}