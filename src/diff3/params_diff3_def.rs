//! This module holds all definitions, text and error messages for sdiff.

use std::fmt::Display;

use crate::{
    arg_parser::{AppOption, ArgParserError, OPT_HELP, OPT_VERSION},
    diff3::params_diff3::ParamsDiff3,
};

// TODO Help text
pub const TEXT_HELP: &str = "Help missing";

// TODO Version text
pub const TEXT_VERSION: &str = concat!("diff3 (Rust DiffUtils) ", env!("CARGO_PKG_VERSION"),);

// AppOptions for diff3_help
// TODO Check everything and add default values!

//   -A, --show-all              output all changes, bracketing conflicts
//   -e, --ed                    output ed script incorporating changes
//   -E, --show-overlap          like -e, but bracket conflicts
//   -3, --easy-only             like -e, but incorporate only non-overlapping changes
//   -x, --overlap-only          like -e, but incorporate only overlapping changes
//   -X                          like -x, but bracket conflicts
// long-name for-X: bracket-conflicts
//   -i                          append 'w' and 'q' commands to ed scripts
// long-name for-i: append-wq-to-ed
//   -m, --merge                 output actual merged file, according to
//                                 -A if no other options are given
//   -a, --text                  treat all files as text
//       --strip-trailing-cr     strip trailing carriage return on input
//   -T, --initial-tab           make tabs line up by prepending a tab
//       --diff-program=PROGRAM  use PROGRAM to compare files
//   -L, --label=LABEL           use LABEL instead of file name
//       --help                  display this help and exit
//   -v, --version               output version information and exit
pub(super) const OPT_APPEND_WQ_TO_ED: AppOption = AppOption {
    long_name: "append-wq-to-ed",
    short: Some('i'),
    has_arg: false,
};
pub(super) const OPT_BRACKET_CONFLICTS: AppOption = AppOption {
    long_name: "bracket-conflicts",
    short: Some('X'),
    has_arg: false,
};
pub(super) const OPT_DIFF_PROGRAM: AppOption = AppOption {
    long_name: "diff-program",
    short: None,
    has_arg: true,
};
pub(super) const OPT_EASY_ONLY: AppOption = AppOption {
    long_name: "easy-only",
    short: Some('3'),
    has_arg: false,
};
pub(super) const OPT_ED: AppOption = AppOption {
    long_name: "ed",
    short: Some('e'),
    has_arg: false,
};
pub(super) const OPT_INITIAL_TAB: AppOption = AppOption {
    long_name: "initial-tab",
    short: Some('T'),
    has_arg: false,
};
pub(super) const OPT_LABEL: AppOption = AppOption {
    long_name: "label",
    short: Some('L'),
    has_arg: true,
};
pub(super) const OPT_MERGE: AppOption = AppOption {
    long_name: "merge",
    short: Some('m'),
    has_arg: false,
};
pub(super) const OPT_OVERLAP_ONLY: AppOption = AppOption {
    long_name: "overlap-only",
    short: Some('x'),
    has_arg: false,
};
pub(super) const OPT_SHOW_ALL: AppOption = AppOption {
    long_name: "show-all",
    short: Some('A'),
    has_arg: false,
};
pub(super) const OPT_SHOW_OVERLAP: AppOption = AppOption {
    long_name: "show-overlap",
    short: Some('E'),
    has_arg: false,
};
pub(super) const OPT_STRIP_TRAILING_CR: AppOption = AppOption {
    long_name: "strip-trailing-cr",
    short: None,
    has_arg: false,
};
pub(super) const OPT_TEXT: AppOption = AppOption {
    long_name: "text",
    short: Some('a'),
    has_arg: false,
};

// Array for ArgParser
pub(super) const APP_OPTIONS: [AppOption; 15] = [
    OPT_APPEND_WQ_TO_ED,
    OPT_BRACKET_CONFLICTS,
    OPT_DIFF_PROGRAM,
    OPT_EASY_ONLY,
    OPT_ED,
    OPT_HELP,
    OPT_INITIAL_TAB,
    OPT_LABEL,
    OPT_MERGE,
    OPT_OVERLAP_ONLY,
    OPT_SHOW_ALL,
    OPT_SHOW_OVERLAP,
    OPT_STRIP_TRAILING_CR,
    OPT_TEXT,
    OPT_VERSION,
];

/// Success return type for parsing of params.
///
/// Successful parsing will return ParamsDiff3, \
/// -- help und --version will return an Info message, \
/// Error will be returned as [ParamsDiff3Error] in the function Result.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamsDiff3Ok {
    Info(String),
    ParamsDiff3(ParamsDiff3),
}

/// Contains all parser errors and their text messages.
#[derive(Debug, PartialEq)]
pub enum ParamsDiff3Error {
    /// Bubbled up error
    ArgParserError(ArgParserError),
    /// Having 3 operands or more
    /// (wrong operand)
    ExtraOperand(String),
}

impl From<ArgParserError> for ParamsDiff3Error {
    fn from(err: ArgParserError) -> Self {
        Self::ArgParserError(err)
    }
}

impl Display for ParamsDiff3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Generally error messages do not attempt to be GNU compatible.
        let msg = match self {
            ParamsDiff3Error::ArgParserError(e) => &e.to_string(),
            ParamsDiff3Error::ExtraOperand(opt) => &format!("extra operand '{opt}'"),
        };
        write!(f, "{msg}")
        //  // Writes the error message, adds cmp: and the --help information.
        //         fn write_err(f: &mut std::fmt::Formatter<'_>, msg: &str) -> Result<(), std::fmt::Error> {
        //             arg_parser::write_err(f, EXE_NAME, msg)
        //         }
        //
        //         // TODO Short and Long name errors
        //         match self {
        //             ParamsDiff3Error::ArgParserError(e) => write_err(f, &e.to_string()),
        //
        //             ParamsDiff3Error::ExtraOperand(opt) => write_err(f, &format!("extra operand '{opt}'")),
        //         }
    }
}
