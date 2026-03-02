use std::{ffi::OsString, fmt::Display, iter::Peekable};

use crate::{
    arg_parser::{
        AppOption, ArgParser, ArgParserError, DiffUtility, OptionNameTypeUsed, ParseBytesError,
        ParsedOption, OPT_HELP, OPT_VERSION, TEXT_COPYRIGHT,
    },
    cmp::{Bytes, IgnInit, EXE_NAME},
};

pub type ResultParamsCmpParse = Result<ParamsCmpOk, ParamsCmpError>;

// -b, --print-bytes          print differing bytes
// -i, --ignore-initial=SKIP         skip first SKIP bytes of both inputs
// -i, --ignore-initial=SKIP1:SKIP2  skip first SKIP1 bytes of FILE1 and
//                                     first SKIP2 bytes of FILE2
// -l, --verbose              output byte numbers and differing byte values
// -n, --bytes=LIMIT          compare at most LIMIT bytes
// -s, --quiet, --silent      suppress all normal output
//     --help                 display this help and exit
// -v, --version              output version information and exit
const OPT_BYTES_LIMIT: AppOption = AppOption {
    long_name: "bytes",
    short: Some('n'),
    has_arg: true,
    arg_default: Some("10"),
};
const OPT_IGNORE_INITIAL: AppOption = AppOption {
    long_name: "ignore-initial",
    short: Some('i'),
    has_arg: true,
    arg_default: None,
};
const OPT_PRINT_BYTES: AppOption = AppOption {
    long_name: "print-bytes",
    short: Some('b'),
    has_arg: false,
    arg_default: None,
};
const OPT_QUIET: AppOption = AppOption {
    long_name: "quiet",
    short: Some('q'),
    has_arg: false,
    arg_default: None,
};
const OPT_SILENT: AppOption = AppOption {
    long_name: "silent",
    short: Some('s'),
    has_arg: false,
    arg_default: None,
};
const OPT_VERBOSE: AppOption = AppOption {
    long_name: "verbose",
    short: Some('l'),
    has_arg: false,
    arg_default: None,
};
// must contain OPT_HELP,and OPT_VERSION

const ARG_OPTIONS: [AppOption; 8] = [
    OPT_BYTES_LIMIT,
    OPT_IGNORE_INITIAL,
    OPT_PRINT_BYTES,
    OPT_QUIET,
    OPT_SILENT,
    OPT_VERBOSE,
    OPT_HELP,
    OPT_VERSION,
];

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

#[derive(Debug)]
pub enum ParamCmpOption {
    /// Bytes Limit with unparsed number String
    BytesLimit(String),
    Help,
    /// Ignore Initial with unparsed number String
    IgnoreInitial(String),
    PrintBytes,
    Silent,
    Verbose,
    Version,
}

impl From<&ParsedOption> for ParamCmpOption {
    fn from(opt: &ParsedOption) -> Self {
        match *opt.app_option {
            OPT_BYTES_LIMIT => ParamCmpOption::BytesLimit(
                opt.arg_for_option
                    .as_ref()
                    .expect("Logic error: Must have option arg.")
                    .clone(),
            ),
            OPT_HELP => ParamCmpOption::Help,
            OPT_IGNORE_INITIAL => ParamCmpOption::IgnoreInitial(
                opt.arg_for_option
                    .as_ref()
                    .expect("Logic error: Must have option arg.")
                    .clone(),
            ),
            OPT_PRINT_BYTES => ParamCmpOption::PrintBytes,
            OPT_QUIET | OPT_SILENT => ParamCmpOption::Silent,
            OPT_VERBOSE => ParamCmpOption::Verbose,
            OPT_VERSION => ParamCmpOption::Version,

            // This is not an error, but a todo. Unfortunately an Enum is not possible.
            _ => todo!("Err Option: {}", opt.app_option.long_name),
        }
    }
}

/// Success return type for parsing of params.
///
/// Successful parsing will return ParamsCmp, \
/// -- help und --version will return an Info message, \
/// Error will be returned as [ParamsCmpParseError] in the function Result.
#[derive(Debug, PartialEq)]
pub enum ParamsCmpOk {
    Info(ParamsCmpInfo),
    ParamsCmp(ParamsCmp),
}

/// Static texts for --help and --version.
///
/// The parser returns these enums to the caller, allowing the caller can identify this as information,
/// so that the program exit code is SUCCESS(0).
#[derive(Debug, PartialEq)]
pub enum ParamsCmpInfo {
    Help,
    Version,
}

impl Display for ParamsCmpInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = match self {
            ParamsCmpInfo::Help => TEXT_HELP,
            ParamsCmpInfo::Version => &format!("{TEXT_VERSION}\n{TEXT_COPYRIGHT}"),
        };

        write!(f, "{}", info)
    }
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

/// Holds the given command line arguments except "--version" and "--help".
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParamsCmp {
    /// Identifier
    pub util: DiffUtility,
    // pub executable: OsString,
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

impl Default for ParamsCmp {
    fn default() -> Self {
        Self {
            util: DiffUtility::Cmp,
            // executable: Default::default(),
            file_1: Default::default(),
            file_2: Default::default(),
            ignore_initial_bytes_file_1: Default::default(),
            ignore_initial_bytes_file_2: Default::default(),
            bytes_limit: Default::default(),
            print_bytes: Default::default(),
            silent: Default::default(),
            verbose: Default::default(),
        }
    }
}

impl ParamsCmp {
    pub fn parse_params<I: Iterator<Item = OsString>>(opts: Peekable<I>) -> ResultParamsCmpParse {
        let p_gen = ArgParser::parse_params(&ARG_OPTIONS, opts)?;
        Self::try_from(&p_gen)
    }

    fn try_from(p_gen: &ArgParser) -> ResultParamsCmpParse {
        let mut params = Self::default();
        //  {
        //     // executable: p_gen.executable.clone(),
        //     ..Default::default()
        // };

        // set options
        for parsed_option in &p_gen.options_parsed {
            let opt = ParamCmpOption::from(parsed_option);
            // dbg!(&parsed_option, &opt);
            match opt {
                ParamCmpOption::BytesLimit(_limit) => {
                    params.set_bytes_limit(parsed_option)?;
                    // if let Err(e) = params.set_bytes_limit(parsed_option) {
                    //     return Err(e);
                    //     // return Err(ParamsCmpParseError::from_parse_byte_error(e, opt_gen));
                    // }
                }
                ParamCmpOption::Help => {
                    return Ok(ParamsCmpOk::Info(ParamsCmpInfo::Help));
                }
                ParamCmpOption::IgnoreInitial(_skip1_2) => {
                    params.set_skip_bytes_files(parsed_option)?;
                }
                ParamCmpOption::PrintBytes => params.set_print_bytes()?,
                ParamCmpOption::Silent => params.set_silent()?,
                ParamCmpOption::Verbose => params.set_verbose()?,
                ParamCmpOption::Version => return Ok(ParamsCmpOk::Info(ParamsCmpInfo::Version)),
            }
        }

        // set operands
        match p_gen.operands.len() {
            0 => {
                return Err(ParamsCmpError::ArgParserError(ArgParserError::NoOperand(
                    params.util,
                )))
            }
            // If only file_1 is set, then file_2 defaults to '-', so it reads from StandardInput.
            1 => {
                params.file_1 = p_gen.operands[0].clone();
                params.file_2 = OsString::from("-");
            }
            2..=4 => {
                params.file_1 = p_gen.operands[0].clone();
                params.file_2 = p_gen.operands[1].clone();
                // ignore if ignore-initial is already set by option
                if p_gen.operands.len() > 2 && params.ignore_initial_bytes_file_1.is_none() {
                    // normally [set_skip_bytes_file] would be used, but GNU cmp does not set the 2nd parameter if operand is used.
                    params.set_skip_bytes_file_1(&ParsedOption {
                        app_option: &OPT_IGNORE_INITIAL,
                        arg_for_option: Some(p_gen.operands[2].to_string_lossy().to_string()),
                        name_type_used: OptionNameTypeUsed::LongName,
                    })?;
                    if p_gen.operands.len() > 3 {
                        params.set_skip_bytes_file_2(&ParsedOption {
                            app_option: &OPT_IGNORE_INITIAL,
                            arg_for_option: Some(p_gen.operands[3].to_string_lossy().to_string()),
                            name_type_used: OptionNameTypeUsed::LongName,
                        })?;
                    }
                    // Alternative give Error
                    // return Err(ParamsParseError::IgnoreInitialDouble(
                    //     params.executable_str(),
                    //     operands[2].to_string_lossy().to_string(),
                    //     params.get_skip_bytes().to_string(),
                    // ));
                }
            }
            _ => {
                return Err(ParamsCmpError::ExtraOperand(
                    p_gen.operands[4].to_string_lossy().to_string(),
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
        Ok(ParamsCmpOk::ParamsCmp(params))
    }

    /// Sets the --bytes limit and returns the input as number.
    ///
    /// bytes - unparsed number string, e.g. '50KiB'
    pub fn set_bytes_limit(
        &mut self,
        parsed_option: &ParsedOption,
    ) -> Result<Bytes, ParamsCmpError> {
        match ArgParser::parse_bytes(parsed_option.arg_for_option.as_ref().expect("Logic error")) {
            Ok(r) => {
                self.bytes_limit = Some(r);
                Ok(r)
            }
            Err(e) => Err(ParamsCmpError::from_parse_byte_error(e, parsed_option)),
            // match e {
            //     ParseBytesError::NoValue => Err(ParamsCmpParseError::ParseGenError(
            //         ParamsGenParseError::ArgForOptionMissing(parsed_option.clone()),
            //     )),
            //     ParseBytesError::PosOverflow => {
            //         Err(ParamsCmpParseError::BytesPosOverflow(parsed_option.clone()))
            //     }
            //     ParseBytesError::InvalidNumber => Err(ParamsCmpParseError::BytesInvalidNumber(
            //         parsed_option.clone(),
            //     )),
            //     ParseBytesError::InvalidUnit => {
            //         Err(ParamsCmpParseError::BytesInvalidUnit(parsed_option.clone()))
            //     }
            // },
        }
    }

    /// Sets the ignore initial bytes for both files.
    ///
    /// Sets the 2nd file to the value of the 1st file if no second parameter is given. \
    /// Returns true if a value for the second file was given.
    fn set_skip_bytes_files(
        &mut self,
        parsed_option: &ParsedOption,
    ) -> Result<bool, ParamsCmpError> {
        // if bytes.is_empty() {
        //     return Err(ParamsCmpParseError::ArgForOptionMissing(
        //         BytesType::IgnoreInitial,
        //     ));
        // }

        let bytes = parsed_option
            .arg_for_option
            .as_ref()
            .expect("Logic error")
            .as_str();
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

        let mut p = parsed_option.clone();
        p.arg_for_option = Some(skip_1.to_string());
        self.set_skip_bytes_file_1(&p)?;
        p.arg_for_option = Some(skip_2.to_string());
        self.set_skip_bytes_file_2(&p)?;

        Ok(has_2nd)
    }

    /// Sets the [Self::skip_bytes_file_1] value.
    ///
    /// * bytes - A valid number String, e.g. 1800 or 12KiB
    ///
    /// If calling this manually, set_skip_bytes_file_2 to the same value unless
    /// separate values are required.  
    pub fn set_skip_bytes_file_1(
        &mut self,
        parsed_option: &ParsedOption,
    ) -> Result<IgnInit, ParamsCmpError> {
        self.set_skip_bytes_file_no(parsed_option, 1)
    }

    /// Sets the [Self::skip_bytes_file_2] value.
    ///
    /// * bytes - A valid number String, e.g. 1800 or 12KiB
    pub fn set_skip_bytes_file_2(
        &mut self,
        parsed_option: &ParsedOption,
    ) -> Result<IgnInit, ParamsCmpError> {
        self.set_skip_bytes_file_no(parsed_option, 2)
    }

    /// Sets the [Self::skip_bytes_file_1] value.
    ///
    /// * bytes - A valid number String, e.g. 1800 or 12KiB
    ///
    /// If calling this manually, set_skip_bytes_file_2 to the same value unless
    /// separate values are required.  
    fn set_skip_bytes_file_no(
        &mut self,
        parsed_option: &ParsedOption,
        file_no: i32,
    ) -> Result<IgnInit, ParamsCmpError> {
        match ArgParser::parse_bytes(
            parsed_option
                .arg_for_option
                .as_ref()
                .expect("Logic error, must have value."),
        ) {
            Ok(r) => {
                #[cfg(feature = "cmp_bytes_limit_128_bit")]
                {
                    if r > IgnInit::MAX as u128 {
                        return Err(ParamsCmpError::BytesPosOverflow(parsed_option.clone()));
                    }
                    let r = r as IgnInit;
                    match file_no {
                        1 => self.ignore_initial_bytes_file_1 = Some(r),
                        2 => self.ignore_initial_bytes_file_2 = Some(r),
                        _ => panic!("Logic error."),
                    }
                    Ok(r)
                }
                #[cfg(not(feature = "cmp_bytes_limit_128_bit"))]
                {
                    match file_no {
                        1 => self.ignore_initial_bytes_file_1 = Some(r),
                        2 => self.ignore_initial_bytes_file_2 = Some(r),
                        _ => panic!("Logic error."),
                    }
                    Ok(r)
                }
            }
            Err(e) => Err(ParamsCmpError::from_parse_byte_error(e, parsed_option)),
        }
    }

    pub fn set_print_bytes(&mut self) -> Result<(), ParamsCmpError> {
        // Should actually raise an error if --silent is set, but GNU cmp does not do that.
        if self.silent {
            Err(ParamsCmpError::SilentPrintBytesIncompatible)
        } else {
            self.print_bytes = true;

            Ok(())
        }
    }

    pub fn set_silent(&mut self) -> Result<(), ParamsCmpError> {
        if self.verbose {
            Err(ParamsCmpError::SilentVerboseIncompatible)
        } else if self.print_bytes {
            Err(ParamsCmpError::SilentPrintBytesIncompatible)
        } else {
            self.silent = true;

            Ok(())
        }
    }

    pub fn set_verbose(&mut self) -> Result<(), ParamsCmpError> {
        if self.silent {
            Err(ParamsCmpError::SilentVerboseIncompatible)
        } else {
            self.verbose = true;

            Ok(())
        }
    }
}

// Usually assert is used like assert_eq(result, desired_result).
#[cfg(test)]
mod tests {
    use crate::arg_parser::OPT_VERSION;

    use super::*;

    pub const TEXT_HELP_HINT: &str = "Try 'cmp --help' for more information.";

    fn os(s: &str) -> OsString {
        OsString::from(s)
    }

    /// Simplify call of parser, just pass a normal string like in the Terminal.
    fn parse(args: &str) -> ResultParamsCmpParse {
        let mut o = Vec::new();
        for arg in args.split(' ') {
            o.push(os(arg));
        }
        let p = o.into_iter().peekable();

        ParamsCmp::parse_params(p)
    }

    fn res_ok(params: ParamsCmp) -> ResultParamsCmpParse {
        Ok(ParamsCmpOk::ParamsCmp(params))
    }

    #[test]
    fn positional() {
        // file_1 and file_2 given
        assert_eq!(
            parse("cmp foo bar"),
            res_ok(ParamsCmp {
                // util: DiffUtility::Cmp,
                util: DiffUtility::Cmp,
                file_1: os("foo"),
                file_2: os("bar"),
                ..Default::default()
            }),
        );

        // file_1 only
        assert_eq!(
            parse("cmp foo"),
            res_ok(ParamsCmp {
                util: DiffUtility::Cmp,
                file_1: os("foo"),
                file_2: os("-"),
                ..Default::default()
            }),
        );

        // double dash without operand
        // Test fails as this behavior is not replicated.
        // assert_eq!(
        //     parse_params("cmp foo -- --help"),
        //     res_ok(ParamsCmp {
        //         util: DiffUtility::Cmp,
        //         file_1: os("foo"),
        //         file_2: os("--help"),
        //         ..Default::default()
        //     }),
        // );

        // --ignore-initial for file_1 as operand
        assert_eq!(
            parse("cmp foo bar 1"),
            res_ok(ParamsCmp {
                util: DiffUtility::Cmp,
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
        //     res_ok(ParamsCmp {
        //         util: DiffUtility::Cmp,
        //         file_1: os("foo"),
        //         file_2: os("bar"),
        //         skip_bytes_file_1: Some(1),
        //         skip_bytes_file_2: Some(usize::MAX),
        //         ..Default::default()
        //     }),
        // );

        #[cfg(feature = "cmp_bytes_limit_128_bit")]
        {
            // Ok 128-Bit: --ignore-initial as operands with 1 2Y (which is greater than u64)
            let bytes_limit = ParamsCmp {
                util: DiffUtility::Cmp,
                file_1: os("foo"),
                file_2: os("bar"),
                bytes_limit: Some(2 * 1_208_925_819_614_629_174_706_176),
                ..Default::default()
            };
            assert_eq!(parse("cmp foo bar --bytes=2Y"), res_ok(bytes_limit));
        }

        #[cfg(feature = "cmp_bytes_limit_128_bit")]
        {
            // Failure: --ignore-initial as operands with 1 2Y (which is greater than u64)
            assert_eq!(
                parse("cmp foo bar 1 2Y"),
                Err(ParamsCmpParseError::BytesPosOverflow(ParsedOption {
                    app_option: &OPT_IGNORE_INITIAL,
                    arg_for_option: Some("2Y".to_string()),
                    name_type_used: OptionNameTypeUsed::LongName
                })),
            );
        }
        #[cfg(not(feature = "cmp_bytes_limit_128_bit"))]
        {
            // Failure: --ignore-initial as operands with 1 2Y (which is greater than u64)
            assert_eq!(
                parse("cmp foo bar 1 2Y"),
                Err(ParamsCmpError::BytesInvalidUnit(ParsedOption {
                    app_option: &OPT_IGNORE_INITIAL,
                    arg_for_option: Some("2Y".to_string()),
                    name_type_used: OptionNameTypeUsed::LongName
                })),
            );
        }
        // Err: too many operands
        assert_eq!(
            parse("cmp foo bar 1 2 3"),
            Err(ParamsCmpError::ExtraOperand("3".to_string())),
        );

        // Err: no arguments
        assert_eq!(
            parse("cmp"),
            Err(ParamsCmpError::ArgParserError(ArgParserError::NoOperand(
                DiffUtility::Cmp
            )))
        );
    }

    #[test]
    fn execution_modes() {
        // --print-bytes
        let print_bytes = ParamsCmp {
            util: DiffUtility::Cmp,
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
        let verbose = ParamsCmp {
            util: DiffUtility::Cmp,
            file_1: os("foo"),
            file_2: os("bar"),
            verbose: true,
            ..Default::default()
        };
        assert_eq!(parse("cmp -l foo bar"), res_ok(verbose.clone()));
        assert_eq!(parse("cmp --verbose foo bar"), res_ok(verbose.clone()));
        assert_eq!(parse("cmp --verb foo bar"), res_ok(verbose.clone()));

        // --ver ambiguous
        assert_eq!(
            parse("cmp --ver foo bar"),
            Err(ParamsCmpError::ArgParserError(
                ArgParserError::AmbiguousOption(
                    "--ver".to_string(),
                    vec![&OPT_VERBOSE, &OPT_VERSION] // "'--verbose' '--version'".to_string()
                )
            )),
        );
        let r = parse("cmp --ver foo bar");
        match r {
            Ok(_) => assert!(false, "Should not be Ok."),
            Err(e) => assert!(e.to_string().contains(
                "cmp: option '--ver' is ambiguous; possibilities: '--verbose' '--version'"
            )),
        }

        // --verbose & --print-bytes
        let verbose_and_print_bytes = ParamsCmp {
            util: DiffUtility::Cmp,
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
        let silent = ParamsCmp {
            util: DiffUtility::Cmp,
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
            Err(ParamsCmpError::SilentVerboseIncompatible),
        );
        // This does not give an error in GNU cmp, but should.
        assert_eq!(
            parse("cmp -b -s foo bar"),
            Err(ParamsCmpError::SilentPrintBytesIncompatible),
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
        let mut bytes_limit = ParamsCmp {
            util: DiffUtility::Cmp,
            file_1: os("foo"),
            file_2: os("bar"),
            bytes_limit: Some(1000),
            ..Default::default()
        };
        assert_eq!(parse("cmp -n 1000 foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n1000 foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1kB foo bar"), res_ok(bytes_limit.clone()));
        assert_eq!(parse("cmp -n 1KB foo bar"), res_ok(bytes_limit.clone()));
        // TODO This is allowed
        // assert_eq!(
        //     parse("cmp -n 1kb foo bar"),
        //     Err(ParamsCmpParseError::BytesInvalidUnit(
        //         BytesType::Limit,
        //         "1kb".to_string(),
        //     )),
        // );

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

        // Failure cases
        #[cfg(feature = "cmp_bytes_limit_128_bit")]
        {
            bytes_limit.bytes_limit = Some(1_000_000_000_000_000_000_000);
            assert_eq!(parse("cmp -n 1ZB foo bar"), res_ok(bytes_limit.clone()));
        }

        #[cfg(not(feature = "cmp_bytes_limit_128_bit"))]
        {
            assert_eq!(
                parse("cmp -n 1ZB foo bar"),
                Err(ParamsCmpError::BytesInvalidUnit(ParsedOption::new(
                    &OPT_BYTES_LIMIT,
                    "1ZB".to_string(),
                    OptionNameTypeUsed::ShortName
                )))
            );
            let r = parse("cmp -n 1ZB foo bar");
            match r {
                Ok(_) => assert!(false, "Should not be Ok."),
                Err(e) => assert_eq!(
                    e.to_string(),
                    format!("cmp: invalid '--bytes' value '1ZB'\ncmp: {TEXT_HELP_HINT}")
                ),
            }
        }

        assert_eq!(
            parse("cmp -n 99999999999999999999999999999999999999999999999999999999999 foo bar"),
            Err(ParamsCmpError::BytesPosOverflow(ParsedOption::new(
                &OPT_BYTES_LIMIT,
                "99999999999999999999999999999999999999999999999999999999999".to_string(),
                OptionNameTypeUsed::ShortName
            )))
        );
    }

    #[test]
    fn ignore_initial() {
        let mut skips = ParamsCmp {
            util: DiffUtility::Cmp,
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
        // TODO This tests for the new error message which is different from GNU cmp
        let r = parse("cmp -i 99999999999999999999999999999999999999999999999999999999999 foo bar");
        match r {
            Ok(_) => assert!(false, "Should not be Ok."),
            Err(e) => assert_eq!(
                e.to_string(),
                format!("cmp: invalid '--ignore-initial' value (too large) '99999999999999999999999999999999999999999999999999999999999'\ncmp: {TEXT_HELP_HINT}")
            ),
        }

        #[cfg(not(feature = "cmp_allow_case_insensitive_byte_units"))]
        {
            // wrong unit
            let r = parse("cmp --ignore-initial=1mb foo bar");
            match r {
                Ok(_) => assert!(false, "Should not be Ok."),
                Err(e) => assert_eq!(
                    e.to_string(),
                    format!("cmp: invalid '--ignore-initial' value '1mb'\ncmp: {TEXT_HELP_HINT}")
                ),
            }
        }

        // wrong unit
        let r = parse("cmp --ignore-initial=1jb foo bar");
        match r {
            Ok(_) => assert!(false, "Should not be Ok."),
            Err(e) => assert_eq!(
                e.to_string(),
                format!("cmp: invalid '--ignore-initial' value '1jb'\ncmp: {TEXT_HELP_HINT}")
            ),
        }

        // // too many values
        let r = parse("cmp --ignore-initial=1:2:3 foo bar");
        match r {
            Ok(_) => assert!(false, "Should not be Ok."),
            Err(e) => assert_eq!(
                e.to_string(),
                format!("cmp: invalid '--ignore-initial' value '2:3'\ncmp: {TEXT_HELP_HINT}")
            ),
        }

        // negative value
        let r = parse("cmp --ignore-initial=-1 foo bar");
        match r {
            Ok(_) => assert!(false, "Should not be Ok."),
            Err(e) => assert_eq!(
                e.to_string(),
                format!("cmp: invalid '--ignore-initial' value '-1'\ncmp: {TEXT_HELP_HINT}")
            ),
        }

        // All special suffixes for ignore-initial.
        for (i, suffixes) in [
            ["kB", "K"],
            ["MB", "M"],
            ["GB", "G"],
            ["TB", "T"],
            ["PB", "P"],
            ["EB", "E"],
            // These values give an error in GNU cmp
            // #[cfg(feature = "cmp_bytes_limit_128_bit")]
            // ["ZB", "Z"],
            // #[cfg(feature = "cmp_bytes_limit_128_bit")]
            // ["YB", "Y"],
        ]
        .iter()
        .enumerate()
        {
            let values = [
                (1_000 as IgnInit)
                    .checked_pow((i + 1) as u32)
                    .expect(&format!("number too large for suffix {:?}", suffixes)),
                (1024 as IgnInit)
                    .checked_pow((i + 1) as u32)
                    .expect(&format!("number too large for suffix {:?}", suffixes)),
            ];
            for (j, v) in values.iter().enumerate() {
                assert_eq!(
                    parse(&format!("cmp -i 1{}:2 foo bar", suffixes[j])),
                    res_ok(ParamsCmp {
                        util: DiffUtility::Cmp,
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
