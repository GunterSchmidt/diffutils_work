// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

use std::{ffi::OsStr, process::ExitCode};

use uudiff::arg_parser::Executable;

// use diffutils::{arg_parser::Executable, cmp, diff3, sdiff};

// mod context_diff;
// mod diff;
// mod ed_diff;
// mod macros;
// mod normal_diff;
// mod params;
// mod side_diff;
// mod unified_diff;
// mod utils;

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
    let mut args = uucore::args_os().peekable();

    let executable = match Executable::get_util(&mut args) {
        Ok(info) => info.executable,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    let code = match executable {
        // Executable::Cmp => cmp::main(args),
        // Executable::Diff => diff::main(args),
        Executable::DiffUtils(name) => second_arg_error(&name),
        Executable::Diff3 => diff3::uumain(args),
        // // Executable::Patch => todo!(),
        Executable::SDiff => sdiff::uumain(args),
        _ => {
            eprintln!("{executable}: utility not supported");
            2
        }
    };
    ExitCode::from(code as u8)
}
