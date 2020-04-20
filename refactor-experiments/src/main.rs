mod exp;
use clap::{Arg, App, SubCommand};

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("Refactoring experiments runner")
     .version("1.0")
     .author("Per Ove Ringdal <peroveri@gmail.com>")
     .arg(Arg::with_name("path")
          .short("p")
          .long("path")
          .value_name("PATH")
          .help("Sets the path")
          .takes_value(true))
     .arg(Arg::with_name("v")
          .short("v")
          .multiple(true)
          .help("Sets the level of verbosity"))
     .subcommand(SubCommand::with_name("run")
               .about("controls testing features")
               .arg(Arg::with_name("debug")
                    .short("d")
                    .help("print debug information verbosely")))
}


fn run_all() -> std::io::Result<()> {
     exp::run_all_exp("extract-block")
}

// given project, already in file system
// run experiment --refactoring=extract-method

fn main() -> std::io::Result<()> {
    // exp::run_all_exp("box-named-field")
    // TODO: we probably don't want to query candidates for the tests
    match app().get_matches().subcommand() {
     ("run", Some(_)) => {
          run_all()
     },
     _ => {
          Ok(())
     }
    }
}
