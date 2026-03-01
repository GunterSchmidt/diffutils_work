/// This is a little helper to create the AppOptions when using ParamsGen.
///
/// Create the file by 'diff --help > diff_help.txt'". \
/// Then 'cargo run -- diff_help.txt'
use std::ffi::OsString;
use std::fmt::Display;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
// mod test;

const SORT_ALPHABETICALLY: bool = false;

/// This contains the args/options the app allows. They must be all of const value.
#[derive(Debug, Default)]
pub struct AppOption {
    pub option: String,
    pub long_name: String,
    pub short: Option<char>,
    pub has_arg: bool,
    pub arg_default: Option<String>,
}

impl Display for AppOption {
    //  const OPT_BYTES_LIMIT: AppOption = AppOption {
    //     long_name: "bytes",
    //     short: Some('n'),
    //     has_arg: true,
    //     arg_default: None,
    // };

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "const {}: AppOption = AppOption {{", self.option)?;
        writeln!(f, "    long_name: \"{}\",", self.long_name)?;
        writeln!(f, "    short: {:?},", self.short)?;
        writeln!(f, "    has_arg: {},", self.has_arg)?;
        writeln!(f, "    arg_default: {:?},", self.arg_default)?;
        write!(f, "}};")
    }
}

fn option_name(long_name: &str) -> String {
    format!("OPT_{}", long_name.to_ascii_uppercase().replace("-", "_"))
}

fn main() {
    let mut args = std::env::args_os();
    if args.len() == 1 {
        let mut msg = "No filename given. Use e.g. 'cargo run -- diff_help.txt'".to_string();
        msg.push_str("\nCreate the file by 'diff --help > diff_help.txt'");
        println!("{msg}");
        return;
    }
    let file_name = args.nth(1).unwrap();

    match read_file(&file_name) {
        Ok(lines) => {
            for (i, line) in lines.iter().enumerate() {
                println!("{}: {}", i + 1, line);
            }
            let mut content = format!("// AppOptions for {}\n", file_name.to_string_lossy());
            content.push_str("// Check everything and add default values!\n\n");
            content.push_str(&parse_to_app_options(&lines));
            let out_file = format!("{}.options.txt", file_name.to_string_lossy());
            println!("Writing result into {out_file}");
            if let Err(e) = write_to_file(&out_file, &content) {
                eprintln!("Error writing file: {}", e);
            }
        }
        Err(e) => eprintln!("Error reading file: {}", e),
    }
}

fn parse_to_app_options(lines: &[String]) -> String {
    println!("\nParsing:");
    let mut opts = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        // find short name
        if line.trim().starts_with("-") {
            println!("{}: {}", i + 1, line);
            let mut opt = AppOption::default();
            let p = line.find('-').unwrap();
            if line.as_bytes()[p + 1] != b'-' {
                opt.short = Some(line.as_bytes()[p + 1] as char);
            }
            // find long name
            match line.find("--") {
                Some(b) => {
                    let e = match line[b..].find(" ") {
                        Some(p) => b + p,
                        None => line.len(),
                    };
                    let mut long = match line[b + 2..e].split_once("=") {
                        Some((long_name, _arg)) => {
                            opt.has_arg = true;
                            long_name.trim().to_string()
                        }
                        None => line[b + 2..e].trim().to_string(),
                    };
                    let has_default = long.find('[');
                    opt.long_name = match has_default {
                        Some(p) => {
                            opt.arg_default = Some("<value>".to_string());
                            _ = long.split_off(p);
                            long
                        }
                        None => long,
                    };
                    opt.option = option_name(&opt.long_name);
                    println!("   Option: {opt}");
                    opts.push(opt);
                }
                None => panic!("no long name found, starting with --"),
            }
        }
    }

    // sort alphabetically
    if SORT_ALPHABETICALLY {
        opts.sort_by_key(|k| k.option.clone());
    }

    let mut content = String::new();
    for opt in opts.iter() {
        println!("{opt}");
        content.push_str(&opt.to_string());
        content.push('\n');
    }

    // create the array
    content.push_str("\n// Array for ParamsGen\n");
    content.push_str(&format!(
        "const ARG_OPTIONS: [AppOption; {}] = [\n",
        opts.len()
    ));
    for opt in opts.iter() {
        content.push_str(&opt.option);
        content.push_str(",\n");
    }
    content.push_str("];\n");

    // create from function
    content.push_str("\n// From function for your parser\n");
    content.push_str("impl From<&ParsedOption> for <ParamXxx> {\n");
    content.push_str("    fn from(opt: &ParsedOption) -> Self {\n");
    content.push_str("        match *opt.app_option {\n");
    for opt in opts.iter() {
        content.push_str(&format!("            {} => todo!(), \n", opt.option));
    }
    content.push_str(
        "\n        // This is not an error, but a todo. Unfortunately an Enum is not possible.\n",
    );
    content.push_str("        _ => todo!(\"Err Option: {}\", opt.app_option.long_name),\n");
    content.push_str("        }\n");
    content.push_str("    }\n");
    content.push_str("}\n");

    content
}

/// Reads a file from the current directory line by line.
fn read_file(filename: &OsString) -> io::Result<Vec<String>> {
    // Create a path relative to the current directory
    let path = Path::new(filename);

    // Open the file in read-only mode
    let file = File::open(path)?;

    // Use a BufReader to efficiently read line by line
    let reader = BufReader::new(file);

    // Collect lines into a Vector, handling potential IO errors per line
    reader.lines().collect()
}

fn write_to_file(filename: &str, content: &str) -> io::Result<()> {
    // fs::write handles opening, truncating, and closing for you
    fs::write(filename, content)?;
    Ok(())
}
