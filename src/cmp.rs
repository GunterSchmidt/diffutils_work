// This file is part of the uutils diffutils package.
//
// For the full copyright and license information, please view the LICENSE-*
// files that was distributed with this source code.

// pub mod params;
pub mod params_cmp;
use crate::arg_parser::{
    add_copyright, format_error_test, get_version_text, Executable, ParseError,
};
use crate::cmp::params_cmp::{CmpParseOk, ParamsCmp};
use crate::utils::format_failure_to_read_input_file;
use std::env::{self, ArgsOs};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::iter::Peekable;
use std::process::ExitCode;
use std::{cmp, fs, io};

/// for --bytes, so really large number limits can be expressed, like 1Y.
pub type BytesLimitU64 = u64;
// ignore initial is currently limited to u64, as take(skip) is used.
pub type SkipU64 = u64;

fn prepare_reader(
    path: &OsString,
    skip: SkipU64,
    params: &ParamsCmp,
) -> Result<Box<dyn BufRead>, String> {
    let mut reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        match fs::File::open(path) {
            Ok(file) => Box::new(BufReader::new(file)),
            Err(e) => {
                return Err(format_failure_to_read_input_file(
                    &params.executable.to_os_string(),
                    path,
                    &e,
                ));
            }
        }
    };

    if skip > 0 {
        if let Err(e) = io::copy(&mut reader.by_ref().take(skip), &mut io::sink()) {
            return Err(format_failure_to_read_input_file(
                &params.executable.to_os_string(),
                path,
                &e,
            ));
        }
    }

    Ok(reader)
}

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

/// Entry into cmp.
///
/// Param options, e.g. 'cmp file1.txt file2.txt -bd n2000kB'.
/// - Program name - Usually 'cmp' as first parameter, but accept any OsString.
/// - cmp options - as documented at <https://www.gnu.org/software/diffutils/manual/html_node/cmp-Options.html>
// Exit codes are documented at
// https://www.gnu.org/software/diffutils/manual/html_node/Invoking-cmp.html
//     An exit status of 0 means no differences were found,
//     1 means some differences were found,
//     and 2 means trouble.
// TODO first param util: Executable,
pub fn main(mut args: Peekable<ArgsOs>) -> ExitCode {
    let Some(executable) = Executable::from_args_os(&mut args, true) else {
        eprintln!("Expected utility name as first argument, got nothing.");
        return ExitCode::FAILURE;
    };
    match cmp(args) {
        Ok(res) => match res {
            CmpOk::Different => ExitCode::FAILURE,
            CmpOk::Equal => ExitCode::SUCCESS,
            CmpOk::Help => {
                println!("{}", add_copyright(TEXT_HELP));
                ExitCode::SUCCESS
            }
            CmpOk::Version => {
                println!("{}", get_version_text(&executable));
                ExitCode::SUCCESS
            }
        },
        Err(e) => {
            let msg = format_error_test(&executable, &e);
            eprintln!("{msg}");
            ExitCode::from(2)
        }
    }
}

/// This is the full sdiff call.
///
/// The first arg needs to be the executable, then the operands and options.
pub fn cmp<I: Iterator<Item = OsString>>(mut args: Peekable<I>) -> Result<CmpOk, CmpError> {
    let Some(executable) = Executable::from_args_os(&mut args, false) else {
        return Err(ParseError::NoExecutable.into());
    };
    // read params
    let params = match ParamsCmp::parse_params(&executable, args)? {
        CmpParseOk::Params(p) => p,
        CmpParseOk::Help => return Ok(CmpOk::Help),
        CmpParseOk::Version => return Ok(CmpOk::Version),
    };

    // dbg!("{params:?}");

    // compare files
    cmp_compare(&params)
}

// TODO struct Cmp
/// This is the main function to compare the files. \
/// Files are limited to u64 bytes and u64 lines.
// TODO CmpError
pub fn cmp_compare(params: &ParamsCmp) -> Result<CmpOk, CmpError> {
    let mut from = prepare_reader(&params.from, params.skip_bytes_from, params)?;
    let mut to = prepare_reader(&params.to, params.skip_bytes_to, params)?;

    let mut offset_width = params.bytes_limit.unwrap_or(BytesLimitU64::MAX);

    if let (Ok(a_meta), Ok(b_meta)) = (fs::metadata(&params.from), fs::metadata(&params.to)) {
        #[cfg(not(target_os = "windows"))]
        let (a_size, b_size) = (a_meta.size(), b_meta.size());

        #[cfg(target_os = "windows")]
        let (a_size, b_size) = (a_meta.file_size(), b_meta.file_size());

        // If the files have different sizes, we already know they are not identical. If we have not
        // been asked to show even the first difference, we can quit early.
        if params.silent && a_size != b_size {
            return Ok(CmpOk::Different);
        }

        let smaller = cmp::min(a_size, b_size) as BytesLimitU64;
        offset_width = cmp::min(smaller, offset_width);
    }

    let offset_width = 1 + offset_width.checked_ilog10().unwrap_or(1) as usize;

    // Capacity calc: at_byte width + 2 x 3-byte octal numbers + 2 x 4-byte value + 4 spaces
    let mut output = Vec::<u8>::with_capacity(offset_width + 3 * 2 + 4 * 2 + 4);

    let mut at_byte: BytesLimitU64 = 1;
    let mut at_line: u64 = 1;
    let mut start_of_line = true;
    let mut stdout = BufWriter::new(io::stdout().lock());
    let mut compare = CmpOk::Equal;
    loop {
        // Fill up our buffers.
        let from_buf = match from.fill_buf() {
            Ok(buf) => buf,
            Err(e) => {
                return Err(format_failure_to_read_input_file(
                    &params.executable.to_os_string(),
                    &params.from,
                    &e,
                )
                .into());
            }
        };

        let to_buf = match to.fill_buf() {
            Ok(buf) => buf,
            Err(e) => {
                return Err(format_failure_to_read_input_file(
                    &params.executable.to_os_string(),
                    &params.to,
                    &e,
                )
                .into());
            }
        };

        // Check for EOF conditions.
        if from_buf.is_empty() && to_buf.is_empty() {
            break;
        }

        if from_buf.is_empty() || to_buf.is_empty() {
            let eof_on = if from_buf.is_empty() {
                &params.from.to_string_lossy()
            } else {
                &params.to.to_string_lossy()
            };

            report_eof(at_byte, at_line, start_of_line, eof_on, params);
            return Ok(CmpOk::Different);
        }

        // Fast path - for long files in which almost all bytes are the same we
        // can do a direct comparison to let the compiler optimize.
        let consumed = std::cmp::min(from_buf.len(), to_buf.len());
        if from_buf[..consumed] == to_buf[..consumed] {
            let last = from_buf[..consumed].last().unwrap();

            // Unclear if this is necessary to prevent errors if the file is larger than Bytes::MAX.
            // Will have an performance impact.
            // if let None = at_byte.checked_add(consumed as Bytes) {
            //     panic!("File larger than {} bytes.", Bytes::MAX);
            // };
            at_byte += consumed as BytesLimitU64;
            at_line += from_buf[..consumed].iter().filter(|&c| *c == b'\n').count() as u64;

            start_of_line = *last == b'\n';

            if let Some(bytes_limit) = params.bytes_limit {
                if at_byte > bytes_limit {
                    break;
                }
            }

            from.consume(consumed);
            to.consume(consumed);

            continue;
        }

        // Iterate over the buffers, the zip iterator will stop us as soon as the
        // first one runs out.
        for (&from_byte, &to_byte) in from_buf.iter().zip(to_buf.iter()) {
            if from_byte != to_byte {
                compare = CmpOk::Different;

                if params.verbose {
                    format_verbose_difference(
                        from_byte,
                        to_byte,
                        at_byte,
                        offset_width,
                        &mut output,
                        params,
                    )?;
                    stdout.write_all(output.as_slice()).map_err(|e| {
                        format!("{}: error printing output: {e}", params.executable)
                    })?;
                    output.clear();
                } else {
                    report_difference(from_byte, to_byte, at_byte, at_line, params);
                    return Ok(CmpOk::Different);
                }
            }

            start_of_line = from_byte == b'\n';
            if start_of_line {
                at_line += 1;
            }

            at_byte += 1;

            if let Some(max_bytes) = params.bytes_limit {
                if at_byte > max_bytes {
                    break;
                }
            }
        }

        // Notify our readers about the bytes we went over.
        from.consume(consumed);
        to.consume(consumed);
    }

    Ok(compare)
}

#[inline]
fn format_octal(byte: u8, buf: &mut [u8; 3]) -> &str {
    *buf = [b' ', b' ', b'0'];

    let mut num = byte;
    let mut idx = 2; // Start at the last position in the buffer

    // Generate octal digits
    while num > 0 {
        buf[idx] = b'0' + num % 8;
        num /= 8;
        idx = idx.saturating_sub(1);
    }

    // SAFETY: the operations we do above always land within ascii range.
    unsafe { std::str::from_utf8_unchecked(&buf[..]) }
}

#[inline]
fn write_visible_byte(output: &mut Vec<u8>, byte: u8) -> usize {
    match byte {
        // Control characters: ^@, ^A, ..., ^_
        0..=31 => {
            output.push(b'^');
            output.push(byte + 64);
            2
        }
        // Printable ASCII (space through ~)
        32..=126 => {
            output.push(byte);
            1
        }
        // DEL: ^?
        127 => {
            output.extend_from_slice(b"^?");
            2
        }
        // High bytes with control equivalents: M-^@, M-^A, ..., M-^_
        128..=159 => {
            output.push(b'M');
            output.push(b'-');
            output.push(b'^');
            output.push(byte - 64);
            4
        }
        // High bytes: M-<space>, M-!, ..., M-~
        160..=254 => {
            output.push(b'M');
            output.push(b'-');
            output.push(byte - 128);
            3
        }
        // Byte 255: M-^?
        255 => {
            output.extend_from_slice(b"M-^?");
            4
        }
    }
}

/// Writes a byte in visible form with right-padding to 4 spaces.
#[inline]
fn write_visible_byte_padded(output: &mut Vec<u8>, byte: u8) {
    const SPACES: &[u8] = b"    ";
    const WIDTH: usize = SPACES.len();

    let display_width = write_visible_byte(output, byte);

    // Add right-padding spaces
    let padding = WIDTH.saturating_sub(display_width);
    output.extend_from_slice(&SPACES[..padding]);
}

/// Formats a byte as a visible string (for non-performance-critical path)
#[inline]
fn format_visible_byte(byte: u8) -> String {
    let mut result = Vec::with_capacity(4);
    write_visible_byte(&mut result, byte);
    // SAFETY: the checks and shifts in write_visible_byte match what cat and GNU
    // cmp do to ensure characters fall inside the ascii range.
    unsafe { String::from_utf8_unchecked(result) }
}

// This function has been optimized to not use the Rust fmt system, which
// leads to a massive speed up when processing large files: cuts the time
// for comparing 2 ~36MB completely different files in half on an M1 Max.
#[inline]
fn format_verbose_difference(
    from_byte: u8,
    to_byte: u8,
    at_byte: BytesLimitU64,
    offset_width: usize,
    output: &mut Vec<u8>,
    params: &ParamsCmp,
) -> Result<(), String> {
    assert!(!params.silent);

    let mut at_byte_buf = itoa::Buffer::new();
    let mut from_oct = [0u8; 3]; // for octal conversions
    let mut to_oct = [0u8; 3];

    if params.print_bytes {
        // "{:>width$} {:>3o} {:4} {:>3o} {}",
        let at_byte_str = at_byte_buf.format(at_byte);
        let at_byte_padding = offset_width.saturating_sub(at_byte_str.len());

        for _ in 0..at_byte_padding {
            output.push(b' ')
        }

        output.extend_from_slice(at_byte_str.as_bytes());

        output.push(b' ');

        output.extend_from_slice(format_octal(from_byte, &mut from_oct).as_bytes());

        output.push(b' ');

        write_visible_byte_padded(output, from_byte);

        output.push(b' ');

        output.extend_from_slice(format_octal(to_byte, &mut to_oct).as_bytes());

        output.push(b' ');

        write_visible_byte(output, to_byte);

        output.push(b'\n');
    } else {
        // "{:>width$} {:>3o} {:>3o}"
        let at_byte_str = at_byte_buf.format(at_byte);
        let at_byte_padding = offset_width - at_byte_str.len();

        for _ in 0..at_byte_padding {
            output.push(b' ')
        }

        output.extend_from_slice(at_byte_str.as_bytes());

        output.push(b' ');

        output.extend_from_slice(format_octal(from_byte, &mut from_oct).as_bytes());

        output.push(b' ');

        output.extend_from_slice(format_octal(to_byte, &mut to_oct).as_bytes());

        output.push(b'\n');
    }

    Ok(())
}

#[inline]
fn report_eof(
    at_byte: BytesLimitU64,
    at_line: u64,
    start_of_line: bool,
    eof_on: &str,
    params: &ParamsCmp,
) {
    if params.silent {
        return;
    }

    if at_byte == 1 {
        eprintln!("{}: EOF on '{}' which is empty", params.executable, eof_on);
    } else if params.verbose {
        eprintln!(
            "{}: EOF on '{}' after byte {}",
            params.executable,
            eof_on,
            at_byte - 1,
        );
    } else if start_of_line {
        eprintln!(
            "{}: EOF on '{}' after byte {}, line {}",
            params.executable,
            eof_on,
            at_byte - 1,
            at_line - 1
        );
    } else {
        eprintln!(
            "{}: EOF on '{}' after byte {}, in line {}",
            params.executable,
            eof_on,
            at_byte - 1,
            at_line
        );
    }
}

fn is_posix_locale() -> bool {
    let locale = if let Ok(locale) = env::var("LC_ALL") {
        locale
    } else if let Ok(locale) = env::var("LC_MESSAGES") {
        locale
    } else if let Ok(locale) = env::var("LANG") {
        locale
    } else {
        "C".to_string()
    };

    locale == "C" || locale == "POSIX"
}

#[inline]
fn report_difference(
    from_byte: u8,
    to_byte: u8,
    at_byte: BytesLimitU64,
    at_line: u64,
    params: &ParamsCmp,
) {
    if params.silent {
        return;
    }

    let term = if is_posix_locale() && !params.print_bytes {
        "char"
    } else {
        "byte"
    };
    print!(
        "{} {} differ: {term} {}, line {}",
        &params.from.to_string_lossy(),
        &params.to.to_string_lossy(),
        at_byte,
        at_line
    );
    if params.print_bytes {
        let char_width = if to_byte >= 0x7F { 2 } else { 1 };
        print!(
            " is {:>3o} {:char_width$} {:>3o} {:char_width$}",
            from_byte,
            format_visible_byte(from_byte),
            to_byte,
            format_visible_byte(to_byte)
        );
    }
    println!();
}

#[cfg(not(target_os = "windows"))]
use std::os::fd::{AsRawFd, FromRawFd};

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

#[cfg(not(target_os = "windows"))]
fn is_stdout_dev_null() -> bool {
    let Ok(dev_null) = fs::metadata("/dev/null") else {
        return false;
    };

    let stdout_fd = io::stdout().lock().as_raw_fd();

    // SAFETY: we have exclusive access to stdout right now.
    let stdout_file = unsafe { fs::File::from_raw_fd(stdout_fd) };
    let Ok(stdout) = stdout_file.metadata() else {
        return false;
    };

    let is_dev_null = stdout.dev() == dev_null.dev() && stdout.ino() == dev_null.ino();

    // Don't let File close the fd. It's unfortunate that File doesn't have a leak_fd().
    std::mem::forget(stdout_file);

    is_dev_null
}

/// The Ok result of cmp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmpOk {
    Different,
    Equal,
    Help,
    Version,
}

/// Errors for cmp.
///
/// To centralize error messages and make it easier to use in a lib.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names, unused)]
pub enum CmpError {
    // parse errors
    ParseError(ParseError),

    // TODO simple string, should be more specific
    GenericString(String),
    // compare errors
    OutputError(String),
    // (msg)
    ReadFileError(String),
}

impl std::error::Error for CmpError {}

impl From<ParseError> for CmpError {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e)
    }
}

impl From<String> for CmpError {
    fn from(str: String) -> Self {
        Self::GenericString(str)
    }
}

impl std::fmt::Display for CmpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            CmpError::ParseError(e) => e.to_string(),
            CmpError::OutputError(msg) | CmpError::ReadFileError(msg) => msg.clone(),
            CmpError::GenericString(msg) => msg.clone(),
        };
        write!(f, "{msg}")
    }
}
