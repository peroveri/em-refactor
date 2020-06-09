mod exp;
use clap::{Arg, App};
use log::{SetLoggerError, LevelFilter};
use chrono::prelude::*;

pub fn init_logger(options: &exp::ExperimentOptions) -> Result<(), SetLoggerError> {
     if options.log_to_file {
          log::set_boxed_logger(Box::new(
               exp::FileLogger::new(format!("{}.txt", options.get_file_prefix()))
          )).map(|()| log::set_max_level(LevelFilter::Info))
     } else {
          log::set_boxed_logger(Box::new(exp::StdoutLogger))
               .map(|()| log::set_max_level(LevelFilter::Info))
     }
}
fn app<'a, 'b>() -> App<'a, 'b> {
     App::new("Refactoring experiments runner")
          .version("0.0.1")
          .author("Per Ove Ringdal <peroveri@gmail.com>")
          // .arg(Arg::with_name("target-dir")
          //    .long("target-dir")
          //    .takes_value(true))
          .arg(Arg::with_name("workspace-root")
               .long("workspace-root")
               .takes_value(true))
          .arg(Arg::with_name("refactoring")
               .takes_value(true)
               .required(true))
          .arg(Arg::with_name("log-to-file")
               .long("log-to-file"))
          .arg(Arg::with_name("only-file")
               .long("only-file")
               .takes_value(true))
}

// given project, already in file system
// run experiment --refactoring=extract-method

fn main() -> std::io::Result<()> {
     let matches = app().get_matches();
     let options = exp::ExperimentOptions {
          refactoring: matches.value_of("refactoring").unwrap().to_string(),
          workspace_root: matches.value_of("workspace-root").unwrap().to_string(),
          started_at: format!("{}", Local::now().format("%Y-%m-%d_%H_%M")),
          log_to_file: matches.is_present("log-to-file"),
          only_file: matches.value_of("only-file").map(|s| s.to_string())
     };
     init_logger(&options).unwrap();
     exp::run_all_exp(options)
}
