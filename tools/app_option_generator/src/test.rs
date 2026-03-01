
// AppOptions for cmp_help
// Check everything and add default values!

const OPT_BYTES: AppOption = AppOption {
    long_name: "bytes",
    short: Some('n'),
    has_arg: true,
    arg_default: None,
};
const OPT_HELP: AppOption = AppOption {
    long_name: "help",
    short: Some('-'),
    has_arg: false,
    arg_default: None,
};
const OPT_IGNORE_INITIAL: AppOption = AppOption {
    long_name: "ignore-initial",
    short: Some('i'),
    has_arg: true,
    arg_default: None,
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
    long_name: "quiet,",
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
const OPT_VERSION: AppOption = AppOption {
    long_name: "version",
    short: Some('v'),
    has_arg: false,
    arg_default: None,
};

// Array for ParamsGen
const ARG_OPTIONS: [AppOption; 8] = [
OPT_BYTES,
OPT_HELP,
OPT_IGNORE_INITIAL,
OPT_IGNORE_INITIAL,
OPT_PRINT_BYTES,
OPT_QUIET,
OPT_VERBOSE,
OPT_VERSION,
];

// From function for your parser
impl From<&ParsedOption> for <ParamXxx> {
    fn from(opt: &ParsedOption) -> Self {
        match *opt.app_option {
            OPT_BYTES => todo!(), 
            OPT_HELP => todo!(), 
            OPT_IGNORE_INITIAL => todo!(), 
            OPT_IGNORE_INITIAL => todo!(), 
            OPT_PRINT_BYTES => todo!(), 
            OPT_QUIET => todo!(), 
            OPT_VERBOSE => todo!(), 
            OPT_VERSION => todo!(), 

        // This is not an error, but a todo. Unfortunately an Enum is not possible.

        _ => todo!("Err Option: {}", opt.app_option.long_name),
        }
    }
}
