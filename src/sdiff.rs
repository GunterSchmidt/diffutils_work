// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

//! This module holds the core compare logic of sdiff.
pub mod params_sdiff;

use std::{
    env::ArgsOs,
    ffi::OsString,
    fmt::Display,
    io::{stdout, Write},
    iter::Peekable,
    process::ExitCode,
};

use crate::{
    arg_parser::{self, add_copyright, Executable, ParseError, TEXT_HELP_FOOTER},
    sdiff::params_sdiff::{ParamsSDiff, SDiffParseOk},
    side_diff, utils,
};

pub type ResultSdiff = Result<SDiffOk, SDiffError>;

// This contains the hard coded 'sdiff'. If required this needs to be replaced with the executable.
pub const TEXT_HELP: &str = const_format::concatcp!(
    r#"
    sdiff is a tool which allows to compare two text files for differences.
    It outputs the differences in a side-by-side view.
    Use 'diff' for a row-by-row view.
    Use 'cmp' to compare binary files.
    
    Usage: sdiff [OPTIONS] [FILE]...
    If a FILE is '-', read operating system's standard input.

    Options:
    -o, --output=FILE                  operate interactively while sending output to FILE
        --diff-program=PROGRAM         use PROGRAM to compare files
    -a, --text                         treat all files as text
    -H, --speed-large-files            assume large files with many scattered small changes
    -d, --minimal                      try to find a smaller set of changes
        
    -i, --ignore-case                  do not distinguish between upper- and lower-case letters
    -E, --ignore-tab-expansion         ignore changes due to tab expansion
    -Z, --ignore-trailing-space        ignore white space at line end
    -b, --ignore-space-change          ignore changes in the amount of white space
    -W, --ignore-all-space             ignore all white space
    -B, --ignore-blank-lines           ignore changes whose lines are all blank
    -I, --ignore-matching-lines=REGEX  ignore changes all whose lines match REGEX expression
        --strip-trailing-cr            strip trailing carriage return on input

    -s, --suppress-common-lines        do not output common lines
    -l, --left-column                  output only the left column of common lines
    -t, --expand-tabs                  expand tabs to spaces in output
        --tabsize=NUM                  tab stops at every NUM (default 8) print columns
    -w, --width=NUM                    limit the print width to NUM print columns (default 130) 

    -h  --help                         display this help and exit
    -v, --version                      output version information and exit

    Exit status is 0 if inputs are identical, 1 if different, 2 in error case.
    "#,
    TEXT_HELP_FOOTER
);

/// Entry into sdiff.
///
/// Param options, e.g. 'sdiff file1.txt file2.txt -bd n2000kB'. \
/// sdiff options as documented in the GNU manual.
///
/// Exit codes are documented at
/// <https://www.gnu.org/software/diffutils/manual/html_node/Invoking-sdiff.html> \
/// Exit status is 0 if inputs are identical, 1 if different, 2 in error case.
pub fn main(mut args: Peekable<ArgsOs>) -> ExitCode {
    // I cannot think of a situation, where this is not an executable.
    let Some(executable) = Executable::from_args_os(&mut args, true) else {
        todo!("execute")
    };
    match sdiff(&executable, args) {
        Ok(res) => match res {
            SDiffOk::Different => ExitCode::FAILURE,
            SDiffOk::Equal => ExitCode::SUCCESS,
            SDiffOk::Help => {
                println!("{}", add_copyright(TEXT_HELP));
                ExitCode::SUCCESS
            }
            SDiffOk::Version => {
                println!("{}", arg_parser::get_version_text(&executable));
                ExitCode::SUCCESS
            }
        },
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

#[derive(Debug)]
pub enum SDiffOk {
    Different,
    Equal,
    Help,
    Version,
}

pub fn sdiff<I: Iterator<Item = OsString>>(
    executable: &Executable,
    args: Peekable<I>,
) -> ResultSdiff {
    // read params
    let params = match ParamsSDiff::parse_params(&executable, args) {
        Ok(res) => match res {
            SDiffParseOk::ParamsSdiff(p) => p,
            SDiffParseOk::Help => return Ok(SDiffOk::Help),
            SDiffParseOk::Version => return Ok(SDiffOk::Version),
        },
        Err(e) => return Err(e.into()),
    };
    // dbg!("{params:?}");

    // compare files
    sdiff_compare(&params)
}

/// This is the main function to compare the files. \
///
/// TODO sdiff is missing a number of options, currently implemented:
/// * expand_tabs
/// * tabsize
/// * width
/// * The output format does not match GNU sdiff
pub fn sdiff_compare(params: &ParamsSDiff) -> Result<SDiffOk, SDiffError> {
    if utils::is_same_file(&params.from, &params.to) {
        return Ok(SDiffOk::Equal);
    }
    let (from_content, to_content) = match utils::read_both_files(&params.from, &params.to) {
        Ok(contents) => contents,
        // Err((filepath, error)) => {
        Err(errors) => {
            let msg =
                utils::format_failure_to_read_input_files(&params.executable.executable(), &errors);
            return Err(SDiffError::ReadFileError(msg));
        }
    };

    // run diff
    let mut output = stdout().lock();
    let result = side_diff::diff(&from_content, &to_content, &mut output, &params.into());

    match std::io::stdout().write_all(&result) {
        Ok(_) => {
            if result.is_empty() {
                Ok(SDiffOk::Equal)
            } else {
                Ok(SDiffOk::Different)
            }
        }
        Err(e) => Err(SDiffError::OutputError(e.to_string())),
    }
}

/// Errors of core sdiff functionality.
///
/// To centralize error messages and make it easier to use in a lib.
#[derive(Debug, PartialEq)]
pub enum SDiffError {
    // parse errors
    ParseError(ParseError),

    // compare errors
    OutputError(String),
    // (msg)
    ReadFileError(String),
}

impl From<ParseError> for SDiffError {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e)
    }
}

impl Display for SDiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            SDiffError::ParseError(e) => e.to_string(),
            SDiffError::OutputError(msg) | SDiffError::ReadFileError(msg) => msg.clone(),
        };
        write!(f, "{msg}")
    }
}
