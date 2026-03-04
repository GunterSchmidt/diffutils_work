//! This module holds the core compare logic of sdiff.
mod params_sdiff;
mod params_sdiff_def;

use std::{env::ArgsOs, fmt::Display, iter::Peekable, process::ExitCode};

use crate::sdiff::{params_sdiff::ParamsSdiff, params_sdiff_def::ParamsSdiffOk};

pub const EXE_NAME: &str = "sdiff";

/// Entry into sdiff.
///
/// Param options, e.g. 'sdiff file1.txt file2.txt -bd n2000kB'. \
/// sdiff options as documented at <https://www.gnu.org/software/diffutils/manual/html_node/sdiff-Options.html>
///
/// Exit codes are documented at
/// https://www.gnu.org/software/diffutils/manual/html_node/Invoking-sdiff.html \
/// Exit status is 0 if inputs are identical, 1 if different, 2 in error case.
// TODO first param util: DiffUtility,
pub fn main(opts: Peekable<ArgsOs>) -> ExitCode {
    let params = match ParamsSdiff::parse_params(opts) {
        Ok(res) => match res {
            ParamsSdiffOk::Info(info) => {
                println!("{info}");
                return ExitCode::from(0);
            }
            ParamsSdiffOk::ParamsSdiff(params) => params,
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

    match sdiff(&params) {
        Ok(SdiffOk::Equal) => ExitCode::SUCCESS,
        Ok(SdiffOk::Different) => ExitCode::from(1),
        Err(e) => {
            // if !params.silent {
            eprintln!("{e}");
            // }
            ExitCode::from(2)
        }
    }
}

#[derive(Debug)]
pub enum SdiffOk {
    Different,
    Equal,
}

/// Errors of core sdiff functionality.
/// To centralize error messages and make it easier to use in a lib.
#[derive(Debug, PartialEq)]
pub enum SdiffError {
    // Dummy,
}

impl Display for SdiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO Display errors
        write!(f, "Error message of Sdiff")
    }
}

/// This is the main function to compare the files. \
/// Files are limited to u64 bytes and u64 lines.
pub fn sdiff(_params: &ParamsSdiff) -> Result<SdiffOk, SdiffError> {
    // TODO sdiff file compare
    // There seems to be a lot of similarity to diff, mainly a different output.
    println!("\nsdiff does not compare files yet.");
    println!("{:?} or {:?}?", SdiffOk::Different, SdiffOk::Equal);
    Ok(SdiffOk::Equal)
}
