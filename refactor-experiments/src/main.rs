mod exp;
use clap::{Arg, App};
use log::{SetLoggerError, LevelFilter};

static LOGGER: exp::SimpleLogger = exp::SimpleLogger;

pub fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
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
          .arg(Arg::with_name("v")
               .short("v")
               .multiple(true)
               .help("Sets the level of verbosity"))
}

// given project, already in file system
// run experiment --refactoring=extract-method

fn main() -> std::io::Result<()> {
     init_logger().unwrap();
     let matches = app().get_matches();
     exp::run_all_exp(
         matches.value_of("refactoring").unwrap(),
         matches.value_of("workspace-root").unwrap()
     )
}
