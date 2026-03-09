// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

//! This module holds the core compare logic of diff3.
pub mod params_diff3;

use std::{env::ArgsOs, ffi::OsString, iter::Peekable, process::ExitCode};

use crate::{
    arg_parser::{
        add_copyright, format_error_test, get_version_text, Executable, ParseError,
        TEXT_HELP_FOOTER,
    },
    diff3::params_diff3::{Diff3ParseOk, ParamsDiff3},
    utils,
};

// This contains the hard coded 'diff3'. If required this needs to be replaced with the executable.
// TODO finalize help text
pub const TEXT_HELP: &str = const_format::concatcp!(
    r#"
    diff3 is a utility which allows to compare three files line by line and 
    show differences in a human readable output. The files can also be merged 
    into a new one.

    Usage: diff3 [OPTIONS] VERSION_A ORIGINAL VERSION_B
    If a FILE is '-', read operating system's standard input.

    Options:
    -a, --text                  treat all files as text
    -A, --show-all              output all changes, bracketing conflicts
  
  
        --strip-trailing-cr     strip trailing carriage return on input
    -T, --initial-tab           make tabs line up by prepending a tab
        --diff-program=PROGRAM  use PROGRAM to compare files
    -L, --label=LABEL           use LABEL instead of file name
                                  (can be repeated up to three times)
  
    -m, --merge                 output actual merged file, according to
                                  -A if no other options are given

    Ed editor script output:
    -e, --ed                    output ed script incorporating changes
                                  from ORIGINAL to VERSION_B into VERSION_A
    -E, --show-overlap          like -e, but bracket conflicts
    -3, --easy-only             like -e, but incorporate only nonoverlapping changes
    -x, --overlap-only          like -e, but incorporate only overlapping changes
    -X                          like -x, but bracket conflicts
    -i                          append 'w' and 'q' commands to ed scripts

    -h  --help                  display this help and exit
    -v, --version               output version information and exit

    The --merge option causes diff3 to output a merged file. For unusual input, 
    this is more robust than using ed.

    Exit status is 0 if inputs are identical, 1 if different, 2 in error case.
    "#,
    TEXT_HELP_FOOTER
);

/// Entry into diff3.
///
/// Param options, e.g. 'diff3 file1.txt file2.txt -bd n2000kB'. \
/// diff3 options as documented in the GNU manual.
///
/// Ends program with Exit Status:
/// * 0 if inputs are identical
/// * 1 if inputs are different
/// * 2 in error case
pub fn main(mut args: Peekable<ArgsOs>) -> ExitCode {
    let Some(executable) = Executable::from_args_os(&mut args, false) else {
        eprintln!("Expected utility name as first argument, got nothing.");
        return ExitCode::FAILURE;
    };
    match diff3(args) {
        Ok(res) => match res {
            Diff3Ok::Different => ExitCode::FAILURE,
            Diff3Ok::Equal => ExitCode::SUCCESS,
            Diff3Ok::Help => {
                println!("{}", add_copyright(TEXT_HELP));
                ExitCode::SUCCESS
            }
            Diff3Ok::Version => {
                println!("{}", get_version_text(&executable));
                ExitCode::SUCCESS
            }
        },
        Err(e) => {
            let msg = format_error_test(&executable, &e);
            eprintln!("{msg}");
            ExitCode::from(2)
        }
    }
}

/// This is the full diff3 call.
///
/// The first arg needs to be the executable, then the operands and options.
#[allow(unused)] // until compare is written
pub fn diff3<I: Iterator<Item = OsString>>(mut args: Peekable<I>) -> Result<Diff3Ok, Diff3Error> {
    let Some(executable) = Executable::from_args_os(&mut args, false) else {
        return Err(ParseError::NoExecutable.into());
    };
    // read params
    let params = match ParamsDiff3::parse_params(&executable, args)? {
        Diff3ParseOk::Params(p) => p,
        Diff3ParseOk::Help => return Ok(Diff3Ok::Help),
        Diff3ParseOk::Version => return Ok(Diff3Ok::Version),
    };
    // dbg!("{params:?}");

    // compare files
    diff3_compare(&params)
}

/// This is the main function to compare the files. \
///
/// TODO diff3 compare functionality
#[allow(unused)] // until compare is written
pub fn diff3_compare(params: &ParamsDiff3) -> Result<Diff3Ok, Diff3Error> {
    if utils::is_same_file(&params.from, &params.to) {
        return Ok(Diff3Ok::Equal);
    }

    let (from_content, to_content) = match utils::read_both_files(&params.from, &params.to) {
        Ok(contents) => contents,
        Err(errors) => {
            let msg =
                utils::format_failure_to_read_input_files(&params.executable.executable(), &errors);
            dbg!(&msg);
            return Err(Diff3Error::ReadFileError(msg));
        }
    };

    // run diff
    Err(Diff3Error::NotYetImplemented)
}

/// The Ok result of diff3.
#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(unused)] // until compare is written
pub enum Diff3Ok {
    Different,
    Equal,
    Help,
    Version,
}

/// Errors for diff3.
///
/// To centralize error messages and make it easier to use in a lib.
#[derive(Debug, Clone, PartialEq)]
#[allow(unused)] // until compare is written
#[allow(clippy::enum_variant_names)]
pub enum Diff3Error {
    // parse errors
    ParseError(ParseError),
    NotYetImplemented,

    // compare errors
    OutputError(String),
    // (msg)
    ReadFileError(String),
}

impl std::error::Error for Diff3Error {}

impl From<ParseError> for Diff3Error {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e)
    }
}

impl std::fmt::Display for Diff3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Diff3Error::ParseError(e) => e.to_string(),
            Diff3Error::OutputError(msg) | Diff3Error::ReadFileError(msg) => msg.clone(),
            Diff3Error::NotYetImplemented => {
                "Diff3 compare functionality is not yet implemented".to_string()
            }
        };
        write!(f, "{msg}")
    }
}
