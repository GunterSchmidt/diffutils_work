//! This module holds all definitions, text and error messages for sdiff.

use std::fmt::Display;

use crate::{
    arg_parser::{AppOption, ArgParserError, ParseBytesError, ParsedOption, OPT_HELP, OPT_VERSION},
    cmp::{params_cmp::ParamsCmp, EXE_NAME},
};

// TODO Help text
pub const TEXT_HELP: &str = r#"
        Usage: {} [OPTION]... FILE1 [FILE2 [SKIP1 [SKIP2]]]
        Compare two files byte by byte.

        The optional SKIP1 and SKIP2 specify the number of bytes to skip
        at the beginning of each file (zero by default).

        Mandatory arguments to long options are mandatory for short options too.
          -b, --print-bytes          print differing bytes
          -i, --ignore-initial=SKIP         skip first SKIP bytes of both inputs
          -i, --ignore-initial=SKIP1:SKIP2  skip first SKIP1 bytes of FILE1 and
                                              first SKIP2 bytes of FILE2
          -l, --verbose              output byte numbers and differing byte values
          -n, --bytes=LIMIT          compare at most LIMIT bytes
          -s, --quiet, --silent      suppress all normal output
              --help                 display this help and exit
          -v, --version              output version information and exit

        SKIP values may be followed by the following multiplicative suffixes:
        kB 1000, K 1024, MB 1,000,000, M 1,048,576,
        GB 1,000,000,000, G 1,073,741,824, and so on for T, P, E, Z, Y.

        If a FILE is '-' or missing, read standard input.
        Exit status is 0 if inputs are the same, 1 if different, 2 if trouble.

        This utility is part of the uutils project: https://github.com/uutils/
        Report bugs here: https://github.com/uutils/diffutils/issues
    "},
        params.executable.to_string_lossy()
    );"#;

// TODO Version text
pub const TEXT_VERSION: &str = concat!("cmp (Rust DiffUtils) ", env!("CARGO_PKG_VERSION"),);

// -b, --print-bytes          print differing bytes
// -i, --ignore-initial=SKIP         skip first SKIP bytes of both inputs
// -i, --ignore-initial=SKIP1:SKIP2  skip first SKIP1 bytes of FILE1 and
//                                     first SKIP2 bytes of FILE2
// -l, --verbose              output byte numbers and differing byte values
// -n, --bytes=LIMIT          compare at most LIMIT bytes
// -s, --quiet, --silent      suppress all normal output
//     --help                 display this help and exit
// -v, --version              output version information and exit
//   -b, --print-bytes          print differing bytes
//   -i, --ignore-initial=SKIP         skip first SKIP bytes of both inputs
//   -i, --ignore-initial=SKIP1:SKIP2  skip first SKIP1 bytes of FILE1 and
//   -l, --verbose              output byte numbers and differing byte values
//   -n, --bytes=LIMIT          compare at most LIMIT bytes
//   -s, --quiet, --silent      suppress all normal output
//       --help                 display this help and exit
//   -v, --version              output version information and exit
pub(super) const OPT_BYTES_LIMIT: AppOption = AppOption {
    long_name: "bytes",
    short: Some('n'),
    has_arg: true,
};
pub(super) const OPT_IGNORE_INITIAL: AppOption = AppOption {
    long_name: "ignore-initial",
    short: Some('i'),
    has_arg: true,
};
pub(super) const OPT_PRINT_BYTES: AppOption = AppOption {
    long_name: "print-bytes",
    short: Some('b'),
    has_arg: false,
};
pub(super) const OPT_QUIET: AppOption = AppOption {
    long_name: "quiet",
    short: Some('s'),
    has_arg: false,
};
pub(super) const OPT_SILENT: AppOption = AppOption {
    long_name: "silent",
    short: Some('s'),
    has_arg: false,
};
pub(super) const OPT_VERBOSE: AppOption = AppOption {
    long_name: "verbose",
    short: Some('l'),
    has_arg: false,
};

// must contain OPT_HELP,and OPT_VERSION
pub(super) const APP_OPTIONS: [AppOption; 8] = [
    OPT_BYTES_LIMIT,
    OPT_IGNORE_INITIAL,
    OPT_PRINT_BYTES,
    OPT_QUIET,
    OPT_SILENT,
    OPT_VERBOSE,
    OPT_HELP,
    OPT_VERSION,
];

/// Success return type for parsing of params.
///
/// Successful parsing will return ParamsCmp, \
/// -- help und --version will return an Info message, \
/// Error will be returned as [ParamsCmpError] in the function Result.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamsCmpOk {
    Info(String),
    ParamsCmp(ParamsCmp),
}

/// Contains all parser errors and their text messages.
#[derive(Debug, PartialEq)]
pub enum ParamsCmpError {
    /// Bubbled up error
    ArgParserError(ArgParserError),

    /// bytes number incorrect
    BytesInvalidNumber(ParsedOption),

    /// bytes unit incorrect, e.g. 1000LB
    BytesInvalidUnit(ParsedOption),

    /// bytes number too large (>u64)
    BytesPosOverflow(ParsedOption),

    /// Having 5 operands or more
    /// (wrong operand)
    ExtraOperand(String),

    // /// Ignore Initial is given as extra operand and also as option. \
    // /// This is not an original GNU cmp error message, where the operands are ignored.
    // /// (3rd operand, 4th operand, --i value).
    // IgnoreInitialDouble(String, String, String),
    SilentPrintBytesIncompatible,
    SilentVerboseIncompatible,
}

impl ParamsCmpError {
    pub fn from_parse_byte_error(
        parse_byte_error: ParseBytesError,
        parsed_option: &ParsedOption,
        // app_option: &AppOption,
        // name_type_used: OptionNameTypeUsed,
    ) -> Self {
        match parse_byte_error {
            ParseBytesError::NoValue => {
                Self::ArgParserError(ArgParserError::ArgForOptionMissing(parsed_option.clone()))
            }
            ParseBytesError::PosOverflow => {
                Self::BytesPosOverflow(parsed_option.clone())
                // Self::ParseGenError(ParamsGenParseError::ArgForOptionMissing(
                //     parsed_option.app_option,
                //     parsed_option.name_type_used,
                // ))
            }
            ParseBytesError::InvalidNumber => Self::BytesInvalidNumber(parsed_option.clone()),
            ParseBytesError::InvalidUnit => Self::BytesInvalidUnit(parsed_option.clone()),
        }
    }
}

impl From<ArgParserError> for ParamsCmpError {
    fn from(err: ArgParserError) -> Self {
        Self::ArgParserError(err)
    }
}

impl Display for ParamsCmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Writes the error message, adds cmp: and the --help information.
        fn write_err(f: &mut std::fmt::Formatter<'_>, msg: &str) -> Result<(), std::fmt::Error> {
            ArgParserError::write_err(f, EXE_NAME, msg)
        }

        // TODO Short and Long name errors
        match self {
            ParamsCmpError::ArgParserError(e) => write_err(f, &e.to_string()),

            ParamsCmpError::BytesInvalidNumber(opt) | ParamsCmpError::BytesInvalidUnit(opt) => {
                write_err(
                    f,
                    &format!(
                        "invalid '--{}' value '{}'",
                        opt.app_option.long_name,
                        opt.arg_for_option
                            .as_ref()
                            .expect("Logic error, number must be given.")
                    ),
                )
            }
            ParamsCmpError::BytesPosOverflow(opt) => write_err(
                f,
                &format!(
                    "invalid '--{}' value (too large) '{}'",
                    opt.app_option.long_name,
                    opt.arg_for_option
                        .as_ref()
                        .expect("Logic error, number must be given.")
                ),
            ),
            ParamsCmpError::ExtraOperand(opt) => write_err(f, &format!("extra operand '{opt}'")),
            ParamsCmpError::SilentPrintBytesIncompatible => write_err(
                f,
                "options '--print-bytes' ('-b') and '--silent' ('-s') are incompatible",
            ),
            ParamsCmpError::SilentVerboseIncompatible => write_err(
                f,
                "options '--verbose' ('-l') and '--silent' ('-s') are incompatible",
            ),
        }
    }
}
