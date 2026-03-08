// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

use std::{
    env::ArgsOs,
    ffi::OsStr,
    iter::Peekable,
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::arg_parser::Executable;
// use predicates::name;

mod arg_parser;
mod cmp;
mod context_diff;
mod diff;
// mod diff3;
mod ed_diff;
mod macros;
mod normal_diff;
mod params;
mod sdiff;
mod side_diff;
mod unified_diff;
mod utils;

/// # Panics
/// Panics if the binary path cannot be determined
fn binary_path(args: &mut Peekable<ArgsOs>) -> PathBuf {
    match args.peek() {
        Some(ref s) if !s.is_empty() => PathBuf::from(s),
        _ => std::env::current_exe().unwrap(),
    }
}

/// #Panics
/// Panics if path has no UTF-8 valid name
fn name(binary_path: &Path) -> &OsStr {
    binary_path.file_stem().unwrap()
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage(name: &str) {
    println!("{name} {VERSION} (multi-call binary)\n");
    println!("Usage: {name} [function [arguments...]]\n");
    println!("Currently defined functions:\n");
    println!("    cmp, diff\n");
}

fn second_arg_error(name: &OsStr) -> ! {
    eprintln!("Expected utility name as second argument, got nothing.");
    usage(&name.to_string_lossy());
    std::process::exit(0);
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().peekable();

    let exe_path = binary_path(&mut args);
    let exe_name = name(&exe_path);

    if exe_name == "diffutils" {
        // Discard the item we peeked.
        let _ = args.next();
    };

    let Some(executable) = Executable::from_args_os(&mut args, false) else {
        second_arg_error(exe_name)
    };
    match executable {
        Executable::Cmp => cmp::main(args),
        Executable::Diff => diff::main(args),
        // Executable::Diff3 => diff3::main(args),
        // Executable::Patch => todo!(),
        Executable::SDiff => sdiff::main(args),
        _ => {
            eprintln!("{executable}: utility not supported");
            ExitCode::from(2)
        }
    }
}
