// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

// pub mod params;
pub mod params_diff3;
pub mod params_diff3_def;

use std::{env::ArgsOs, iter::Peekable, process::ExitCode};

use crate::diff3::{
    params_diff3::ParamsDiff3,
    params_diff3_def::{ParamsDiff3Error, ParamsDiff3Ok},
};

pub const EXE_NAME: &str = "diff3";

/// Entry into diff3.
///
/// Param options, e.g. 'diff3 file1.txt file2.txt -bd n2000kB'. \
/// diff3 options as documented at <https://www.gnu.org/software/diffutils/manual/html_node/diff3-Options.html>
///
/// Exit codes are documented at
/// https://www.gnu.org/software/diffutils/manual/html_node/Invoking-diff3.html \
/// Exit status is 0 if inputs are identical, 1 if different, 2 in error case.
pub fn main(opts: Peekable<ArgsOs>) -> ExitCode {
    let params = match ParamsDiff3::parse_params(opts) {
        Ok(res) => match res {
            ParamsDiff3Ok::Info(info) => {
                println!("{info}");
                return ExitCode::from(0);
            }
            ParamsDiff3Ok::ParamsDiff3(params) => params,
        },
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };

    if params.from == "-" && params.to == "-"
        || same_file::is_same_file(&params.from, &params.to).unwrap_or(false)
    {
        return ExitCode::SUCCESS;
    }

    //run diff3 comparison
    match diff3(&params) {
        Ok(CompareResultOk::Equal) => ExitCode::SUCCESS,
        Ok(CompareResultOk::Different) => ExitCode::from(1),
        Err(e) => {
            // if !params.silent {
            eprintln!("{e}");
            // }
            ExitCode::from(2)
        }
    }
}

#[derive(Debug)]
pub enum CompareResultOk {
    Different,
    Equal,
}

fn diff3(params: &ParamsDiff3) -> Result<CompareResultOk, ParamsDiff3Error> {
    dbg!(params);
    eprintln!("\n\ndiff3 comparison is not yet implemented");
    std::process::exit(2);
}
