// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

//! This module contains the Parser for diff3 arguments.
use std::{ffi::OsString, iter::Peekable};

use crate::arg_parser::{AppOption, Executable, ParseError, Parser, OPT_HELP, OPT_VERSION};

pub type ResultSdiffParse = Result<Diff3ParseOk, ParseError>;

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
pub const OPT_APPEND_WQ_TO_ED: AppOption = AppOption {
    long_name: "append-wq-to-ed",
    short: Some('i'),
    has_arg: false,
};
pub const OPT_BRACKET_CONFLICTS: AppOption = AppOption {
    long_name: "bracket-conflicts",
    short: Some('X'),
    has_arg: false,
};
pub const OPT_DIFF_PROGRAM: AppOption = AppOption {
    long_name: "diff-program",
    short: None,
    has_arg: true,
};
pub const OPT_EASY_ONLY: AppOption = AppOption {
    long_name: "easy-only",
    short: Some('3'),
    has_arg: false,
};
pub const OPT_ED: AppOption = AppOption {
    long_name: "ed",
    short: Some('e'),
    has_arg: false,
};
pub const OPT_INITIAL_TAB: AppOption = AppOption {
    long_name: "initial-tab",
    short: Some('T'),
    has_arg: false,
};
pub const OPT_LABEL: AppOption = AppOption {
    long_name: "label",
    short: Some('L'),
    has_arg: true,
};
pub const OPT_MERGE: AppOption = AppOption {
    long_name: "merge",
    short: Some('m'),
    has_arg: false,
};
pub const OPT_OVERLAP_ONLY: AppOption = AppOption {
    long_name: "overlap-only",
    short: Some('x'),
    has_arg: false,
};
pub const OPT_SHOW_ALL: AppOption = AppOption {
    long_name: "show-all",
    short: Some('A'),
    has_arg: false,
};
pub const OPT_SHOW_OVERLAP: AppOption = AppOption {
    long_name: "show-overlap",
    short: Some('E'),
    has_arg: false,
};
pub const OPT_STRIP_TRAILING_CR: AppOption = AppOption {
    long_name: "strip-trailing-cr",
    short: None,
    has_arg: false,
};
pub const OPT_TEXT: AppOption = AppOption {
    long_name: "text",
    short: Some('a'),
    has_arg: false,
};

// Array for ArgParser
pub const APP_OPTIONS: [AppOption; 15] = [
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

// These options throw an error, rather than go unnoticed.
#[cfg(feature = "feat_check_not_yet_implemented")]
pub const NOT_YET_IMPLEMENTED: [AppOption; 13] = [
    OPT_APPEND_WQ_TO_ED,
    OPT_BRACKET_CONFLICTS,
    OPT_DIFF_PROGRAM,
    OPT_EASY_ONLY,
    OPT_ED,
    OPT_INITIAL_TAB,
    OPT_LABEL,
    OPT_MERGE,
    OPT_OVERLAP_ONLY,
    OPT_SHOW_ALL,
    OPT_SHOW_OVERLAP,
    OPT_STRIP_TRAILING_CR,
    OPT_TEXT,
];

/// Parser Result Ok Enum with Params.
///
/// # Returns
/// * Params in normal cases
/// * Just Help or Version when these are requested as the params are then not relevant.
///
/// Error will be returned as [ParseError] in the function Result Error.
#[derive(Debug, PartialEq)]
pub enum Diff3ParseOk {
    Params(ParamsDiff3),
    Help,
    Version,
}

// The Param struct
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParamsDiff3 {
    /// Identifier
    pub executable: Executable,
    pub from: OsString,
    pub to: OsString,
    //  -i                          append 'w' and 'q' commands to ed scripts
    pub append_wq_to_ed: bool,
    //  -X                          like -x, but bracket conflicts
    pub bracket_conflicts: bool,
    /// --diff-program=PROGRAM  use PROGRAM to compare files
    pub diff_program: Option<String>,
    /// -3, --easy-only             like -e, but incorporate only non-overlapping changes
    pub easy_only: bool,
    /// -e, --ed                    output ed script incorporating changes
    pub ed: bool,
    /// --help                  display this help and exit
    pub help: bool,
    /// -T, --initial-tab           make tabs line up by prepending a tab
    pub initial_tab: bool,
    /// -L, --label=LABEL           use LABEL instead of file name
    pub label: Option<String>,
    /// -m, --merge                 output actual merged file, according to
    pub merge: bool,
    /// -x, --overlap-only          like -e, but incorporate only overlapping changes
    pub overlap_only: bool,
    /// -A, --show-all              output all changes, bracketing conflicts
    pub show_all: bool,
    /// -E, --show-overlap          like -e, but bracket conflicts
    pub show_overlap: bool,
    /// --strip-trailing-cr     strip trailing carriage return on input
    pub strip_trailing_cr: bool,
    /// -a, --text                  treat all files as text
    pub text: bool,
    /// -v, --version               output version information and exit
    pub version: bool,
}

// TODO default
// TODO rustanalyzer issue: no entry or missing entry: create defaults
impl Default for ParamsDiff3 {
    fn default() -> Self {
        Self {
            executable: Executable::Diff3,
            from: Default::default(),
            to: Default::default(),
            append_wq_to_ed: Default::default(),
            bracket_conflicts: Default::default(),
            diff_program: Default::default(),
            easy_only: Default::default(),
            ed: Default::default(),
            help: Default::default(),
            initial_tab: Default::default(),
            label: Default::default(),
            merge: Default::default(),
            overlap_only: Default::default(),
            show_all: Default::default(),
            show_overlap: Default::default(),
            strip_trailing_cr: Default::default(),
            text: Default::default(),
            version: Default::default(),
        }
    }
}

impl ParamsDiff3 {
    /// Parses the program arguments.
    ///
    /// First argument is expected to be the executable.
    pub fn parse_params<I: Iterator<Item = OsString>>(
        executable: &Executable,
        args: Peekable<I>,
    ) -> ResultSdiffParse {
        let parser = Parser::parse_params(&APP_OPTIONS, args)?;

        // check implemented options
        #[cfg(feature = "feat_check_not_yet_implemented")]
        {
            crate::arg_parser::is_implemented(&parser.options_parsed, &NOT_YET_IMPLEMENTED)?;
        }

        let mut params = Self {
            executable: executable.clone(),
            ..Default::default()
        };

        // set options
        for parsed_option in &parser.options_parsed {
            // dbg!(parsed_option);
            match *parsed_option.app_option {
                OPT_APPEND_WQ_TO_ED => params.append_wq_to_ed = true,
                OPT_BRACKET_CONFLICTS => params.bracket_conflicts = true,
                OPT_DIFF_PROGRAM => params.diff_program = parsed_option.arg_for_option.clone(),
                OPT_EASY_ONLY => params.easy_only = true,
                OPT_ED => params.ed = true,
                OPT_HELP => return Ok(Diff3ParseOk::Help),
                OPT_INITIAL_TAB => params.initial_tab = true,
                OPT_LABEL => params.label = parsed_option.arg_for_option.clone(),
                OPT_MERGE => params.merge = true,
                OPT_OVERLAP_ONLY => params.overlap_only = true,
                OPT_SHOW_ALL => params.show_all = true,
                OPT_SHOW_OVERLAP => params.show_overlap = true,
                OPT_STRIP_TRAILING_CR => params.strip_trailing_cr = true,
                OPT_TEXT => params.text = true,
                OPT_VERSION => return Ok(Diff3ParseOk::Version),

                // This is not an error, but a todo. Unfortunately an Enum is not possible.
                _ => todo!("Err Option: {}", parsed_option.app_option.long_name),
            }
        }

        // set operands
        match parser.operands.len() {
            0 => return Err(ParseError::NoOperands(executable.clone())),
            // If only file_1 is set, then file_2 defaults to '-', so it reads from StandardInput.
            1 => {
                params.from = parser.operands[0].clone();
                params.to = OsString::from("-");
            }
            2 => {
                params.from = parser.operands[0].clone();
                params.to = parser.operands[1].clone();
            }
            _ => {
                return Err(ParseError::ExtraOperand(parser.operands[2].clone()));
            }
        }

        // dbg!(&params);
        Ok(Diff3ParseOk::Params(params))
    }
}

// // Usually assert is used like assert_eq(test result, expected result).
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     fn os(s: &str) -> OsString {
//         OsString::from(s)
//     }
//
//     /// Simplify call of parser, just pass a normal string like in the Terminal.
//     fn parse(args: &str) -> ResultSdiffParse {
//         let mut o = Vec::new();
//         for arg in args.split(' ') {
//             o.push(os(arg));
//         }
//         let mut p = o.into_iter().peekable();
//         // remove executable
//         let executable = Executable::from_args_os(&mut p, true).unwrap();
//
//         ParamsDiff3::parse_params(&executable, p)
//     }
//
//     fn res_ok(params: ParamsDiff3) -> ResultSdiffParse {
//         Ok(Diff3ParseOk::Params(params))
//     }
//
//     #[test]
//     fn positional() {
//         // file_1 and file_2 given
//         assert_eq!(
//             parse("diff3 foo bar"),
//             res_ok(ParamsDiff3 {
//                 executable: Executable::Diff3,
//                 from: os("foo"),
//                 to: os("bar"),
//                 ..Default::default()
//             }),
//         );
//
//         // file_1 only
//         assert_eq!(
//             parse("diff3 foo"),
//             res_ok(ParamsDiff3 {
//                 executable: Executable::Diff3,
//                 from: os("foo"),
//                 to: os("-"),
//                 ..Default::default()
//             }),
//         );
//
//         // double dash without operand
//         assert_eq!(
//             parse("diff3 foo -- --help"),
//             res_ok(ParamsDiff3 {
//                 executable: Executable::Diff3,
//                 from: os("foo"),
//                 to: os("--help"),
//                 ..Default::default()
//             }),
//         );
//
//         // Err: no arguments
//         let msg = "missing operand after 'diff3'";
//         match parse("diff3") {
//             Ok(_) => assert!(false, "Should not be ok!"),
//             Err(e) => assert!(
//                 e.to_string().contains(msg),
//                 "error must contain: \"{msg}\"\nactual error: \"{e}\""
//             ),
//         }
//
//         // Err: too many operands
//         let msg = "extra operand 'should-not-be-here'";
//         match parse("diff3 foo bar should-not-be-here") {
//             Ok(_) => assert!(false, "Should not be ok!"),
//             Err(e) => assert!(
//                 e.to_string().contains(msg),
//                 "error must contain: \"{msg}\"\nactual error: \"{e}\""
//             ),
//         }
//     }
//
//     #[test]
//     fn execution_modes() {
//         // Test all options
//         // Disable feature "feat_check_not_yet_implemented"
//         // I^A is at the end of the single options, forcing '^A' as argument for 'I'.
//         // --wi is abbreviated and uses equal sign
//         // diff-program uses next arg
//         // -O uses next arg
//         let params = ParamsDiff3 {
//             executable: Executable::Diff3,
//             from: os("foo"),
//             to: os("bar"),
//             diff_program: Some("prg".to_string()),
//             expand_tabs: true,
//             help: false,
//             ignore_all_space: true,
//             ignore_blank_lines: true,
//             ignore_case: true,
//             ignore_matching_lines: Some("^A".to_string()),
//             ignore_space_change: true,
//             ignore_tab_expansion: true,
//             ignore_trailing_space: true,
//             left_column: true,
//             minimal: true,
//             output: Some("out".to_string()),
//             speed_large_files: true,
//             strip_trailing_cr: true,
//             suppress_common_lines: true,
//             tabsize: 2,
//             text: true,
//             version: false,
//             width: 150,
//         };
//         let r = parse(
//             "diff3 foo bar -iEZbWBalstdHI^A --wi=150 --diff-program prg -o out --strip --tab=2",
//         );
//         match &r {
//             Ok(_) => assert_eq!(r, res_ok(params.clone())),
//             Err(e) => match e {
//                 ParseError::NotYetImplemented(_) => {}
//                 _ => assert_eq!(r, res_ok(params.clone())),
//             },
//         }
//
//         // negative value
//         // let msg = "invalid argument '-2' for '--tabsize'";
//         let msg = "invalid --tabsize value '-2'";
//         let r = parse("diff3 foo bar --tab=-2");
//         match r {
//             Ok(_) => assert!(false, "Should not be Ok."),
//             Err(e) => assert!(
//                 e.to_string().contains(msg),
//                 "Must contain: {msg}\nactual: {e}"
//             ),
//         }
//     }
// }
