use std::path::Path;
use super::rustc_utils::get_sys_root;

pub fn arg_value<'a>(
    args: impl IntoIterator<Item = &'a String>,
    find_arg: &str,
    pred: impl Fn(&str) -> bool,
) -> Option<&'a str> {
    let mut args = args.into_iter().map(String::as_str);

    while let Some(arg) = args.next() {
        let arg: Vec<_> = arg.splitn(2, '=').collect();
        if arg.get(0) != Some(&find_arg) {
            continue;
        }

        let value = arg.get(1).cloned().or_else(|| args.next());
        if value.as_ref().map_or(false, |p| pred(p)) {
            return value;
        }
    }
    None
}

// Call compiler with refactoring tools callbacks
// Args to the compiler: file, sysroot, ++
// args to the refactoring tools: refactoringargs
// returns: a set of changes
//
fn is_wrapper_mode(args: &[String]) -> bool {
    Path::new(&args[1]).file_stem() == Some("rustc".as_ref())
}
fn get_file_path(args: &[String]) -> Option<&String> {
    args.iter().find(|s| !s.starts_with('-'))
}
pub fn get_refactor_args(args: &[String]) -> Vec<String> {
    if is_wrapper_mode(&args) {
        std::env::var("MY_REFACTOR_ARGS")
            .unwrap()
            .split(';')
            .map(|s| s.to_string())
            .collect()
    } else {
        let mut ret = args
            .iter()
            .skip_while(|s| *s != "--")
            .skip(1)
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        ret.push(format!("--file={}", get_file_path(args).unwrap()));
        ret
    }
}

///
/// Collect all arguments until '--', which should be passed to rustc
///
pub fn get_compiler_args(args: &[String]) -> Vec<String> {
    let have_sys_root = arg_value(args, "--sysroot", |_| true).is_some();
    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this/
    let wrapper_mode = Path::new(&args[1]).file_stem() == Some("rustc".as_ref());

    let mut rustc_args: Vec<_>;

    if wrapper_mode {
        // we still want to be able to invoke it normally though
        rustc_args = args.iter().skip(1).map(|s| s.to_string()).collect();
    } else {
        rustc_args = args
            .iter()
            .skip(1)
            .take_while(|s| *s != "--")
            .map(|s| s.to_string())
            .collect();
        rustc_args.insert(0, "".to_owned());
    }

    // this conditional check for the --sysroot flag is there so users can call
    // `clippy_driver` directly
    // without having to pass --sysroot or anything
    if !have_sys_root {
        rustc_args.push("--sysroot".to_owned());
        rustc_args.push(get_sys_root());
    }
    rustc_args.push("--allow".to_owned());
    rustc_args.push("dead_code".to_owned());
    rustc_args.push("--allow".to_owned());
    rustc_args.push("deprecated".to_owned());
    rustc_args.push("--allow".to_owned());
    rustc_args.push("unused".to_owned());

    rustc_args
}
