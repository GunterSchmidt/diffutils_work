/// This is a generic parser for params/options.
///
/// The concept is to have this generic parser, which will parse e.g. 'cmp --options' or 'diff --options'. \
/// For the parser to know which options are possible, they must be given as a list of AppOptions.
use std::{ffi::OsString, fmt::Display, iter::Peekable};

use crate::cmp::{Bytes, EXE_NAME};

pub type ResultParamsGenParse = Result<ParamsGen, ParamsGenParseError>;
type ResultBytesParse = Result<Bytes, ParseBytesError>;

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

// TODO Help text
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

/// This contains the args/options the app allows. They must be all of const value.
#[derive(Debug, PartialEq)]
pub struct AppOption {
    /// long name of option
    pub long_name: &'static str,
    pub short: Option<char>,
    pub has_arg: bool,
    pub arg_default: Option<&'static str>,
}

// #[derive(Debug)]
// pub struct AppOptions(&'static [AppOption]);
//
// impl AppOptions {
//     fn identify_options_from_partial_text(&self, opt: &str) -> Vec<&'static AppOption> {
//         let l = opt.len();
//         let v: Vec<&'static AppOption> = self
//             .0
//             .iter()
//             .filter(|&it| it.long.len() >= l && &it.long[0..l] == opt)
//             // .copied()
//             .collect();
//
//         v
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedOption {
    pub app_option: &'static AppOption,
    pub arg_for_option: Option<String>,
    pub name_type_used: OptionNameTypeUsed,
}

impl ParsedOption {
    #[allow(unused)]
    pub fn new(
        app_option: &'static AppOption,
        arg_for_option: String,
        name_type_used: OptionNameTypeUsed,
    ) -> Self {
        Self {
            app_option,
            arg_for_option: Some(arg_for_option),
            name_type_used,
        }
    }

    pub fn new_none(app_option: &'static AppOption, name_type_used: OptionNameTypeUsed) -> Self {
        Self {
            app_option,
            arg_for_option: None,
            name_type_used,
        }
    }

    pub fn check_add_arg<I: Iterator<Item = OsString>>(
        &mut self,
        opts: &mut Peekable<I>,
    ) -> Result<(), ParamsGenParseError> {
        // argument missing
        if self.app_option.has_arg {
            if self.arg_for_option.is_none() {
                // take following argument
                if let Some(arg) = opts.next() {
                    self.arg_for_option = Some(arg.to_string_lossy().to_string())
                }
                if self.arg_for_option.is_none() {
                    if let Some(default) = self.app_option.arg_default {
                        self.arg_for_option = Some(default.to_string())
                    } else {
                        return Err(ParamsGenParseError::ArgForOptionMissing(self.clone()));
                    }
                }
            }
        } else {
            // argument allowed?
            if self.arg_for_option.is_some() {
                return Err(ParamsGenParseError::ArgForOptionNotAllowed(self.clone()));
            }
        }

        Ok(())
    }
}

impl Default for ParsedOption {
    fn default() -> Self {
        Self {
            app_option: &AppOption {
                long_name: "dummy",
                short: None,
                has_arg: false,
                arg_default: None,
            },
            arg_for_option: None,
            name_type_used: OptionNameTypeUsed::LongName,
        }
    }
}

/// To differentiate the user input, did he use -s or --silent.
/// While this is technically no difference, the error message may vary.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum OptionNameTypeUsed {
    #[default]
    LongName,
    ShortName,
}

// trait ParamsParseError {
//     fn write_err(f: &mut std::fmt::Formatter<'_>, msg: &str) -> Result<(), std::fmt::Error>;
// }

/// Contains all parser errors and their text messages.
///
/// First argument is always the exe name ('cmp'). \
#[derive(Debug, PartialEq)]
pub enum ParamsGenParseError {
    /// When the long option is abbreviated, but does not have a unique match.
    /// (ambiguous option, possible options)
    AmbiguousOption(String, Vec<&'static AppOption>),

    /// cmp: option '--silent' doesn't allow an argument
    /// (wrong option)
    ArgForOptionNotAllowed(ParsedOption),

    /// (option, short or long name used)
    ArgForOptionMissing(ParsedOption),

    /// cmp as parameter missing.
    NoExecutable,

    /// Non-existent single dash option.
    /// (unidentified option)
    InvalidOption(String),

    /// cmp but no args for it
    NoOperand,

    // /// Two dashes '--' without option not allowed. GNU cmp has somewhat undefined behavior, this is cleaner.
    // OptionUndefined(String),
    /// Non-existent double dash option. This is unrecognized because the name can be abbreviated.
    /// (unrecognized option)
    UnrecognizedOption(String),
}

// impl ParamsParseError for ParamsGenParseError {
//     fn write_err(f: &mut std::fmt::Formatter<'_>, msg: &str) -> Result<(), std::fmt::Error> {
//         write!(f, "{EXE_NAME}: {msg}\n{EXE_NAME}: {TEXT_HELP_HINT}")
//     }
// }

// #[derive(Debug)]
// pub struct ParamsGenParseErrorExe {
//     executable: String,
//     err: ParamsGenParseError,
// }

// impl ParamsGenParseError {
//     /// Writes the error message, adds cmp: and the --help information.
//     /// This passes the executable name as the caller does not know this
//     fn write_err(
//         f: &mut std::fmt::Formatter<'_>,
//         cmp: &str,
//         msg: &str,
//     ) -> Result<(), std::fmt::Error> {
//         write!(f, "{cmp}: {msg}\n{cmp}: {TEXT_HELP_HINT}")
//     }
// }

impl Display for ParamsGenParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // fn write_err(
        //     f: &mut std::fmt::Formatter<'_>,
        //     exe_name: &str,
        //     msg: &str,
        // ) -> Result<(), std::fmt::Error> {
        //     write!(f, "{exe_name}: {msg}\n{exe_name}: {TEXT_HELP_HINT}")
        // }
        fn write_err(f: &mut std::fmt::Formatter<'_>, msg: &str) -> Result<(), std::fmt::Error> {
            write!(f, "{EXE_NAME}: {msg}\n{EXE_NAME}: {TEXT_HELP_HINT}")
        }

        match &self {
            ParamsGenParseError::AmbiguousOption(param, possible_opts) => {
                // create list of possible options
                let mut list = Vec::new();
                for opt in possible_opts {
                    list.push("'--".to_string() + opt.long_name + "'");
                }
                write_err(
                    f,
                    // &self.executable,
                    &format!(
                        "option '{param}' is ambiguous; possibilities: {}",
                        list.join(" ")
                    ),
                )
            }

            ParamsGenParseError::ArgForOptionNotAllowed(opt) => write_err(
                f,
                // &self.executable,
                &format!(
                    "option '{}' doesn't allow an argument",
                    opt.app_option.long_name
                ),
            ),
            ParamsGenParseError::ArgForOptionMissing(opt) => {
                // TODO differentiate long and short name
                write_err(
                    f,
                    // &self.executable,
                    &format!("option {} requires an argument", opt.app_option.long_name),
                )
            }
            // ParamsGenParseError::BytesInvalidNumber(t, n) => {
            //     write_err(f, &format!("invalid {t} value '{n}'"))
            // }
            // ParamsGenParseError::BytesInvalidUnit(t, unit) => {
            //     write_err(f, &format!("invalid {t} value '{unit}'"))
            // }
            // ParamsGenParseError::BytesPosOverflow(t, bytes) => {
            //     write_err(f, &format!("invalid {t} value (too large) '{bytes}'"))
            // }
            ParamsGenParseError::NoExecutable => write!(
                f,
                "This program must be called with 'cmp' and its parameters."
            ),
            // ParamsGenParseError::ExtraOperand(opt) => {
            //     write_err(f, &format!("extra operand '{opt}'"))
            // }
            // ParamsGenParseError::IgnoreInitialDouble( op3, ig) => {
            //     write_err(f,  &format!("option '--ignore-initial' ('-i') is set to {ig} but also values ares passed as operand '{op3}'"))
            // }
            ParamsGenParseError::InvalidOption(opt) => {
                // write_err(f, &self.executable, &format!("invalid option '{opt}'"))
                write_err(f, &format!("invalid option '{opt}'"))
            }
            ParamsGenParseError::NoOperand => write_err(
                f,
                // &self.executable,
                &format!("missing operand after '{EXE_NAME}'"),
            ),
            ParamsGenParseError::UnrecognizedOption(opt) => {
                // write_err(f, &self.executable, &format!("unrecognized option '{opt}'"))
                write_err(f, &format!("unrecognized option '{opt}'"))
            }
        }
    }
}

pub enum ParseBytesError {
    NoValue,
    PosOverflow,
    InvalidNumber,
    InvalidUnit,
}

#[derive(Debug, Default)]
pub struct ParamsGen {
    pub executable: OsString,
    pub options_parsed: Vec<ParsedOption>,
    pub operands: Vec<OsString>, // pub arg_options: &'a [ArgOption],
}

impl ParamsGen {
    pub fn parse_params<I: Iterator<Item = OsString>>(
        arg_options: &'static [AppOption],
        mut opts: Peekable<I>,
    ) -> ResultParamsGenParse {
        let Some(name_executable) = opts.next() else {
            return Err(ParamsGenParseError::NoExecutable);
        };
        let mut params = Self {
            executable: name_executable,
            // arg_options: options,
            ..Default::default()
        };

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
                        let mut p_opt = ParsedOption::default();
                        // has 3rd char
                        match ci.peek() {
                            Some((pos_c2, _c2)) => {
                                if c1 == '-' {
                                    // long option, e.g. --bytes

                                    // Find argument for some options, either '=' or following arg.
                                    // This also shortens param to the name.
                                    if let Some(p) = param[*pos_c2..].find('=') {
                                        // only --bytes and --ignore-initial must have bytes, else error
                                        // reduce param to option and
                                        // return bytes without = sign.
                                        p_opt.arg_for_option =
                                            Some(param.split_off(p + *pos_c2)[1..].to_string());
                                    }

                                    let possible_opts = Self::identify_options_from_partial_text(
                                        // allow partial option descriptors
                                        arg_options,
                                        &param[2..],
                                    );
                                    match possible_opts.len() {
                                        0 => {
                                            return Err(ParamsGenParseError::UnrecognizedOption(
                                                param,
                                            ));
                                        }

                                        1 => p_opt.app_option = *possible_opts.first().unwrap(),

                                        _ => {
                                            return Err(ParamsGenParseError::AmbiguousOption(
                                                param,
                                                possible_opts,
                                            ));
                                        }
                                    }

                                    // identified unique option
                                    p_opt.name_type_used = OptionNameTypeUsed::LongName;
                                    p_opt.check_add_arg(&mut opts)?;
                                    params.options_parsed.push(p_opt);
                                } else {
                                    // -MultiSingleChar, e.g. -bl or option with bytes -n200
                                    let mut c = c1;
                                    let mut pos = 1;
                                    loop {
                                        match arg_options.iter().find(|o| o.short == Some(c)) {
                                            Some(opt) => {
                                                if opt.has_arg {
                                                    // take rest of the string as arg
                                                    let arg_for_option = if param.len() > pos + 1 {
                                                        Some(param[pos + 1..].to_string())
                                                    } else {
                                                        opts.next().map(|arg| {
                                                            arg.to_string_lossy().to_string()
                                                        })
                                                    };
                                                    match arg_for_option {
                                                        Some(_) => {
                                                            params.options_parsed.push(
                                                                ParsedOption {
                                                                    app_option: opt,
                                                                    arg_for_option,
                                                                    name_type_used: OptionNameTypeUsed::ShortName,
                                                                },
                                                            );
                                                            break;
                                                        }
                                                        None => return Err(ParamsGenParseError::ArgForOptionMissing(ParsedOption::new_none(opt, OptionNameTypeUsed::ShortName))),
                                                    }
                                                } else {
                                                    params.options_parsed.push(ParsedOption {
                                                        app_option: opt,
                                                        arg_for_option: None,
                                                        name_type_used:
                                                            OptionNameTypeUsed::ShortName,
                                                    });
                                                }
                                            }
                                            None => {
                                                return Err(ParamsGenParseError::InvalidOption(
                                                    param,
                                                ))
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
                                // single short options, e.g. -b.
                                match arg_options.iter().find(|opt| {
                                    if let Some(c) = opt.short {
                                        c == c1
                                    } else {
                                        false
                                    }
                                }) {
                                    Some(opt) => {
                                        // identified unique option
                                        p_opt.app_option = opt;
                                        p_opt.name_type_used = OptionNameTypeUsed::ShortName;
                                        p_opt.check_add_arg(&mut opts)?;
                                        params.options_parsed.push(p_opt);

                                        // params.options_parsed.push(ParsedOption {
                                        //     app_option: a,
                                        //     arg_for_option: None,
                                        //     name_type_used: OptionNameTypeUsed::ShortName,
                                        // });
                                    }
                                    None => return Err(ParamsGenParseError::InvalidOption(param)),
                                }
                            }
                        }
                    }
                    None => {
                        // single dash '-', this is for file as StandardInput
                        params.operands.push(param_os);
                    }
                }
            } else {
                // Operand, not an option with - or --
                params.operands.push(param_os);
            }
        }

        if params.operands.is_empty() {
            return Err(ParamsGenParseError::NoOperand);
        }

        Ok(params)
    }

    pub fn identify_options_from_partial_text(
        app_options: &'static [AppOption],
        opt: &str,
    ) -> Vec<&'static AppOption> {
        let l = opt.len();
        let v: Vec<&'static AppOption> = app_options
            .iter()
            .filter(|&it| it.long_name.len() >= l && &it.long_name[0..l] == opt)
            // .copied()
            .collect();

        v
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
    #[cfg(not(feature = "cmp_allow_case_insensitive_byte_units"))]
    pub fn parse_number_unit(unit: &str) -> ResultBytesParse {
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
            "Z" | "ZiB" => 1_180_591_620_717_411_303_424,
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "YB" => 1_000_000_000_000_000_000_000_000,
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "Y" | "YiB" => 1_208_925_819_614_629_174_706_176,
            _ => {
                return Err(ParseBytesError::InvalidUnit);
            }
        };

        Ok(multiplier)
    }

    /// Returns a multiplier depending on the given unit, e.g. 'KiB' -> 1024,
    /// which then can be used to calculate the final number of bytes.
    /// Following GNU documentation: https://www.gnu.org/software/diffutils/manual/html_node/cmp-Options.html
    #[cfg(feature = "cmp_allow_case_insensitive_byte_units")]
    pub fn parse_number_unit(unit: &str) -> ResultBytesParse {
        // Note that GNU cmp advertises supporting up to Y, but fails if you try
        // to actually use anything beyond E.
        let unit = unit.to_owned().to_ascii_lowercase();
        // .to_ascii_lowercase().as_str();
        let multiplier = match unit.as_str() {
            "kb" => 1_000,
            "k" | "kib" => 1_024,
            "mb" => 1_000_000,
            "m" | "mib" => 1_048_576,
            "gb" => 1_000_000_000,
            "g" | "gib" => 1_073_741_824,

            "tb" => 1_000_000_000_000,
            "t" | "tib" => 1_099_511_627_776,
            "pb" => 1_000_000_000_000_000,
            "p" | "pib" => 1_125_899_906_842_624,
            "eb" => 1_000_000_000_000_000_000,
            "e" | "eib" => 1_152_921_504_606_846_976,

            // #[cfg(not(feature = "cmp_bytes_limit_128_bit"))]
            // // Everything above EiB cannot fit into u64.
            // // GNU cmp just returns an invalid bytes value
            // "z" | "zb" | "zib" | "y" | "yb" | "yib" => {
            //     return Err(ParseBytesError::InvalidUnit);
            // }
            // Everything above EiB cannot fit into u64.
            // GNU cmp just returns an invalid bytes value
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "zb" => 1_000_000_000_000_000_000_000,
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "z" | "zib" => 1_180_591_620_717_411_303_424,
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "yb" => 1_000_000_000_000_000_000_000_000,
            #[cfg(feature = "cmp_bytes_limit_128_bit")]
            "y" | "yib" => 1_208_925_819_614_629_174_706_176,
            _ => {
                return Err(ParseBytesError::InvalidUnit);
            }
        };

        Ok(multiplier)
    }
}
