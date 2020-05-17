use log::{Record, Level, Metadata};
use std::fs::OpenOptions;
use std::io::prelude::*;

pub struct FileLogger {
    file: String
}

impl FileLogger {
    pub fn new(file: String) -> Self {
        Self {
            file
        }
    }
}

impl log::Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
            let mut file = OpenOptions::new()
                .create(true).append(true).open(&self.file)
                .unwrap();

            file.write_all(format!("{} - {}\n", record.level(), record.args()).as_bytes()).unwrap();
        }
    }

    fn flush(&self) {}
}