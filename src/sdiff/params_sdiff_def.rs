//! This module holds all definitions, text and error messages for sdiff.
use std::fmt::Display;

use const_format::concatcp;

use crate::{
    arg_parser::{
        AppOption, ArgParserError, ParsedOption, OPT_HELP, OPT_VERSION, TEXT_COPYRIGHT,
        TEXT_HELP_FOOTER,
    },
    sdiff::{params_sdiff::ParamsSdiff, EXE_NAME},
    // sdiff::{Bytes, IgnInit, EXE_NAME},
};

pub type ResultParamsSdiffParse = Result<ParamsSdiffOk, ParamsSdiffError>;

// AppOptions for sdiff
pub(super) const OPT_DIFF_PROGRAM: AppOption = AppOption {
    long_name: "diff-program",
    short: None,
    has_arg: true,
    arg_default: None,
};
pub(super) const OPT_EXPAND_TABS: AppOption = AppOption {
    long_name: "expand-tabs",
    short: Some('t'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_IGNORE_ALL_SPACE: AppOption = AppOption {
    long_name: "ignore-all-space",
    short: Some('W'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_IGNORE_BLANK_LINES: AppOption = AppOption {
    long_name: "ignore-blank-lines",
    short: Some('B'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_IGNORE_CASE: AppOption = AppOption {
    long_name: "ignore-case",
    short: Some('i'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_IGNORE_MATCHING_LINES: AppOption = AppOption {
    long_name: "ignore-matching-lines",
    short: Some('I'),
    has_arg: true,
    arg_default: None,
};
pub(super) const OPT_IGNORE_SPACE_CHANGE: AppOption = AppOption {
    long_name: "ignore-space-change",
    short: Some('b'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_IGNORE_TAB_EXPANSION: AppOption = AppOption {
    long_name: "ignore-tab-expansion",
    short: Some('E'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_IGNORE_TRAILING_SPACE: AppOption = AppOption {
    long_name: "ignore-trailing-space",
    short: Some('Z'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_LEFT_COLUMN: AppOption = AppOption {
    long_name: "left-column",
    short: Some('l'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_MINIMAL: AppOption = AppOption {
    long_name: "minimal",
    short: Some('d'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_OUTPUT: AppOption = AppOption {
    long_name: "output",
    short: Some('o'),
    has_arg: true,
    arg_default: None,
};
pub(super) const OPT_SPEED_LARGE_FILES: AppOption = AppOption {
    long_name: "speed-large-files",
    short: Some('H'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_STRIP_TRAILING_CR: AppOption = AppOption {
    long_name: "strip-trailing-cr",
    short: None,
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_SUPPRESS_COMMON_LINES: AppOption = AppOption {
    long_name: "suppress-common-lines",
    short: Some('s'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_TABSIZE: AppOption = AppOption {
    long_name: "tabsize",
    short: None,
    has_arg: true,
    arg_default: Some("8"),
};
pub(super) const OPT_TEXT: AppOption = AppOption {
    long_name: "text",
    short: Some('a'),
    has_arg: false,
    arg_default: None,
};
pub(super) const OPT_WIDTH: AppOption = AppOption {
    long_name: "width",
    short: Some('w'),
    has_arg: true,
    arg_default: None,
};

// Array for ParamsGen
pub(super) const ARG_OPTIONS: [AppOption; 20] = [
    OPT_DIFF_PROGRAM,
    OPT_EXPAND_TABS,
    OPT_HELP,
    OPT_IGNORE_ALL_SPACE,
    OPT_IGNORE_BLANK_LINES,
    OPT_IGNORE_CASE,
    OPT_IGNORE_MATCHING_LINES,
    OPT_IGNORE_SPACE_CHANGE,
    OPT_IGNORE_TAB_EXPANSION,
    OPT_IGNORE_TRAILING_SPACE,
    OPT_LEFT_COLUMN,
    OPT_MINIMAL,
    OPT_OUTPUT,
    OPT_SPEED_LARGE_FILES,
    OPT_STRIP_TRAILING_CR,
    OPT_SUPPRESS_COMMON_LINES,
    OPT_TABSIZE,
    OPT_TEXT,
    OPT_VERSION,
    OPT_WIDTH,
];

// TODO Help text rewrite, this is copyrighted by GNU
pub const TEXT_HELP: &str = concatcp!(
    r#"
Usage: sdiff FILE1 FILE2 [OPTIONs] 
Options: Any number of options, may also be in front of the file names. 

sdiff is a tool which allows to compare two text files for differences.
It outputs the differences in a side-by-side view.
Use 'diff' for a row-by-row view.
Use 'cmp' to compare binary files

Options:
  -o, --output=FILE            operate interactively, sending output to FILE

  -i, --ignore-case            consider upper- and lower-case to be the same
  -E, --ignore-tab-expansion   ignore changes due to tab expansion
  -Z, --ignore-trailing-space  ignore white space at line end
  -b, --ignore-space-change    ignore changes in the amount of white space
  -W, --ignore-all-space       ignore all white space
  -B, --ignore-blank-lines     ignore changes whose lines are all blank
  -I, --ignore-matching-lines=RE  ignore changes all whose lines match RE
      --strip-trailing-cr      strip trailing carriage return on input
  -a, --text                   treat all files as text

  -w, --width=NUM              output at most NUM (default 130) print columns
  -l, --left-column            output only the left column of common lines
  -s, --suppress-common-lines  do not output common lines

  -t, --expand-tabs            expand tabs to spaces in output
      --tabsize=NUM            tab stops at every NUM (default 8) print columns

  -d, --minimal                try hard to find a smaller set of changes
  -H, --speed-large-files      assume large files, many scattered small changes
      --diff-program=PROGRAM   use PROGRAM to compare files

      --help                   display this help and exit
  -v, --version                output version information and exit

If a FILE is '-', read operating system's standard input.
Exit status is 0 if inputs are identical, 1 if different, 2 in error case.
"#,
    TEXT_HELP_FOOTER
);

// TODO Version text, possibly centralized.
pub const TEXT_VERSION: &str = concat!("sdiff (Rust DiffUtils) ", env!("CARGO_PKG_VERSION"),);

/// Success return type for parsing of params.
///
/// Successful parsing will return ParamsSdiff, \
/// '-- help' und '--version' will return an [ParamsSdiffInfo] enum, \
/// Error will be returned as [ParamsSdiffError] in the function Result.
#[derive(Debug, PartialEq)]
pub enum ParamsSdiffOk {
    Info(ParamsSdiffInfo),
    ParamsSdiff(ParamsSdiff),
}

/// Static texts for '--help' and '--version'.
///
/// The parser returns these enums to the caller, allowing to identify this as information,
/// so the program exit code is SUCCESS(0).
#[derive(Debug, PartialEq)]
pub enum ParamsSdiffInfo {
    Help,
    Version,
}

impl Display for ParamsSdiffInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = match self {
            ParamsSdiffInfo::Help => TEXT_HELP,
            ParamsSdiffInfo::Version => &format!("{TEXT_VERSION}\n{TEXT_COPYRIGHT}"),
        };

        write!(f, "{}", info)
    }
}

/// Contains all parser errors and their text messages.
/// This allows centralized maintenance.
#[derive(Debug, PartialEq)]
pub enum ParamsSdiffError {
    /// Bubbled up error
    ArgParserError(ArgParserError),

    /// number argument incorrect
    InvalidNumber(ParsedOption),

    // (param argument)
    #[allow(dead_code)] // unclear why this triggers, it is used
    WidthInvalid(String),

    /// Having 3 operands or more
    /// (wrong operand)
    ExtraOperand(String),
    // TODO more errors
}

impl From<ArgParserError> for ParamsSdiffError {
    fn from(err: ArgParserError) -> Self {
        Self::ArgParserError(err)
    }
}

impl Display for ParamsSdiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Writes the error message, adds sdiff: and the --help information.
        fn write_err(f: &mut std::fmt::Formatter<'_>, msg: &str) -> Result<(), std::fmt::Error> {
            ArgParserError::write_err(f, EXE_NAME, msg)
        }

        // TODO Different error messages for Short and Long name calls? Generally error messages do not attempt to be GNU compatible.
        match self {
            ParamsSdiffError::ArgParserError(e) => write_err(f, &e.to_string()),
            ParamsSdiffError::ExtraOperand(opt) => write_err(f, &format!("extra operand '{opt}'")),
            ParamsSdiffError::InvalidNumber(opt) => write_err(
                f,
                &format!(
                    "invalid argument '{}' for '--{}'{}",
                    opt.arg_for_option_or_empty_string(),
                    opt.app_option.long_name,
                    opt.short_char_or_empty_string(),
                ),
            ),
            ParamsSdiffError::WidthInvalid(param) => write_err(f, &format!("invalid '{param}'")),
        }
    }
}
