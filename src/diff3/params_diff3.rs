use std::{ffi::OsString, iter::Peekable};

use crate::{
    arg_parser::{self, ArgParser, ArgParserError, Executable, OPT_HELP, OPT_VERSION},
    diff3::params_diff3_def::*,
};

pub type ResultParamsDiff3Parse = Result<ParamsDiff3Ok, ParamsDiff3Error>;

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
    pub fn parse_params<I: Iterator<Item = OsString>>(opts: Peekable<I>) -> ResultParamsDiff3Parse {
        let parser = ArgParser::parse_params(&APP_OPTIONS, opts)?;

        Self::try_from(&parser)
    }

    // match for ArgParser output
    fn try_from(parser: &ArgParser) -> ResultParamsDiff3Parse {
        let mut params = Self::default();
        //  {
        //     // executable: parser.executable.clone(),
        //     ..Default::default()
        // };

        // set options
        for parsed_option in &parser.options_parsed {
            dbg!(parsed_option);
            match *parsed_option.app_option {
                // OPT_DIFF_PROGRAM => params.set_diff_program()?,
                OPT_APPEND_WQ_TO_ED => params.append_wq_to_ed = true,
                OPT_BRACKET_CONFLICTS => params.bracket_conflicts = true,
                OPT_EASY_ONLY => params.easy_only = true,
                OPT_ED => params.ed = true,
                OPT_HELP => return Ok(ParamsDiff3Ok::Info(arg_parser::add_copyright(TEXT_HELP))),
                OPT_INITIAL_TAB => params.initial_tab = true,
                OPT_LABEL => params.label = parsed_option.arg_for_option.clone(),
                OPT_MERGE => params.merge = true,
                OPT_OVERLAP_ONLY => params.overlap_only = true,
                OPT_SHOW_ALL => params.show_all = true,
                OPT_SHOW_OVERLAP => params.show_overlap = true,
                OPT_STRIP_TRAILING_CR => params.strip_trailing_cr = true,
                OPT_TEXT => params.text = true,
                OPT_VERSION => return Ok(ParamsDiff3Ok::Info(TEXT_VERSION.to_string())),

                // This is not an error, but a todo. Unfortunately an Enum is not possible.
                _ => todo!("Err Option: {}", parsed_option.app_option.long_name),
            }
        }

        // set operands
        match parser.operands.len() {
            0 => {
                return Err(ParamsDiff3Error::ArgParserError(ArgParserError::NoOperand(
                    params.executable,
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
                return Err(ParamsDiff3Error::ExtraOperand(
                    parser.operands[2].to_string_lossy().to_string(),
                ));
            }
        }

        // dbg!(&params);
        Ok(ParamsDiff3Ok::ParamsDiff3(params))
    }
}
