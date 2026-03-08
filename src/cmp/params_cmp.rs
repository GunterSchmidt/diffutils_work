// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

use std::{ffi::OsString, iter::Peekable};

use crate::{
    arg_parser::{
        AppOption, Executable, NumberParser, OptionNameTypeUsed, ParseError, ParsedOption, Parser,
        OPT_HELP, OPT_VERSION,
    },
    cmp::{BytesLimitU64, SkipU64},
};

pub type ResultParamsCmpParse = Result<CmpParseOk, ParseError>;

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
pub const OPT_BYTES_LIMIT: AppOption = AppOption {
    long_name: "bytes",
    short: Some('n'),
    has_arg: true,
};
pub const OPT_IGNORE_INITIAL: AppOption = AppOption {
    long_name: "ignore-initial",
    short: Some('i'),
    has_arg: true,
};
pub const OPT_PRINT_BYTES: AppOption = AppOption {
    long_name: "print-bytes",
    short: Some('b'),
    has_arg: false,
};
pub const OPT_QUIET: AppOption = AppOption {
    long_name: "quiet",
    short: Some('s'),
    has_arg: false,
};
pub const OPT_SILENT: AppOption = AppOption {
    long_name: "silent",
    short: Some('s'),
    has_arg: false,
};
pub const OPT_VERBOSE: AppOption = AppOption {
    long_name: "verbose",
    short: Some('l'),
    has_arg: false,
};

// must contain OPT_HELP,and OPT_VERSION
pub const APP_OPTIONS: [AppOption; 8] = [
    OPT_BYTES_LIMIT,
    OPT_IGNORE_INITIAL,
    OPT_PRINT_BYTES,
    OPT_QUIET,
    OPT_SILENT,
    OPT_VERBOSE,
    OPT_HELP,
    OPT_VERSION,
];

/// Parser Result Ok Enum with Params.
///
/// # Returns
/// * Params in normal cases
/// * Just Help or Version when these are requested as the params are then not relevant.
///
/// Error will be returned as [ParseError] in the function Result Error.
#[derive(Debug, PartialEq)]
pub enum CmpParseOk {
    Params(ParamsCmp),
    Help,
    Version,
}

/// Holds the given command line arguments except "--version" and "--help".
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParamsCmp {
    pub executable: Executable,
    // pub executable: OsString,
    pub from: OsString,
    pub to: OsString,
    /// -n, --bytes=LIMIT          compare at most LIMIT bytes
    /// cmp from diffutils has a limit of i64::MAX (9_223_372_036_854_775_807)
    /// TODO This is wrong, or not?. If None limit will be set to Bytes::MAX.
    pub bytes_limit: Option<BytesLimitU64>,
    /// -i, --ignore-initial=SKIP         skip first SKIP bytes of both inputs
    pub skip_bytes_from: SkipU64,
    /// -i, --ignore-initial=SKIP1:SKIP2  skip first SKIP1 bytes of FILE1 and
    pub skip_bytes_to: SkipU64,
    /// -b, --print-bytes          print differing bytes
    pub print_bytes: bool,
    /// -s, --quiet, --silent      suppress all normal output \
    /// Do not set directly, use set_silent().
    pub silent: bool,
    /// -l, --verbose              output byte numbers and differing byte values \
    /// Do not set directly, use set_verbose().
    pub verbose: bool,
}

impl Default for ParamsCmp {
    fn default() -> Self {
        Self {
            executable: Executable::Cmp,
            from: Default::default(),
            to: Default::default(),
            skip_bytes_from: Default::default(),
            skip_bytes_to: Default::default(),
            bytes_limit: Default::default(),
            print_bytes: Default::default(),
            silent: Default::default(),
            verbose: Default::default(),
        }
    }
}

impl ParamsCmp {
    pub fn parse_params<I: Iterator<Item = OsString>>(
        executable: &Executable,
        args: Peekable<I>,
    ) -> ResultParamsCmpParse {
        let parser = Parser::parse_params(&APP_OPTIONS, args)?;

        let mut params = Self {
            executable: executable.clone(),
            ..Default::default()
        };

        // set options
        for parsed_option in &parser.options_parsed {
            // dbg!(&parsed_option);
            match *parsed_option.app_option {
                OPT_BYTES_LIMIT => {
                    params.set_bytes_limit(parsed_option)?;
                }
                OPT_HELP => return Ok(CmpParseOk::Help),
                OPT_IGNORE_INITIAL => {
                    params.set_skip_bytes_files(parsed_option)?;
                }
                OPT_PRINT_BYTES => params.set_print_bytes()?,
                OPT_QUIET | OPT_SILENT => params.set_silent()?,
                OPT_VERBOSE => params.set_verbose()?,
                OPT_VERSION => return Ok(CmpParseOk::Version),

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
            2..=4 => {
                params.from = parser.operands[0].clone();
                params.to = parser.operands[1].clone();
                // ignore if ignore-initial is already set by option
                if parser.operands.len() > 2 && params.skip_bytes_from == 0 {
                    // normally [set_skip_bytes_file] would be used, but GNU cmp does not set the 2nd parameter if operand is used.
                    params.set_skip_bytes_file_1(&ParsedOption::new(
                        &OPT_IGNORE_INITIAL,
                        parser.operands[2].clone(),
                        OptionNameTypeUsed::LongName,
                    ))?;
                    // TODO should crash
                    if let Some(skip) = parser.operands.get(3) {
                        params.set_skip_bytes_file_2(&ParsedOption::new(
                            &OPT_IGNORE_INITIAL,
                            skip.clone(),
                            OptionNameTypeUsed::LongName,
                        ))?;
                    }
                }
            }
            _ => {
                return Err(ParseError::ExtraOperand(parser.operands[4].clone()));
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
        Ok(CmpParseOk::Params(params))
    }

    /// Sets the --bytes limit and returns the input as number.
    ///
    /// bytes - unparsed number string, e.g. '50KiB'
    pub fn set_bytes_limit(
        &mut self,
        parsed_option: &ParsedOption,
    ) -> Result<BytesLimitU64, ParseError> {
        let num = NumberParser::parse_number(parsed_option)?;
        self.bytes_limit = Some(num);
        Ok(num)
    }

    /// Sets the ignore initial bytes for both files.
    ///
    /// Sets the 2nd file to the value of the 1st file if no second parameter is given. \
    /// Returns true if a value for the second file was given.
    /// TODO double?
    fn set_skip_bytes_files(&mut self, parsed_option: &ParsedOption) -> Result<bool, ParseError> {
        // if bytes.is_empty() {
        //     return Err(ParamsCmpError::ArgForOptionMissing(
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
    /// * ParsedOption.arg_for_option - A valid number String, e.g. 1800 or 12KiB
    ///
    /// If calling this manually, set_skip_bytes_file_2 to the same value unless
    /// separate values are required.  
    pub fn set_skip_bytes_file_1(
        &mut self,
        parsed_option: &ParsedOption,
    ) -> Result<SkipU64, ParseError> {
        self.set_skip_bytes_file_no(parsed_option, 1)
    }

    /// Sets the [Self::skip_bytes_file_2] value.
    ///
    /// * bytes - A valid number String, e.g. 1800 or 12KiB
    pub fn set_skip_bytes_file_2(
        &mut self,
        parsed_option: &ParsedOption,
    ) -> Result<SkipU64, ParseError> {
        self.set_skip_bytes_file_no(parsed_option, 2)
    }

    /// Sets the [Self::skip_bytes_file_1] value.
    ///
    /// If calling this manually, set_skip_bytes_file_2 to the same value unless
    /// separate values are required.  
    fn set_skip_bytes_file_no(
        &mut self,
        parsed_option: &ParsedOption,
        file_no: i32,
    ) -> Result<SkipU64, ParseError> {
        let skip = NumberParser::parse_number(parsed_option)?;
        match file_no {
            1 => self.skip_bytes_from = skip,
            2 => self.skip_bytes_to = skip,
            _ => panic!("Logic error."),
        }

        Ok(skip)
    }

    pub fn set_print_bytes(&mut self) -> Result<(), ParseError> {
        // Should actually raise an error if --silent is set, but GNU cmp does not do that.
        if self.silent {
            Err(ParseError::OptionsIncompatible(
                &OPT_PRINT_BYTES,
                &OPT_SILENT,
            ))
        } else {
            self.print_bytes = true;

            Ok(())
        }
    }

    pub fn set_silent(&mut self) -> Result<(), ParseError> {
        if self.verbose {
            Err(ParseError::OptionsIncompatible(&OPT_VERBOSE, &OPT_SILENT))
        } else if self.print_bytes {
            Err(ParseError::OptionsIncompatible(
                &OPT_PRINT_BYTES,
                &OPT_SILENT,
            ))
        } else {
            self.silent = true;

            Ok(())
        }
    }

    pub fn set_verbose(&mut self) -> Result<(), ParseError> {
        if self.silent {
            Err(ParseError::OptionsIncompatible(&OPT_VERBOSE, &OPT_SILENT))
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

    fn os(s: &str) -> OsString {
        OsString::from(s)
    }

    /// Simplify call of parser, just pass a normal string like in the Terminal.
    fn parse(args: &str) -> ResultParamsCmpParse {
        let mut o = Vec::new();
        for arg in args.split(' ') {
            o.push(os(arg));
        }
        let mut p = o.into_iter().peekable();
        // remove executable
        let executable = Executable::from_args_os(&mut p, true).unwrap();

        ParamsCmp::parse_params(&executable, p)
    }

    fn res_ok(params: ParamsCmp) -> ResultParamsCmpParse {
        Ok(CmpParseOk::Params(params))
    }

    #[test]
    fn positional() {
        // file_1 and file_2 given
        assert_eq!(
            parse("cmp foo bar"),
            res_ok(ParamsCmp {
                // util: Executable::Cmp,
                executable: Executable::Cmp,
                from: os("foo"),
                to: os("bar"),
                ..Default::default()
            }),
        );

        // file_1 only
        assert_eq!(
            parse("cmp foo"),
            res_ok(ParamsCmp {
                executable: Executable::Cmp,
                from: os("foo"),
                to: os("-"),
                ..Default::default()
            }),
        );

        // double dash without operand
        assert_eq!(
            parse("sdiff foo -- --help"),
            res_ok(ParamsCmp {
                executable: Executable::SDiff,
                from: os("foo"),
                to: os("--help"),
                ..Default::default()
            }),
        );

        // --ignore-initial for file_1 as operand
        assert_eq!(
            parse("cmp foo bar 1"),
            res_ok(ParamsCmp {
                executable: Executable::Cmp,
                from: os("foo"),
                to: os("bar"),
                skip_bytes_from: 1,
                skip_bytes_to: 0,
                ..Default::default()
            }),
        );

        // TODO This test is not valid, or?. GNU cmp gives an invalid error, it does not set it to usize::MAX
        // --ignore-initial as operands with 1 2Y (which is greater than u64)
        // assert_eq!(
        //     parse_params("cmp foo bar 1 2Y"),
        //     res_ok(ParamsCmp {
        //         util: Executable::Cmp,
        //         file_1: os("foo"),
        //         file_2: os("bar"),
        //         skip_bytes_file_1: Some(1),
        //         skip_bytes_file_2: Some(usize::MAX),
        //         ..Default::default()
        //     }),
        // );

        // Failure: --ignore-initial as operands with 1 2Y (which is greater than u64)
        let msg = "invalid --ignore-initial value '2Y'";
        match parse("cmp foo bar 1 2Y") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }

        // Err: too many operands
        assert_eq!(
            parse("cmp foo bar 1 2 3"),
            Err(ParseError::ExtraOperand(os("3")).into()),
        );

        // Err: no arguments
        assert_eq!(
            parse("cmp"),
            Err(ParseError::NoOperands(Executable::Cmp).into())
        );
    }

    #[test]
    fn execution_modes() {
        // --print-bytes
        let print_bytes = ParamsCmp {
            executable: Executable::Cmp,
            from: os("foo"),
            to: os("bar"),
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
            executable: Executable::Cmp,
            from: os("foo"),
            to: os("bar"),
            verbose: true,
            ..Default::default()
        };
        assert_eq!(parse("cmp -l foo bar"), res_ok(verbose.clone()));
        assert_eq!(parse("cmp --verbose foo bar"), res_ok(verbose.clone()));
        assert_eq!(parse("cmp --verb foo bar"), res_ok(verbose.clone()));

        // --ver ambiguous
        assert_eq!(
            parse("cmp --ver foo bar"),
            Err(ParseError::AmbiguousOption(
                os("--ver"),
                vec![&OPT_VERBOSE, &OPT_VERSION] // "'--verbose' '--version'".to_string()
            )
            .into()),
        );
        // let msg = "option '--ver' is ambiguous; possibilities: '--verbose' '--version'";
        let msg = "option '--ver' is ambiguous; possibilities: --verbose --version";
        match parse("cmp --ver foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }

        // --verbose & --print-bytes
        let verbose_and_print_bytes = ParamsCmp {
            executable: Executable::Cmp,
            from: os("foo"),
            to: os("bar"),
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
            executable: Executable::Cmp,
            from: os("foo"),
            to: os("bar"),
            silent: true,
            ..Default::default()
        };
        assert_eq!(parse("cmp -s foo bar"), res_ok(silent.clone()));
        assert_eq!(parse("cmp --silent foo bar"), res_ok(silent.clone()));
        assert_eq!(parse("cmp --quiet foo bar"), res_ok(silent.clone()));

        // Options --silent and --verbose do not mix.
        // let msg = "options -l and -s are incompatible";
        let msg = "options --verbose and --silent are incompatible";
        match parse("cmp -l -s foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }

        // Options --silent and --print-bytes do not mix.
        // This does not give an error in GNU cmp, but should.
        let msg = "options --print-bytes and --silent are incompatible";
        match parse("cmp -b -s foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }
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
            executable: Executable::Cmp,
            from: os("foo"),
            to: os("bar"),
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
        //     Err(ParamsCmpError::BytesInvalidUnit(
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
        let msg = "invalid --bytes value '1ZB'";
        match parse("cmp -n 1ZB foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }
        let msg =
            "invalid --bytes value '99999999999999999999999999999999999999999999999999999999999' (too large)";
        match parse("cmp -n 99999999999999999999999999999999999999999999999999999999999 foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }
    }

    #[test]
    fn ignore_initial() {
        let mut skips = ParamsCmp {
            executable: Executable::Cmp,
            from: os("foo"),
            to: os("bar"),
            skip_bytes_from: 1,
            skip_bytes_to: 1,
            ..Default::default()
        };
        assert_eq!(parse("cmp -i 1 foo bar"), res_ok(skips.clone()));
        assert_eq!(
            parse("cmp --ignore-initial 1 foo bar"),
            res_ok(skips.clone())
        );
        assert_eq!(parse("cmp --ig 1 foo bar"), res_ok(skips.clone()));

        // 2nd value different
        skips.skip_bytes_to = 2;
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
        skips.skip_bytes_from = 1_000_000_000;
        skips.skip_bytes_to = 2 * 1_152_921_504_606_846_976;
        assert_eq!(
            parse("cmp --ignore-initial=1GB:2E foo bar"),
            res_ok(skips.clone())
        );

        // Failure cases
        // Number too large
        // TODO This tests for the new error message which is different from GNU cmp
        let msg =
            "invalid --ignore-initial value '99999999999999999999999999999999999999999999999999999999999' (too large)";
        match parse("cmp -i 99999999999999999999999999999999999999999999999999999999999 foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }

        #[cfg(not(feature = "feat_allow_case_insensitive_number_units"))]
        {
            // wrong unit
            let msg = "invalid --ignore-initial value '1mb'";
            match parse("cmp --ignore-initial=1mb foo bar") {
                Ok(_) => assert!(false, "Should not be ok!"),
                Err(e) => assert!(
                    e.to_string().contains(msg),
                    "error must contain: \"{msg}\"\nactual error: \"{e}\""
                ),
            }
        }

        // wrong unit
        let msg = "invalid --ignore-initial value '1jb'";
        match parse("cmp --ignore-initial=1jb foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }

        // // too many values
        let msg = "invalid --ignore-initial value '2:3'";
        match parse("cmp --ignore-initial=1:2:3 foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
            ),
        }

        // negative value
        let msg = "invalid --ignore-initial value '-1'";
        match parse("cmp --ignore-initial=-1 foo bar") {
            Ok(_) => assert!(false, "Should not be ok!"),
            Err(e) => assert!(
                e.to_string().contains(msg),
                "error must contain: \"{msg}\"\nactual error: \"{e}\""
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
                (1_000 as SkipU64)
                    .checked_pow((i + 1) as u32)
                    .expect(&format!("number too large for suffix {:?}", suffixes)),
                (1024 as SkipU64)
                    .checked_pow((i + 1) as u32)
                    .expect(&format!("number too large for suffix {:?}", suffixes)),
            ];
            for (j, v) in values.iter().enumerate() {
                assert_eq!(
                    parse(&format!("cmp -i 1{}:2 foo bar", suffixes[j])),
                    res_ok(ParamsCmp {
                        executable: Executable::Cmp,
                        from: os("foo"),
                        to: os("bar"),
                        skip_bytes_from: *v,
                        skip_bytes_to: 2,
                        ..Default::default()
                    }),
                );
            }
        }
    }
}
