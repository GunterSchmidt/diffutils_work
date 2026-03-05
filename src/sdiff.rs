//! This module holds the core compare logic of sdiff.
pub mod params_sdiff;
pub mod params_sdiff_def;

use std::{
    env::ArgsOs,
    fmt::Display,
    io::{stdout, Write},
    iter::Peekable,
    process::ExitCode,
};

use crate::{
    sdiff::{params_sdiff::ParamsSdiff, params_sdiff_def::ParamsSdiffOk},
    side_diff, utils,
};

pub const EXE_NAME: &str = "sdiff";

/// Entry into sdiff.
///
/// Param options, e.g. 'sdiff file1.txt file2.txt -bd n2000kB'. \
/// sdiff options as documented at <https://www.gnu.org/software/diffutils/manual/html_node/sdiff-Options.html>
///
/// Exit codes are documented at
/// https://www.gnu.org/software/diffutils/manual/html_node/Invoking-sdiff.html \
/// Exit status is 0 if inputs are identical, 1 if different, 2 in error case.
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompareResultOk {
    Different,
    Equal,
}

/// Errors of core sdiff functionality.
/// To centralize error messages and make it easier to use in a lib.
#[derive(Debug, PartialEq)]
pub enum SdiffError {
    OutputError(String),
    // (msg)
    ReadFileError(String),
}

impl Display for SdiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SdiffError::OutputError(msg) => write!(f, "{msg}"),
            SdiffError::ReadFileError(msg) => write!(f, "{msg}"),
        }
    }
}

/// This is the main function to compare the files. \
/// Files are limited to u64 bytes and u64 lines.
/// TODO sdiff is missing a number of options, currently implemented:
/// * expand_tabs
/// * tabsize
/// * width
pub fn sdiff(params: &ParamsSdiff) -> Result<CompareResultOk, SdiffError> {
    let (from_content, to_content) = match utils::read_both_files(&params.from, &params.to) {
        Ok(contents) => contents,
        Err((filepath, error)) => {
            let msg = utils::format_failure_to_read_input_file(
                &params.util.executable(),
                &filepath,
                &error,
            );
            return Err(SdiffError::ReadFileError(msg));
        }
    };

    // run diff
    let mut output = stdout().lock();
    let result = side_diff::diff(&from_content, &to_content, &mut output, &params.into());

    match std::io::stdout().write_all(&result) {
        Ok(_) => {
            if result.is_empty() {
                Ok(CompareResultOk::Equal)
            } else {
                Ok(CompareResultOk::Different)
            }
        }
        Err(e) => Err(SdiffError::OutputError(e.to_string())),
    }

    // println!("\nsdiff does not compare files yet.");
    // println!("{:?} or {:?}?", SdiffOk::Different, SdiffOk::Equal);
    // Ok(SdiffOk::Equal)
}
