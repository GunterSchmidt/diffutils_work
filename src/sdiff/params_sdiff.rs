//! This module contains the Parser for sdiff arguments.
//!
//! All option definitions, output texts and the Error handling is in [super::params_sdiff_def].
use std::{ffi::OsString, iter::Peekable};

use crate::{
    arg_parser::{ArgParser, ArgParserError, DiffUtility, ParsedOption, OPT_HELP, OPT_VERSION},
    sdiff::params_sdiff_def::*,
};

/// Holds the given command line arguments except "--version" and "--help".
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct ParamsSdiff {
    /// Identifier
    pub util: DiffUtility,
    pub from: OsString,
    pub to: OsString,
    /// --diff-program=PROGRAM   use PROGRAM to compare files
    diff_program: Option<String>,
    /// -t, --expand-tabs            expand tabs to spaces in output
    expand_tabs: bool,
    /// --help                   display this help and exit
    help: bool,
    /// -W, --ignore-all-space       ignore all white space
    ignore_all_space: bool,
    /// -B, --ignore-blank-lines     ignore changes whose lines are all blank
    ignore_blank_lines: bool,
    /// -i, --ignore-case            consider upper- and lower-case to be the same
    ignore_case: bool,
    /// -I, --ignore-matching-lines=REGEXP  ignore changes all whose lines match REGEXP
    ignore_matching_lines: Option<String>,
    /// -b, --ignore-space-change    ignore changes in the amount of white space
    ignore_space_change: bool,
    /// -E, --ignore-tab-expansion   ignore changes due to tab expansion
    ignore_tab_expansion: bool,
    /// -Z, --ignore-trailing-space  ignore white space at line end
    ignore_trailing_space: bool,
    /// -l, --left-column            output only the left column of common lines
    left_column: bool,
    /// -d, --minimal                try hard to find a smaller set of changes
    minimal: bool,
    /// -o, --output=FILE            operate interactively, sending output to FILE
    output: Option<String>,
    /// -H, --speed-large-files      assume large files, many scattered small changes
    speed_large_files: bool,
    /// --strip-trailing-cr      strip trailing carriage return on input
    strip_trailing_cr: bool,
    /// -s, --suppress-common-lines  do not output common lines
    suppress_common_lines: bool,
    /// --tabsize=NUM            tab stops at every NUM (default 8) print columns
    tabsize: Option<usize>,
    /// -a, --text                   treat all files as text
    text: bool,
    /// -v, --version                output version information and exit
    version: bool,
    /// -w, --width=NUM              output at most NUM (default 130) print columns
    width: Option<usize>,
}

impl ParamsSdiff {
    pub fn parse_params<I: Iterator<Item = OsString>>(opts: Peekable<I>) -> ResultParamsSdiffParse {
        let p_gen = ArgParser::parse_params(&ARG_OPTIONS, opts)?;
        Self::try_from(&p_gen)
    }

    fn try_from(parser: &ArgParser) -> ResultParamsSdiffParse {
        let mut params = Self {
            util: DiffUtility::SDiff,
            ..Default::default()
        };

        // set options
        for parsed_option in &parser.options_parsed {
            dbg!(parsed_option);
            match *parsed_option.app_option {
                OPT_DIFF_PROGRAM => params.diff_program = parsed_option.arg_for_option.clone(),
                OPT_EXPAND_TABS => params.expand_tabs = true,
                OPT_HELP => return Ok(ParamsSdiffOk::Info(ParamsSdiffInfo::Help)),
                OPT_IGNORE_ALL_SPACE => params.ignore_all_space = true,
                OPT_IGNORE_BLANK_LINES => params.ignore_blank_lines = true,
                OPT_IGNORE_CASE => params.ignore_case = true,
                OPT_IGNORE_MATCHING_LINES => {
                    params.ignore_matching_lines = parsed_option.arg_for_option.clone()
                }
                OPT_IGNORE_SPACE_CHANGE => params.ignore_space_change = true,
                OPT_IGNORE_TAB_EXPANSION => params.ignore_tab_expansion = true,
                OPT_IGNORE_TRAILING_SPACE => params.ignore_trailing_space = true,
                OPT_LEFT_COLUMN => params.left_column = true,
                OPT_MINIMAL => params.minimal = true,
                OPT_OUTPUT => params.output = parsed_option.arg_for_option.clone(),
                OPT_SPEED_LARGE_FILES => params.speed_large_files = true,
                OPT_STRIP_TRAILING_CR => params.strip_trailing_cr = true,
                OPT_SUPPRESS_COMMON_LINES => params.suppress_common_lines = true,
                OPT_TABSIZE => {
                    params.set_tabsize(parsed_option)?;
                }
                OPT_TEXT => params.text = true,
                OPT_VERSION => return Ok(ParamsSdiffOk::Info(ParamsSdiffInfo::Version)),
                OPT_WIDTH => {
                    params.set_width(parsed_option)?;
                }

                // This is not an error, but a todo. Unfortunately an Enum is not possible.
                _ => todo!("Err Option: {}", parsed_option.app_option.long_name),
            }
        }

        // set operands
        match parser.operands.len() {
            0 => {
                return Err(ParamsSdiffError::ArgParserError(ArgParserError::NoOperand(
                    params.util,
                )))
            }
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
                return Err(ParamsSdiffError::ExtraOperand(
                    parser.operands[2].to_string_lossy().to_string(),
                ));
            }
        }

        // // Do as GNU sdiff, and completely disable printing if we are
        // // outputting to /dev/null.
        // #[cfg(not(target_os = "windows"))]
        // if crate::sdiff::is_stdout_dev_null() {
        //     params.silent = true;
        //     params.verbose = false;
        //     params.print_bytes = false;
        // }

        // dbg!(&params);
        Ok(ParamsSdiffOk::ParamsSdiff(params))
    }

    pub fn set_tabsize(&mut self, parsed_option: &ParsedOption) -> Result<usize, ParamsSdiffError> {
        let tab_size = parsed_option.arg_for_option.clone().unwrap_or_default();
        let t = match tab_size.parse::<usize>() {
            Ok(w) => w,
            Err(_) => return Err(ParamsSdiffError::InvalidNumber(parsed_option.clone())),
        };
        self.tabsize = Some(t);

        Ok(t)
    }

    pub fn set_width(&mut self, parsed_option: &ParsedOption) -> Result<usize, ParamsSdiffError> {
        let width = parsed_option.arg_for_option.clone().unwrap_or_default();
        let w = match width.parse::<usize>() {
            Ok(w) => w,
            Err(_) => return Err(ParamsSdiffError::InvalidNumber(parsed_option.clone())),
        };
        self.width = Some(w);

        Ok(w)
    }
}

// Usually assert is used like assert_eq(test result, expected result).
#[cfg(test)]
mod tests {
    use super::*;
    // use crate::arg_parser::OPT_VERSION;

    pub const TEXT_HELP_HINT: &str = "Try 'sdiff --help' for more information.";

    fn os(s: &str) -> OsString {
        OsString::from(s)
    }

    /// Simplify call of parser, just pass a normal string like in the Terminal.
    fn parse(args: &str) -> ResultParamsSdiffParse {
        let mut o = Vec::new();
        for arg in args.split(' ') {
            o.push(os(arg));
        }
        let p = o.into_iter().peekable();

        ParamsSdiff::parse_params(p)
    }

    fn res_ok(params: ParamsSdiff) -> ResultParamsSdiffParse {
        Ok(ParamsSdiffOk::ParamsSdiff(params))
    }

    #[test]
    fn positional() {
        // file_1 and file_2 given
        assert_eq!(
            parse("sdiff foo bar"),
            res_ok(ParamsSdiff {
                util: DiffUtility::SDiff,
                from: os("foo"),
                to: os("bar"),
                ..Default::default()
            }),
        );

        // file_1 only
        assert_eq!(
            parse("sdiff foo"),
            res_ok(ParamsSdiff {
                util: DiffUtility::SDiff,
                from: os("foo"),
                to: os("-"),
                ..Default::default()
            }),
        );

        // double dash without operand
        // Test fails as this behavior is not replicated.
        // assert_eq!(
        //     parse_params("sdiff foo -- --help"),
        //     res_ok(ParamsSdiff {
        //         util: DiffUtility::SDiff,
        //         file_1: os("foo"),
        //         file_2: os("--help"),
        //         ..Default::default()
        //     }),
        // );

        // Err: too many operands
        assert_eq!(
            parse("sdiff foo bar extra"),
            Err(ParamsSdiffError::ExtraOperand("extra".to_string())),
        );

        // Err: no arguments
        assert_eq!(
            parse("sdiff"),
            Err(ParamsSdiffError::ArgParserError(ArgParserError::NoOperand(
                DiffUtility::SDiff
            )))
        );
    }

    #[test]
    fn execution_modes() {
        // Test all options
        // I^A is at the end of the single options, forcing '^A' as argument for 'I'.
        // --wi is abbreviated and uses equal sign
        // diff-program uses next arg
        // -O uses next arg
        let params = ParamsSdiff {
            util: DiffUtility::SDiff,
            from: os("foo"),
            to: os("bar"),
            diff_program: Some("prg".to_string()),
            expand_tabs: true,
            help: false,
            ignore_all_space: true,
            ignore_blank_lines: true,
            ignore_case: true,
            ignore_matching_lines: Some("^A".to_string()),
            ignore_space_change: true,
            ignore_tab_expansion: true,
            ignore_trailing_space: true,
            left_column: true,
            minimal: true,
            output: Some("out".to_string()),
            speed_large_files: true,
            strip_trailing_cr: true,
            suppress_common_lines: true,
            tabsize: Some(2),
            text: true,
            version: false,
            width: Some(150),
        };
        assert_eq!(
            parse(
                "sdiff foo bar -iEZbWBalstdHI^A --wi=150 --diff-program prg -o out --strip --tab=2"
            ),
            res_ok(params.clone())
        );

        // negative value
        let r = parse("sdiff foo bar --tab=-2");
        match r {
            Ok(_) => assert!(false, "Should not be Ok."),
            Err(e) => assert_eq!(
                e.to_string(),
                format!("sdiff: invalid argument '-2' for '--tabsize'\nsdiff: {TEXT_HELP_HINT}")
            ),
        }
    }
}
