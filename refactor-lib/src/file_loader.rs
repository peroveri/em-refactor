use std::io;
use std::path::{Path, PathBuf};
use syntax::source_map::{FileLoader};
use crate::change::Change;

///
/// Used for running the compiler with modified files without having to write the modifications to the filesystem.
/// 
#[derive(Clone)]
pub(crate) struct InMemoryFileLoader<T: FileLoader + Send + Sync> {
    inner_file_loader: T,
    changes: Vec<Change>
}
impl<T: FileLoader + Send + Sync> InMemoryFileLoader<T> {
    pub fn new(inner: T) -> InMemoryFileLoader<T> {
        InMemoryFileLoader {
            inner_file_loader: inner,
            changes: vec![]
        }
    }

    pub fn add_changes(&mut self, changes: Vec<Change>) {
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
        let mut changes =  self.changes.clone();
        changes.sort_by_key(|c| c.start);
        changes.reverse();
        for change in changes {
            if change.file_name == path.file_name().unwrap().to_str().unwrap() || change.file_name == path.to_str().unwrap() {
                let s1 = &content[..(change.start - change.file_start_pos) as usize];
                let s2 = &content[(change.end - change.file_start_pos) as usize..];
                content = format!("{}{}{}", s1, change.replacement, s2);
            }
        }

        Ok(content)
    }
}