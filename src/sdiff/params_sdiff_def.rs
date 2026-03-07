// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

//! This module holds all definitions, text and error messages for sdiff.
use std::fmt::Display;

use crate::{
    arg_parser::{self, AppOption, Executable, ParseError, ParsedOption, OPT_HELP, OPT_VERSION},
    sdiff::params_sdiff::ParamsSDiff,
};

pub type ResultSdiffParse = Result<SDiffParseOk, ParseError>;

// AppOptions for sdiff
pub const OPT_DIFF_PROGRAM: AppOption = AppOption {
    long_name: "diff-program",
    short: None,
    has_arg: true,
};
pub const OPT_EXPAND_TABS: AppOption = AppOption {
    long_name: "expand-tabs",
    short: Some('t'),
    has_arg: false,
};
pub const OPT_IGNORE_ALL_SPACE: AppOption = AppOption {
    long_name: "ignore-all-space",
    short: Some('W'),
    has_arg: false,
};
pub const OPT_IGNORE_BLANK_LINES: AppOption = AppOption {
    long_name: "ignore-blank-lines",
    short: Some('B'),
    has_arg: false,
};
pub const OPT_IGNORE_CASE: AppOption = AppOption {
    long_name: "ignore-case",
    short: Some('i'),
    has_arg: false,
};
pub const OPT_IGNORE_MATCHING_LINES: AppOption = AppOption {
    long_name: "ignore-matching-lines",
    short: Some('I'),
    has_arg: true,
};
pub const OPT_IGNORE_SPACE_CHANGE: AppOption = AppOption {
    long_name: "ignore-space-change",
    short: Some('b'),
    has_arg: false,
};
pub const OPT_IGNORE_TAB_EXPANSION: AppOption = AppOption {
    long_name: "ignore-tab-expansion",
    short: Some('E'),
    has_arg: false,
};
pub const OPT_IGNORE_TRAILING_SPACE: AppOption = AppOption {
    long_name: "ignore-trailing-space",
    short: Some('Z'),
    has_arg: false,
};
pub const OPT_LEFT_COLUMN: AppOption = AppOption {
    long_name: "left-column",
    short: Some('l'),
    has_arg: false,
};
pub const OPT_MINIMAL: AppOption = AppOption {
    long_name: "minimal",
    short: Some('d'),
    has_arg: false,
};
pub const OPT_OUTPUT: AppOption = AppOption {
    long_name: "output",
    short: Some('o'),
    has_arg: true,
};
pub const OPT_SPEED_LARGE_FILES: AppOption = AppOption {
    long_name: "speed-large-files",
    short: Some('H'),
    has_arg: false,
};
pub const OPT_STRIP_TRAILING_CR: AppOption = AppOption {
    long_name: "strip-trailing-cr",
    short: None,
    has_arg: false,
};
pub const OPT_SUPPRESS_COMMON_LINES: AppOption = AppOption {
    long_name: "suppress-common-lines",
    short: Some('s'),
    has_arg: false,
};
pub const OPT_TABSIZE: AppOption = AppOption {
    long_name: "tabsize",
    short: None,
    has_arg: true,
};
pub const OPT_TEXT: AppOption = AppOption {
    long_name: "text",
    short: Some('a'),
    has_arg: false,
};
pub const OPT_WIDTH: AppOption = AppOption {
    long_name: "width",
    short: Some('w'),
    has_arg: true,
};

// Array for ParamsGen
pub const ARG_OPTIONS: [AppOption; 20] = [
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

/// Success return type for parsing of params.
///
/// Successful parsing will return ParamsSdiff, \
/// '-- help' und '--version' will return the text as Enum value, \
/// Error will be returned as [ParamsSdiffError] in the function Result.
#[derive(Debug, PartialEq)]
pub enum SDiffParseOk {
    ParamsSdiff(ParamsSDiff),
    Help,
    Version,
    // Info(String),
}

/// Contains all parser errors and their text messages.
/// This allows centralized maintenance.
#[derive(Debug, PartialEq)]
pub struct ParamsSdiffErrorCtx {
    pub util: Executable,
    pub error: SDiffParseError,
}

impl std::error::Error for ParamsSdiffErrorCtx {}

// impl From<ArgParserErrorCtx> for ParamsSdiffErrorCtx {
//     fn from(e: ArgParserErrorCtx) -> Self {
//         Self {
//             util: e.util.clone(),
//             error: ParamsSdiffError::ArgParserError(e),
//         }
//     }
// }

impl Display for ParamsSdiffErrorCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        arg_parser::write_err(f, &self.util, &self.error)
    }
}

/// Contains all parser errors and their text messages.
/// This allows centralized maintenance.
#[derive(Debug, PartialEq)]
pub enum SDiffParseError {
    /// Bubbled up error
    // ArgParserError(ArgParserErrorCtx),
    ParserError(ParseError),

    /// number for an option argument incorrect
    InvalidNumber(ParsedOption),

    // (param argument)
    // WidthInvalid(String),
    /// Having 3 operands or more
    /// (wrong operand)
    ExtraOperand(String),
}

impl std::error::Error for SDiffParseError {}

// impl From<ArgParserErrorCtx> for ParamsSdiffError {
//     fn from(err: ArgParserErrorCtx) -> Self {
//         Self::ArgParserError(err)
//     }
// }
impl From<ParseError> for SDiffParseError {
    fn from(e: ParseError) -> Self {
        Self::ParserError(e)
    }
}

impl Display for SDiffParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Generally error messages do not attempt to be GNU compatible.
        let msg = match self {
            // ParamsSdiffError::ArgParserError(e) => &e.to_string(),
            SDiffParseError::ParserError(e) => &e.to_string(),
            SDiffParseError::ExtraOperand(opt) => &format!("extra operand '{opt}'"),
            SDiffParseError::InvalidNumber(opt) => &format!(
                "invalid argument '{}' for '--{}'{}",
                opt.arg_for_option_or_empty_string(),
                opt.app_option.long_name,
                opt.short_char_or_empty_string(),
            ),
            // ParamsSdiffError::WidthInvalid(param) => write_err(f, &format!("invalid '{param}'")),
        };
        write!(f, "{msg}")
    }
}
