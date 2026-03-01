#![allow(unused)]
use std::{ffi::OsString, fmt::Display, iter::Peekable};

use crate::cmp::{params_cmp::ParamsCmp, Bytes, IgnInit};

pub type ResultParamsParse = Result<ParamsParseOk, ParamsParseError>;
type ResultBytesParse = Result<Bytes, ParseBytesError>;

// TODO remove, replace with text, easier to read
// or filter to enums, which should be faster
const OPT_PRINT_BYTES: &str = "print-bytes";
const OPT_IGNORE_INITIAL: &str = "ignore-initial";
const OPT_VERBOSE: &str = "verbose";
const OPT_BYTES: &str = "bytes";
const OPT_QUIET: &str = "quiet";
const OPT_SILENT: &str = "silent";
const OPT_HELP: &str = "help";
const OPT_VERSION: &str = "version";
const OPTIONS: [&str; 8] = [
    OPT_BYTES,
    OPT_HELP,
    OPT_IGNORE_INITIAL,
    OPT_PRINT_BYTES,
    OPT_QUIET,
    OPT_SILENT,
    OPT_VERBOSE,
    OPT_VERSION,
];

// TODO Version text
pub const TEXT_VERSION: &str = concat!(
    "cmp (Rust diffutils) ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    r#"Copyright (C) 2026 <TODO>?
Licenses: MIT License, Apache License 2.0 <https://www.apache.org/licenses/LICENSE-2.0>.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

Written by <TODO>."#
);

pub const TEXT_HELP: &str = r#"Usage: cmp [OPTION]... FILE1 [FILE2 [SKIP1 [SKIP2]]]
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

Report bugs by creating an issue.
home page: <https://github.com/uutils/diffutils>
"#;

pub const TEXT_HELP_HINT: &str = "Try 'cmp --help' for more information.";

// pub enum ParamOption {
//     /// Bytes Limit with unparsed number String
//     BytesLimit(String),
//     Help,
//     IgnoreInitial(String, String),
//     PrintBytes,
//     Silent,
//     Verbose,
//     Version,
// }

/// Static texts for --help and --version.
///
/// The parser returns these enums to the caller, allowing the caller can identify this as information,
/// so that the program exit code is SUCCESS(0).
#[derive(Debug, PartialEq)]
pub enum ParamsParseInfo {
    Help,
    Version,
}

impl Display for ParamsParseInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = match self {
            ParamsParseInfo::Help => TEXT_HELP,
            ParamsParseInfo::Version => TEXT_VERSION,
        };

        write!(f, "{}", info)
    }
}

/// Contains all parser errors and their text messages.
///
/// First argument is always the exe name ('cmp'). \
/// TODO Usually this would be hard coded, but for some unknown reason the executable name is variable.
#[derive(Debug, PartialEq)]
pub enum ParamsParseError {
    /// ('cmp', ambiguous option, possible Options)
    AmbiguousOption(String, String, String),
    /// cmp: option '--silent' doesn't allow an argument
    /// ('cmp', Option)
    ArgForOptionNotAllowed(String, String),
    /// ('cmp', BytesType)
    BytesNoValue(String, BytesType),
    /// ('cmp', original bytes option, e.g. 1000LB)
    BytesInvalidNumber(String, BytesType, String),
    /// ('cmp', original bytes option, e.g. 1000LB)
    BytesInvalidUnit(String, BytesType, String),
    BytesPosOverflow(String, BytesType, String),
    /// Having 5 operands or more
    /// ('cmp', wrong operand)
    ExtraOperand(String, String),
    // /// Ignore Initial is given as extra operand and also as option. \
    // /// This is not an original GNU cmp error message, where the operands are ignored.
    // /// ('cmp', 3rd operand, 4th operand, --i value).
    // IgnoreInitialDouble(String, String, String),
    /// Non-existent single dash option.
    /// ('cmp', wrong option)
    InvalidOption(String, String),
    /// cmp as parameter missing. I wonder how this can happen.
    NoExecutable,
    /// cmp, but no other args
    /// ('cmp')
    NoOperand(String),
    // /// Two dashes '--' without option not allowed. GNU cmp has somewhat undefined behavior, this is cleaner.
    // /// ('cmp')
    // OptionUndefined(String),
    /// --silent=50
    /// ('cmp')
    SilentPrintBytesIncompatible(String),
    /// ('cmp')
    SilentVerboseIncompatible(String),
    /// Non-existent double dash option. This is unrecognized because the name can be abbreviated.
    /// ('cmp', wrong option)
    UnrecognizedOption(String, String),
}

impl Display for ParamsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Returns the error message, adds cmp: and the --help information.
        // fn err_message_with_help(cmp: &str, msg: &str) -> String {
        //     format!("{cmp}: {msg}\n{cmp}: {TEXT_HELP_HINT}")
        // }

        fn write_err(
            f: &mut std::fmt::Formatter<'_>,
            cmp: &str,
            msg: &str,
        ) -> Result<(), std::fmt::Error> {
            write!(f, "{cmp}: {msg}\n{cmp}: {TEXT_HELP_HINT}")
        }

        // fn err_msg_invalid_option_short(cmp: &str, option_char: char) -> String {
        //     return err_message_with_help(cmp, &format!("invalid option -- '{}'", option_char));
        // }

        match self {
            ParamsParseError::AmbiguousOption(cmp, param, options) => write_err(
                f,
                cmp,
                &format!("option '{param}' is ambiguous; possibilities: {options}"),
            ),

            ParamsParseError::ArgForOptionNotAllowed(cmp, opt) => {
                write_err(f, cmp, &format!("option '{opt}' doesn't allow an argument"))
            }
            ParamsParseError::BytesNoValue(cmp, t) => {
                write_err(f, cmp, &format!("option {t} requires an argument"))
            }
            ParamsParseError::BytesInvalidNumber(cmp, t, n) => {
                write_err(f, cmp, &format!("invalid {t} value '{n}'"))
            }
            ParamsParseError::BytesInvalidUnit(cmp, t, unit) => {
                write_err(f, cmp, &format!("invalid {t} value '{unit}'"))
            }
            ParamsParseError::BytesPosOverflow(cmp, t, bytes) => {
                write_err(f, cmp, &format!("invalid {t} value (too large) '{bytes}'"))
            }
            ParamsParseError::NoExecutable => write!(
                f,
                "This program must be called with 'cmp' and its parameters."
            ),
            ParamsParseError::ExtraOperand(cmp, opt) => {
                write_err(f, cmp, &format!("extra operand '{opt}'"))
            }
            // ParamsParseError::IgnoreInitialDouble(cmp, op3, ig) => {
            //     write_err(f, cmp, &format!("option '--ignore-initial' ('-i') is set to {ig} but also values ares passed as operand '{op3}'"))
            // }
            ParamsParseError::InvalidOption(cmp, opt) => {
                write_err(f, cmp, &format!("invalid option '{opt}'"))
            }
            ParamsParseError::NoOperand(cmp) => {
                write_err(f, cmp, &format!("missing operand after '{cmp}'"))
            }
            ParamsParseError::UnrecognizedOption(cmp, opt) => {
                write_err(f, cmp, &format!("unrecognized option '{opt}'"))
            }
            ParamsParseError::SilentPrintBytesIncompatible(cmp) => write_err(
                f,
                cmp,
                "options '--print-bytes' ('-b') and '--silent' ('-s') are incompatible",
            ),
            ParamsParseError::SilentVerboseIncompatible(cmp) => write_err(
                f,
                cmp,
                "options '--verbose' ('-l') and '--silent' ('-s') are incompatible",
            ),
        }
    }
}

/// Differentiation for error messages when parsing of a numeric value fails.
#[derive(Debug, PartialEq)]
pub enum BytesType {
    Limit,
    IgnoreInitial,
    // Optional Types if different Error Messages are required depending on short or long option name.
    // LimitChar,
    // LimitName,
    // IgnoreInitialChar,
    // IgnoreInitialName,
}

impl Display for BytesType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opt = match self {
            BytesType::Limit => "'--bytes' ('-n')",
            BytesType::IgnoreInitial => "'--ignore-initial' ('-i')",
        };

        write!(f, "{}", opt)
    }
}

/// Success return type for parsing of params.
///
/// Successful parsing will return Params, \
/// -- help und --version will return an Info message, \
/// Error will be returned as [ParamsParseError] in the function Result.
#[derive(Debug, PartialEq)]
pub enum ParamsParseOk {
    Info(ParamsParseInfo),
    Params(Params),
}

pub enum ParseBytesError {
    NoValue,
    PosOverflow,
    InvalidUnit,
    InvalidNumber,
}

impl ParseBytesError {
    pub fn to_params_parse_error(
        &self,
        executable: String,
        bytes_type: BytesType,
        bytes: String,
    ) -> ParamsParseError {
        match self {
            ParseBytesError::NoValue => ParamsParseError::BytesNoValue(executable, bytes_type),
            ParseBytesError::PosOverflow => {
                ParamsParseError::BytesPosOverflow(executable, bytes_type, bytes)
            }
            ParseBytesError::InvalidNumber => {
                ParamsParseError::BytesInvalidNumber(executable, bytes_type, bytes)
            }
            ParseBytesError::InvalidUnit => {
                ParamsParseError::BytesInvalidUnit(executable, bytes_type, bytes)
            }
        }
    }
}

/// Holds the given command line arguments except "--version" and "--help".
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Params {
    /// Identifier
    pub executable: OsString,
    pub file_1: OsString,
    pub file_2: OsString,
    /// If None will be set to 0.
    pub ignore_initial_bytes_file_1: Option<IgnInit>,
    pub ignore_initial_bytes_file_2: Option<IgnInit>,
    /// cmp from diffutils has a limit of i64::MAX (9_223_372_036_854_775_807)
    /// If None limit will be set to Bytes::MAX.
    pub bytes_limit: Option<Bytes>,
    pub print_bytes: bool,
    // use set_silent
    pub silent: bool,
    // use set_verbose
    pub verbose: bool,
}

impl Params {
    /// Returns the OsString cmp as normal String, should be "cmp".
    pub fn executable_str(&self) -> String {
        self.executable.to_string_lossy().to_string()
    }

    // Returns the next opt as String or an empty String
    pub fn get_next_opt_as_bytes<I: Iterator<Item = OsString>>(opts: &mut Peekable<I>) -> String {
        if let Some(bytes_os) = opts.next() {
            bytes_os.to_string_lossy().to_string()
        } else {
            String::new()
        }
    }

    /// Parses a number as defined in <https://www.gnu.org/software/diffutils/manual/html_node/cmp-Options.html>. \
    /// e.g. 1024 or 1KiB
    pub fn parse_bytes(bytes: &str) -> ResultBytesParse {
        if bytes.is_empty() {
            return Err(ParseBytesError::NoValue);
        }

        let multiplier: Bytes;
        let n = match bytes.find(|b: char| !b.is_ascii_digit()) {
            Some(pos) => {
                if pos == 0 {
                    return Err(ParseBytesError::InvalidNumber);
                }
                multiplier = Self::parse_number_unit(&bytes[pos..])?;
                &bytes[0..pos]
            }
            None => {
                multiplier = 1;
                bytes
            }
        };

        // return value
        match n.parse::<Bytes>() {
            Ok(num) => {
                if multiplier == 1 {
                    Ok(num)
                } else {
                    match num.checked_mul(multiplier) {
                        Some(r) => Ok(r),
                        None => Err(ParseBytesError::PosOverflow),
                    }
                }
            }
            // This is an additional error message not present in GNU cmp.
            Err(e) if *e.kind() == std::num::IntErrorKind::PosOverflow => {
                Err(ParseBytesError::PosOverflow)
            }
            Err(_) => Err(ParseBytesError::InvalidNumber),
        }
    }

    /// Returns a multiplier depending on the given unit, e.g. 'KiB' -> 1024,
    /// which then can be used to calculate the final number of bytes.
    /// Following GNU documentation: https://www.gnu.org/software/diffutils/manual/html_node/cmp-Options.html
    fn parse_number_unit(unit: &str) -> ResultBytesParse {
        // Note that GNU cmp advertises supporting up to Y, but fails if you try
        // to actually use anything beyond E.
        let multiplier = match unit {
            "kB" | "KB" => 1_000,
            "k" | "K" | "KiB" | "kiB" => 1_024,
            "MB" => 1_000_000,
            "M" | "MiB" => 1_048_576,
            "GB" => 1_000_000_000,
            "G" | "GiB" => 1_073_741_824,

            "TB" => 1_000_000_000_000,
            "T" | "TiB" => 1_099_511_627_776,
            "PB" => 1_000_000_000_000_000,
            "P" | "PiB" => 1_125_899_906_842_624,
            "EB" => 1_000_000_000_000_000_000,
            "E" | "EiB" => 1_152_921_504_606_846_976,

            // #[cfg(not(feature = "cmp_bytes_limit_128_bit"))]
            // // Everything above EiB cannot fit into u64.
            // // GNU cmp just returns an invalid bytes value
            // "Z" | "ZB" | "ZiB" | "Y" | "YB" | "YiB" => {
            //     return Err(ParseBytesError::InvalidUnit);
            // }
            // Everything above EiB cannot fit into u64.
            // GNU cmp just returns an invalid bytes value
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "ZB" => 1_000_000_000_000_000_000_000,
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "Z" | "ZIB" => 1_180_591_620_717_411_303_424,
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "YB" => 1_000_000_000_000_000_000_000_000,
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "Y" | "YIB" => 1_208_925_819_614_629_174_706_176,
            _ => {
                return Err(ParseBytesError::InvalidUnit);
            }
        };

        Ok(multiplier)
    }

    /// Sets the --bytes limit and returns the input as number.
    pub fn set_bytes_limit(&mut self, bytes: &str) -> Result<Bytes, ParamsParseError> {
        match Self::parse_bytes(bytes) {
            Ok(r) => {
                self.bytes_limit = Some(r);
                Ok(r)
            }
            Err(e) => match e {
                ParseBytesError::NoValue => Err(ParamsParseError::BytesNoValue(
                    self.executable_str(),
                    BytesType::Limit,
                )),
                ParseBytesError::PosOverflow => Err(ParamsParseError::BytesPosOverflow(
                    self.executable_str(),
                    BytesType::Limit,
                    bytes.to_string(),
                )),
                ParseBytesError::InvalidNumber => Err(ParamsParseError::BytesInvalidNumber(
                    self.executable_str(),
                    BytesType::Limit,
                    bytes.to_string(),
                )),
                ParseBytesError::InvalidUnit => Err(ParamsParseError::BytesInvalidUnit(
                    self.executable_str(),
                    BytesType::Limit,
                    bytes.to_string(),
                )),
            },
        }
    }

    /// Sets the ignore initial bytes for both files.
    ///
    /// Sets the 2nd file to the value of the 1st file if no second parameter is given. \
    /// Returns true if a value for the second file was given.
    fn set_skip_bytes_files(&mut self, bytes: &str) -> Result<bool, ParamsParseError> {
        if bytes.is_empty() {
            return Err(ParamsParseError::BytesNoValue(
                self.executable_str(),
                BytesType::IgnoreInitial,
            ));
        }

        let has_2nd;
        let (skip_1, skip_2) = match bytes.split_once(':') {
            Some((s1, s2)) => {
                has_2nd = true;
                (s1, s2)
            }
            None => {
                has_2nd = false;
                (bytes, bytes)
            }
        };

        self.set_skip_bytes_file_1(skip_1)?;
        self.set_skip_bytes_file_2(skip_2)?;

        Ok(has_2nd)
    }

    /// Sets the [Self::skip_bytes_file_1] value.
    ///
    /// * bytes - A valid number String, e.g. 1800 or 12KiB
    ///
    /// If calling this manually, set_skip_bytes_file_2 to the same value unless
    /// separate values are required.  
    pub fn set_skip_bytes_file_1(&mut self, bytes: &str) -> Result<IgnInit, ParamsParseError> {
        match Self::parse_bytes(bytes) {
            Ok(r) => {
                if r > IgnInit::MAX {
                    Err(ParamsParseError::BytesPosOverflow((), (), ()))
                }
                self.ignore_initial_bytes_file_1 = Some(r);
                Ok(r)
            }
            Err(e) => Err(e.to_params_parse_error(
                self.executable_str(),
                BytesType::IgnoreInitial,
                bytes.to_string(),
            )),
        }
    }

    /// Sets the [Self::skip_bytes_file_2] value.
    ///
    /// * bytes - A valid number String, e.g. 1800 or 12KiB
    pub fn set_skip_bytes_file_2(&mut self, bytes: &str) -> Result<IgnInit, ParamsParseError> {
        match Self::parse_bytes(bytes) {
            Ok(r) => {
                self.ignore_initial_bytes_file_2 = Some(r);
                Ok(r)
            }
            Err(e) => Err(e.to_params_parse_error(
                self.executable_str(),
                BytesType::IgnoreInitial,
                bytes.to_string(),
            )),
        }
    }

    pub fn set_print_bytes(&mut self) -> Result<(), ParamsParseError> {
        // Should actually raise an error if --silent is set, but GNU cmp does not do that.
        if self.silent {
            Err(ParamsParseError::SilentPrintBytesIncompatible(
                self.executable_str(),
            ))
        } else {
            self.print_bytes = true;

            Ok(())
        }
    }

    pub fn set_silent(&mut self) -> Result<(), ParamsParseError> {
        if self.verbose {
            Err(ParamsParseError::SilentVerboseIncompatible(
                self.executable_str(),
            ))
        } else if self.print_bytes {
            Err(ParamsParseError::SilentPrintBytesIncompatible(
                self.executable_str(),
            ))
        } else {
            self.silent = true;

            Ok(())
        }
    }

    pub fn set_verbose(&mut self) -> Result<(), ParamsParseError> {
        if self.silent {
            Err(ParamsParseError::SilentVerboseIncompatible(
                self.executable_str(),
            ))
        } else {
            self.verbose = true;

            Ok(())
        }
    }

    //     pub fn parse_params_defined(&mut self, param: ParamOption) -> Result<(), String> {
    //         // The error messages are not exactly as in GNU cmp.
    //         // Calling gnu cmp with -n without value returns "option requires an argument -- 'n'", which is hard to read.
    //         // Calling gnu cmp with --bytes without value returns "option '--bytes' requires an argument", which is good.
    //         // Calling gnu cmp with -n=9x returns "invalid --bytes value '9x'", which is the same as for --bytes=9x. Notice
    //         // the missing '' around --bytes now.
    //         // Therefore no distinction is made between these two. Otherwise split into BytesLimitShort, BytesLimitLong for
    //         // individual error messages.
    //         match param {
    //             ParamOption::BytesLimit(bytes) => self.set_bytes_limit(&bytes)?,
    //             ParamOption::Help => return Err(TEXT_HELP.to_string()),
    //             ParamOption::PrintBytes => self.print_bytes = true,
    //             ParamOption::Silent => self.set_silent()?,
    //             ParamOption::Verbose => self.set_verbose()?,
    //             ParamOption::Version => return Err(TEXT_VERSION.to_string()),
    //         }
    //
    //         Ok(())
    //     }

    fn identify_options_from_partial_text(opt: &str) -> Vec<&str> {
        let l = opt.len();
        let v: Vec<&str> = OPTIONS
            .iter()
            .filter(|&it| it.len() >= l && &it[0..l] == opt)
            .copied()
            .collect();

        v
    }

    /// Parses the command line arguments. \
    /// Since cmp is called from diffutils, the first argument must always be "cmp".
    ///
    /// The following checks require more extensive checks than a simple compare.
    /// These are all identical and make parsing extensive:
    /// - cmp file_1 file_2 -b -l -n 50
    /// - cmp file_1 file_2 -b -l -n50
    /// - cmp file_1 file_2 -bl -n50
    /// - cmp file_1 file_2 -bln 50
    /// - cmp file_1 file_2 -bln50
    /// - cmp file_1 file_2 --print-bytes --verbose --bytes 50
    /// - cmp file_1 file_2 --print-bytes --verbose --bytes=50
    /// - cmp file_1 file_2 --p --verb --by 50
    /// - cmp file_1 file_2 --p --verb --by=50
    ///
    ///
    /// Other rules:
    /// - parts of correct long options work, too: ignore-initial can be abbreviated, e.g. --i, --ig, --igno, --ignore-init
    /// - -l -s are incompatible
    /// - -b -s are incompatible, but give no error in GNU cmp
    /// - --version or --help will disregard any other params
    /// - -lbv still outputs --version
    /// - -bytes will output y is invalid option (only one dash)
    /// - cmp -- file_1 --help will classify --help as file_2. \
    ///   This may be an issue that the arg parsers of C and Rust work differently. \
    ///   In this case an error message will be thrown in this Rust cmp version.
    ///
    /// For --bytes and ignore-initial a byte value can be given. \
    /// These are all identical:
    /// - cmp file_1 file_2 -bl -n 1024
    /// - cmp file_1 file_2 -bl -n 1k
    /// - cmp file_1 file_2 -bl -n 1K
    /// - cmp file_1 file_2 -bl -n 1KiB
    /// - cmp file_1 file_2 -bl -n 1kiB
    /// - cmp file_1 file_2 -bl -n1kiB
    /// - cmp file_1 file_2 -bln1kiB
    ///
    /// '--ignore-initial' has some perks as the value can be given as operand and as option.
    /// - set 10 10
    ///   - cmp file_1 file_2 -i10:10
    ///   - cmp file_1 file_2 10 10
    ///   - cmp file_1 file_2 -i10, which translates to 'cmp file_1 file_2 i10:10'
    ///   - but not 'cmp file_1 file_2 10', which translates to 'cmp file_1 file_2 10'
    ///   - cmp file_1 file_2 99 -i10 (-i overrides operand)
    ///   - cmp file_1 file_2 99 20 -i10:10 (-i overrides both operands)
    ///   - but not 'cmp file_1 file_2 99 20 -i10' (-i overrides only first operand)
    /// - set 10 20:
    ///   - cmp file_1 file_2 10 20
    ///   - cmp file_1 file_2 -i10:20'
    ///   - cmp file_1 file_2 99 99 -i10:20 (-i overrides both operands)
    pub fn parse_params<I: Iterator<Item = OsString>>(mut opts: Peekable<I>) -> ResultParamsParse {
        let Some(name_executable) = opts.next() else {
            return Err(ParamsParseError::NoExecutable);
        };
        let mut params = Params {
            executable: name_executable,
            ..Default::default()
        };
        // let mut options = Vec::new();
        let mut operands = Vec::new();
        // let mut has_ignore_initial_2 = false;

        while let Some(param_os) = opts.next() {
            let mut param = param_os.to_string_lossy().to_string();
            // dbg!(&param);
            let mut ci = param.char_indices().peekable();
            // is param?
            let (_, c0) = ci.next().expect("Param must have at least one char!");
            if c0 == '-' {
                // check 2nd char
                match ci.next() {
                    Some((_, c1)) => {
                        // has 3rd char
                        match ci.peek() {
                            Some((pos_c2, _c2)) => {
                                if c1 == '-' {
                                    // long option, e.g. --bytes
                                    // find bytes for some options
                                    let mut bytes = match param[*pos_c2..].find('=') {
                                        Some(p) => {
                                            // only --bytes and --ignore-initial must have bytes, else error
                                            // reduce param to option and
                                            // return bytes without = sign.
                                            param.split_off(p + *pos_c2)[1..].to_string()
                                        }
                                        None => String::new(),
                                    };

                                    // allow partial option descriptors
                                    let possible_opts =
                                        Self::identify_options_from_partial_text(&param[2..]);
                                    match possible_opts.len() {
                                        0 => {
                                            return Err(ParamsParseError::UnrecognizedOption(
                                                params.executable_str(),
                                                param,
                                            ));
                                        }

                                        1 => param = possible_opts[0].to_string(),

                                        _ => {
                                            let mut list = Vec::new();
                                            for opt in possible_opts {
                                                list.push("'--".to_string() + opt + "'");
                                            }
                                            return Err(ParamsParseError::AmbiguousOption(
                                                params.executable_str(),
                                                param,
                                                list.join(" "),
                                            ));
                                        }
                                    }

                                    // only --bytes and ignore-initial allowed
                                    if !bytes.is_empty()
                                        && param != OPT_BYTES
                                        && param != OPT_IGNORE_INITIAL
                                    {
                                        return Err(ParamsParseError::ArgForOptionNotAllowed(
                                            params.executable_str(),
                                            param,
                                        ));
                                    }
                                    // remaining unique option
                                    match param.as_str() {
                                        OPT_BYTES => {
                                            if bytes.is_empty() {
                                                // must be the next arg
                                                bytes = Params::get_next_opt_as_bytes(&mut opts);
                                            }
                                            params.set_bytes_limit(&bytes)?;
                                        }

                                        OPT_HELP => {
                                            return Ok(ParamsParseOk::Info(ParamsParseInfo::Help));
                                        }

                                        OPT_IGNORE_INITIAL => {
                                            if bytes.is_empty() {
                                                // must be the next arg
                                                bytes = Params::get_next_opt_as_bytes(&mut opts);
                                            }
                                            // has_ignore_initial_2 =
                                            params.set_skip_bytes_files(&bytes)?;
                                        }
                                        OPT_QUIET | OPT_SILENT => {
                                            params.set_silent()?;
                                        }

                                        OPT_PRINT_BYTES => {
                                            params.set_print_bytes()?;
                                        }

                                        OPT_VERBOSE => {
                                            params.set_verbose()?;
                                        }

                                        OPT_VERSION => {
                                            return Ok(ParamsParseOk::Info(
                                                ParamsParseInfo::Version,
                                            ));
                                        }

                                        _ => {
                                            return Err(ParamsParseError::UnrecognizedOption(
                                                params.executable_str(),
                                                param,
                                            ));
                                        }
                                    }
                                } else {
                                    // -MultiSingleChar, e.g. -bl or option with bytes -n200
                                    let mut c = c1;
                                    let mut pos = 1;
                                    loop {
                                        match c {
                                            'b' => params.set_print_bytes()?,
                                            'l' => params.set_verbose()?,
                                            'i' => {
                                                // allow -bli50:100K
                                                let bytes = if param.len() > pos + 1 {
                                                    // all chars up to here are ASCII else Err would have been raised
                                                    param[pos + 1..].to_string()
                                                } else {
                                                    Params::get_next_opt_as_bytes(&mut opts)
                                                };
                                                // has_ignore_initial_2 =
                                                params.set_skip_bytes_files(&bytes)?;
                                                break;
                                            }
                                            'n' => {
                                                // allow -bln50
                                                let bytes = if param.len() > pos + 1 {
                                                    // all chars up to here are ASCII else Err would have been raised
                                                    param[pos + 1..].to_string()
                                                } else {
                                                    Params::get_next_opt_as_bytes(&mut opts)
                                                };
                                                params.set_bytes_limit(&bytes)?;
                                                break;
                                            }
                                            's' => params.set_silent()?,
                                            'v' => {
                                                return Ok(ParamsParseOk::Info(
                                                    ParamsParseInfo::Version,
                                                ));
                                            }
                                            _ => {
                                                return Err(ParamsParseError::InvalidOption(
                                                    params.executable_str(),
                                                    param,
                                                ));
                                            }
                                        }
                                        match ci.next() {
                                            Some((p, cx)) => {
                                                c = cx;
                                                pos = p
                                            }
                                            None => break,
                                        }
                                    }
                                }
                            }
                            None => {
                                // Check single short options, e.g. -b.
                                match c1 {
                                    'b' => params.set_print_bytes()?,
                                    // no need to store
                                    // b'b' => options.push(ParamOption::PrintBytes),
                                    'i' => {
                                        // bytes must be the next param
                                        let bytes = Params::get_next_opt_as_bytes(&mut opts);
                                        // has_ignore_initial_2 =
                                        params.set_skip_bytes_files(&bytes)?;
                                    }
                                    'l' => params.set_verbose()?,
                                    'n' => {
                                        // bytes must be the next param
                                        let bytes = Params::get_next_opt_as_bytes(&mut opts);
                                        params.set_bytes_limit(&bytes)?;
                                    }
                                    's' => params.set_silent()?,
                                    'v' => {
                                        return Ok(ParamsParseOk::Info(ParamsParseInfo::Version));
                                    }
                                    '-' => {
                                        // this is '--' only and behavior is unclear
                                        return Err(ParamsParseError::UnrecognizedOption(
                                            params.executable_str(),
                                            param,
                                        ));
                                    }
                                    _ => {
                                        return Err(ParamsParseError::InvalidOption(
                                            params.executable_str(),
                                            param,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        // single dash '-', this is for file as StandardInput
                        operands.push(param_os);
                    }
                }
            } else {
                // Operand, not an option with - or --
                operands.push(param_os);
            }
        }

        match operands.len() {
            0 => return Err(ParamsParseError::NoOperand(params.executable_str())),
            // If only file_1 is set, then file_2 defaults to '-', so it reads from StandardInput.
            1 => {
                params.file_1 = operands[0].clone();
                params.file_2 = OsString::from("-");
            }
            2..=4 => {
                params.file_1 = operands[0].clone();
                params.file_2 = operands[1].clone();
                if operands.len() > 2 && params.ignore_initial_bytes_file_1.is_none() {
                    params.set_skip_bytes_file_1(&operands[2].to_string_lossy())?;
                    // Alternative give Error
                    // return Err(ParamsParseError::IgnoreInitialDouble(
                    //     params.executable_str(),
                    //     operands[2].to_string_lossy().to_string(),
                    //     params.get_skip_bytes().to_string(),
                    // ));
                    // if operands.len() > 3 && !has_ignore_initial_2 {
                    if operands.len() > 3 {
                        params.set_skip_bytes_file_2(&operands[3].to_string_lossy())?;
                    }
                }
            }
            _ => {
                return Err(ParamsParseError::ExtraOperand(
                    params.executable_str(),
                    operands[4].to_string_lossy().to_string(),
                ));
            }
        }

        // Do as GNU cmp, and completely disable printing if we are
        // outputting to /dev/null.
        #[cfg(not(target_os = "windows"))]
        if crate::cmp::is_stdout_dev_null() {
            params.silent = true;
            params.verbose = false;
            params.print_bytes = false;
        }

        // dbg!(&params);
        Ok(ParamsParseOk::Params(params))
    }
}

impl From<ParamsCmp> for Params {
    fn from(p: ParamsCmp) -> Self {
        Self {
            executable: p.executable,
            file_1: p.file_1,
            file_2: p.file_2,
            ignore_initial_bytes_file_1: p.ignore_initial_bytes_file_1,
            ignore_initial_bytes_file_2: p.ignore_initial_bytes_file_2,
            bytes_limit: p.bytes_limit,
            print_bytes: p.print_bytes,
            silent: p.silent,
            verbose: p.verbose,
        }
    }
}

// Usually assert is used like assert_eq(result, desired_result).
#[cfg(test)]
mod tests {
    use crate::cmp::params::{BytesType, ParamsParseError, ResultParamsParse};

    use super::*;

    fn os(s: &str) -> OsString {
        OsString::from(s)
    }

    /// Simplify call of parser, just pass a normal string like in the Terminal.
    fn parse(args: &str) -> Result<ParamsParseOk, ParamsParseError> {
        let mut o = Vec::new();
        for arg in args.split(' ') {
            o.push(os(arg));
        }
        let p = o.into_iter().peekable();

        Params::parse_params(p)
    }

    fn res_ok(params: Params) -> ResultParamsParse {
        Ok(ParamsParseOk::Params(params))
    }

    #[test]
    fn positional() {
        // file_1 and file_2 given
        assert_eq!(
            parse("cmp foo bar"),
            res_ok(Params {
                executable: os("cmp"),
                file_1: os("foo"),
                file_2: os("bar"),
                ..Default::default()
            }),
        );

        // file_1 only
        assert_eq!(
            parse("cmp foo"),
            res_ok(Params {
                executable: os("cmp"),
                file_1: os("foo"),
                file_2: os("-"),
                ..Default::default()
            }),
        );

        // double dash without operand
        // Test fails as this behavior is not replicated.
        // assert_eq!(
        //     parse_params("cmp foo -- --help"),
        //     res_ok(Params {
        //         executable: os("cmp"),
        //         file_1: os("foo"),
        //         file_2: os("--help"),
        //         ..Default::default()
        //     }),
        // );

        // --ignore-initial for file_1 as operand
        assert_eq!(
            parse("cmp foo bar 1"),
            res_ok(Params {
                executable: os("cmp"),
                file_1: os("foo"),
                file_2: os("bar"),
                ignore_initial_bytes_file_1: Some(1),
                ignore_initial_bytes_file_2: None,
                ..Default::default()
            }),
        );

        // This test is not valid. GNU cmp gives an invalid error, it does not set it to usize::MAX
        // --ignore-initial as operands with 1 2Y (which is greater than u64)
        // assert_eq!(
        //     parse_params("cmp foo bar 1 2Y"),
        //     res_ok(Params {
        //         executable: os("cmp"),
        //         file_1: os("foo"),
        //         file_2: os("bar"),
        //         skip_bytes_file_1: Some(1),
        //         skip_bytes_file_2: Some(usize::MAX),
        //         ..Default::default()
        //     }),
        // );

        // Err: --ignore-initial as operands with 1 2Y (which is greater than u64)
        assert_eq!(
            parse("cmp foo bar 1 2Y"),
            Err(ParamsParseError::BytesInvalidUnit(
                "cmp".to_string(),
                BytesType::IgnoreInitial,
                "2Y".to_string()
            )),
        );

        // Err: too many operands
        assert_eq!(
            parse("cmp foo bar 1 2 3"),
            Err(ParamsParseError::ExtraOperand(
                "cmp".to_string(),
                "3".to_string()
            )),
        );

        // Err: no arguments
        assert_eq!(
            parse("cmp"),
            Err(ParamsParseError::NoOperand("cmp".to_string())),
        );
    }

    #[test]
    fn execution_modes() {
        // --print-bytes
        let print_bytes = Params {
            executable: os("cmp"),
            file_1: os("foo"),
            file_2: os("bar"),
            print_bytes: true,
            ..Default::default()
        };
        assert_eq!(parse("cmp -b foo bar"), res_ok(print_bytes.clone()));
        assert_eq!(
            parse("cmp --print-bytes foo bar"),
            res_ok(print_bytes.clone())
        );
        assert_eq!(parse("cmp --pr foo bar"), res_ok(print_bytes));

        // --verbose
        let verbose = Params {
            executable: os("cmp"),
            file_1: os("foo"),
            file_2: os("bar"),
            verbose: true,
            ..Default::default()
        };
        assert_eq!(parse("cmp -l foo bar"), res_ok(verbose.clone()));
        assert_eq!(parse("cmp --verbose foo bar"), res_ok(verbose.clone()));
        assert_eq!(parse("cmp --verb foo bar"), res_ok(verbose.clone()));
        assert_eq!(
            parse("cmp --ver foo bar"),
            Err(ParamsParseError::AmbiguousOption(
                "cmp".to_string(),
                "--ver".to_string(),
                "'--verbose' '--version'".to_string()
            )),
        );

        // --verbose & --print-bytes
        let verbose_and_print_bytes = Params {
            executable: os("cmp"),
            file_1: os("foo"),
            file_2: os("bar"),
            print_bytes: true,
            verbose: true,
            ..Default::default()
        };
        assert_eq!(
            parse("cmp -l -b foo bar"),
            res_ok(verbose_and_print_bytes.clone())
        );
        assert_eq!(
            parse("cmp -lb foo bar"),
            res_ok(verbose_and_print_bytes.clone())
        );
        assert_eq!(
            parse("cmp -bl foo bar"),
            res_ok(verbose_and_print_bytes.clone())
        );
        assert_eq!(
            parse("cmp --verbose --print-bytes foo bar"),
            res_ok(verbose_and_print_bytes.clone())
        );
        assert_eq!(
            parse("cmp --verb --p foo bar"),
            res_ok(verbose_and_print_bytes.clone())
        );

        // --silent --quiet
        let silent = Params {
            executable: os("cmp"),
            file_1: os("foo"),
            file_2: os("bar"),
            silent: true,
            ..Default::default()
        };
        assert_eq!(parse("cmp -s foo bar"), res_ok(silent.clone()));
        assert_eq!(parse("cmp --silent foo bar"), res_ok(silent.clone()));
        assert_eq!(parse("cmp --quiet foo bar"), res_ok(silent.clone()));

        // Some options do not mix.
        assert_eq!(
            parse("cmp -l -s foo bar"),
            Err(ParamsParseError::SilentVerboseIncompatible(
                "cmp".to_string(),
            )),
        );
        // This does not give an error in GNU cmp, but should.
        assert_eq!(
            parse("cmp -b -s foo bar"),
            Err(ParamsParseError::SilentPrintBytesIncompatible(
                "cmp".to_string(),
            )),
        );
    }

    #[test]
    /// These are all identical:
    /// - cmp file_1 file_2 -bl -n 1024
    /// - cmp file_1 file_2 -bl -n 1k
    /// - cmp file_1 file_2 -bl -n 1K
    /// - cmp file_1 file_2 -bl -n 1KiB
    /// - cmp file_1 file_2 -bl -n 1kiB
    /// - cmp file_1 file_2 -bl -n1kiB
    /// - cmp file_1 file_2 -bln1kiB
    fn bytes_limit() {
        let mut bytes_limit = Params {
            executable: os("cmp"),
            file_1: os("foo"),
            file_2: os("bar"),
            bytes_limit: Some(1000),
            ..Default::default()
        };
        assert_eq!(parse("cmp -n 1000 foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n1000 foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1kB foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1KB foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(
            parse("cmp -n 1kb foo bar"),
            Err(ParamsParseError::BytesInvalidUnit(
                "cmp".to_string(),
                BytesType::Limit,
                "1kb".to_string(),
            )),
        );

        bytes_limit.bytes_limit = Some(1024);
        assert_eq!(parse("cmp -n 1024 foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1k foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1K foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1KiB foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1kiB foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n1024 foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n1k foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n1K foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(
            parse("cmp --bytes=1024 foo bar"),
            res_ok(bytes_limit.clone())
        );
        assert_eq!(parse("cmp --bytes=1K foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(
            parse("cmp --bytes 1024 foo bar"),
            res_ok(bytes_limit.clone())
        );
        assert_eq!(parse("cmp --bytes 1K foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.print_bytes = true;
        bytes_limit.verbose = true;
        assert_eq!(parse("cmp -bln1kiB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.print_bytes = false;
        bytes_limit.verbose = false;

        // Test large numbers
        // Most modern Linux distributions (like Debian, Ubuntu, or CentOS) compile their core utilities (GNU diffutils) with Large File Support (LFS).
        // This uses the _FILE_OFFSET_BITS=64 flag, which forces the system to use a 64-bit integer ($off\_t$) for file offsets and sizes.
        // Even on a 32-bit processor, cmp can handle files much larger than the system's memory or 4 GB address space.The limit:
        // Technically $9,223,372,036,854,775,807$ bytes.
        // This is a problematic topic. File sizes can be larger than u64. Should the new cmp allow larger numbers (u128)?
        bytes_limit.bytes_limit = Some(1_000_000);
        assert_eq!(parse("cmp -n 1MB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_048_576);
        assert_eq!(parse("cmp -n 1M foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1MiB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_000_000_000);
        assert_eq!(parse("cmp -n 1GB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_073_741_824);
        assert_eq!(parse("cmp -n 1G foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1GiB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_000_000_000_000);
        assert_eq!(parse("cmp -n 1TB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_099_511_627_776);
        assert_eq!(parse("cmp -n 1T foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1TiB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_000_000_000_000_000);
        assert_eq!(parse("cmp -n 1PB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_125_899_906_842_624);
        assert_eq!(parse("cmp -n 1P foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1PiB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_000_000_000_000_000_000);
        assert_eq!(parse("cmp -n 1EB foo bar"), res_ok(bytes_limit.clone()));
        bytes_limit.bytes_limit = Some(1_152_921_504_606_846_976);
        assert_eq!(parse("cmp -n 1E foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1EiB foo bar"), res_ok(bytes_limit.clone()));

        // Error cases
        assert_eq!(
            parse("cmp -n 1ZB foo bar"),
            Err(ParamsParseError::BytesInvalidUnit(
                "cmp".to_string(),
                BytesType::Limit,
                "1ZB".to_string(),
            ))
        );
        assert_eq!(
            parse("cmp -n 99999999999999999999999999999999999999999999999999999999999 foo bar"),
            Err(ParamsParseError::BytesPosOverflow(
                "cmp".to_string(),
                BytesType::Limit,
                "99999999999999999999999999999999999999999999999999999999999".to_string(),
            ))
        );
    }

    #[test]
    fn ignore_initial() {
        let mut skips = Params {
            executable: os("cmp"),
            file_1: os("foo"),
            file_2: os("bar"),
            ignore_initial_bytes_file_1: Some(1),
            ignore_initial_bytes_file_2: Some(1),
            ..Default::default()
        };
        assert_eq!(parse("cmp -i 1 foo bar"), res_ok(skips.clone()));
        assert_eq!(
            parse("cmp --ignore-initial 1 foo bar"),
            res_ok(skips.clone())
        );
        assert_eq!(parse("cmp --ig 1 foo bar"), res_ok(skips.clone()));

        // 2nd value different
        skips.ignore_initial_bytes_file_2 = Some(2);
        assert_eq!(
            parse("cmp --ignore-initial=1:2 foo bar"),
            res_ok(skips.clone())
        );

        // Ignores positional arguments when -i is provided.
        assert_eq!(
            parse("cmp --ignore-initial=1:2 foo bar 3 4"),
            res_ok(skips.clone())
        );

        // large numbers
        skips.ignore_initial_bytes_file_1 = Some(1_000_000_000);
        skips.ignore_initial_bytes_file_2 = Some(2 * 1_152_921_504_606_846_976);
        assert_eq!(
            parse("cmp --ignore-initial=1GB:2E foo bar"),
            res_ok(skips.clone())
        );

        // Failure cases
        // Number too large
        assert_eq!(
            parse("cmp -i 99999999999999999999999999999999999999999999999999999999999 foo bar"),
            Err(ParamsParseError::BytesPosOverflow(
                "cmp".to_string(),
                BytesType::IgnoreInitial,
                "99999999999999999999999999999999999999999999999999999999999".to_string(),
            ))
        );
        // wrong unit
        assert_eq!(
            parse("cmp --ignore-initial=1mb foo bar"),
            Err(ParamsParseError::BytesInvalidUnit(
                "cmp".to_string(),
                BytesType::IgnoreInitial,
                "1mb".to_string(),
            ))
        );
        // too many values
        assert_eq!(
            parse("cmp --ignore-initial=1:2:3 foo bar"),
            Err(ParamsParseError::BytesInvalidUnit(
                "cmp".to_string(),
                BytesType::IgnoreInitial,
                "2:3".to_string(),
            ))
        );
        // negative value
        assert_eq!(
            parse("cmp --ignore-initial=-1 foo bar"),
            Err(ParamsParseError::BytesInvalidNumber(
                "cmp".to_string(),
                BytesType::IgnoreInitial,
                "-1".to_string(),
            ))
        );

        // All special suffixes.
        for (i, suffixes) in [
            ["kB", "K"],
            ["MB", "M"],
            ["GB", "G"],
            ["TB", "T"],
            ["PB", "P"],
            ["EB", "E"],
            // These values give an error in GNU cmp
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            ["ZB", "Z"],
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            ["YB", "Y"],
        ]
        .iter()
        .enumerate()
        {
            let values = [
                (1_000 as Bytes)
                    .checked_pow((i + 1) as u32)
                    .expect(&format!("number too large for suffix {:?}", suffixes)),
                (1024 as Bytes)
                    .checked_pow((i + 1) as u32)
                    .expect(&format!("number too large for suffix {:?}", suffixes)),
            ];
            for (j, v) in values.iter().enumerate() {
                assert_eq!(
                    parse(&format!("cmp -i 1{}:2 foo bar", suffixes[j])),
                    res_ok(Params {
                        executable: os("cmp"),
                        file_1: os("foo"),
                        file_2: os("bar"),
                        ignore_initial_bytes_file_1: Some(*v),
                        ignore_initial_bytes_file_2: Some(2),
                        ..Default::default()
                    }),
                );
            }
        }
    }
}
